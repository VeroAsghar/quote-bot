use std::{time::SystemTime, sync::Arc, collections::HashMap};

use chrono::{NaiveDate, Utc};
use serenity::{
    client::Context,
    framework::standard::{macros::{command, group}, Args, CommandResult},
    model::channel::Message, prelude::{TypeMapKey, RwLock},
};


#[group]
#[prefix = "quotes"]
#[commands(random, add, amount)]
#[default_command(random)]
struct Quotes;

struct DatabaseContainer;

impl TypeMapKey for DatabaseContainer {
    type Value = Arc<RwLock<sqlx::SqlitePool>>;
}

struct MemberContainer;

impl TypeMapKey for MemberContainer {
    type Value = Arc<RwLock<HashMap<String, String>>>;
}

#[command]
#[allowed_roles("Static")]
pub async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let quote = args.message();
    let data = ctx.data.read().await;
    let database = data
        .get::<DatabaseContainer>()
        .expect("Expected DatabaseContainer in TypeMap")
        .clone();
    let database = database.read()
        .await;
    let author = find_author(ctx, args.clone()).await;
    let date = Utc::now().date().to_string();
    let id = sqlx::query!(
        r#"INSERT INTO quotes ( quote, author, date ) VALUES ( ?, ?, ? )"#,
        quote,
        author,
        date
    )
    .execute(&*database)
    .await?
    .last_insert_rowid();
    let reply = format!("Added quote #{}.", id);
    msg.channel_id.say(&ctx.http, reply).await?;
    Ok(())
}

async fn num_of_quotes(author: &Option<String>, database: &sqlx::SqlitePool) -> i32 {
    if let Some(author) = author.as_deref() {
        let quotes = sqlx::query!(
            r"SELECT COUNT(*) as count FROM quotes WHERE author = ?",
            author
        )
        .fetch_one(database)
        .await
        .unwrap();
        quotes.count
    } else {
        let quotes = sqlx::query!(r"SELECT COUNT(*) as count FROM quotes")
            .fetch_one(database)
            .await
            .unwrap();
        quotes.count
    }
}

#[command]
pub async fn amount(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let database = data
        .get::<DatabaseContainer>()
        .expect("Expected DatabaseContainer in TypeMap")
        .clone();
    let database = database.read()
        .await;
    let author = find_author(ctx, args).await;
    if let Some(author) = author {
        let quotes_count = num_of_quotes(&Some(author.clone()), &*database).await;
        let reply = format!(r#"{} has {} quotes saved."#, author, quotes_count);
        msg.channel_id.say(&ctx.http, reply).await?
    } else {
        let quotes_count = num_of_quotes(&None, &*database).await;
        let reply = format!("Bot has {} quotes saved.", quotes_count);
        msg.channel_id.say(&ctx.http, reply).await?
    };
    Ok(())
}

async fn find_author(ctx: &Context, mut args: Args) -> Option<String> {
    let data = ctx.data.read().await;
    let members = data
        .get::<MemberContainer>()
        .expect("Expected MemberContainer in TypeMap")
        .clone();
    if let Ok(first_arg) = args.single::<String>() {
        Some(
            members
                .read()
                .await
                .get(&first_arg.to_ascii_lowercase())?
                .to_string(),
        )
    } else {
        None
    }
}

#[command]
pub async fn random(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let database = data
        .get::<DatabaseContainer>()
        .expect("Expected DatabaseContainer in TypeMap")
        .clone();
    let database = database.read()
        .await;
    let author = find_author(ctx, args).await;

    let column_length = num_of_quotes(&author, &*database).await;
    let rand_rowid: i64 = (SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % column_length as u64)
        .try_into()?;

    if let Some(author) = author {
        let quotes = sqlx::query!(r#"SELECT quote, date FROM quotes WHERE author = ?"#, author)
            .fetch_all(&*database)
            .await?;
        let date = quotes.get(rand_rowid as usize).unwrap().date.clone();
        let date = NaiveDate::parse_from_str(&date, "%Y-%m-%dUTC")?
            .format("%b %d, %Y")
            .to_string();
        let reply = format!(
            r#""{}" - {} ({})"#,
            quotes.get(rand_rowid as usize).unwrap().quote,
            author,
            date,
        );
        msg.channel_id.say(&ctx.http, reply).await?;
    } else {
        let quotes = sqlx::query!(r#"SELECT quote, author, date FROM quotes"#)
            .fetch_all(&*database)
            .await?;
        let date = quotes.get(rand_rowid as usize).unwrap().date.clone();
        let date = NaiveDate::parse_from_str(&date, "%Y-%m-%dUTC")?
            .format("%b %d, %Y")
            .to_string();
        let reply = format!(
            r#""{}" - {} ({})"#,
            quotes.get(rand_rowid as usize).unwrap().quote,
            quotes.get(rand_rowid as usize).unwrap().author,
            date,
        );
        msg.channel_id.say(&ctx.http, reply).await?;
    }
    Ok(())
}
