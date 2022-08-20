use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::SystemTime,
};

use chrono::{NaiveDate, Utc};
use serenity::{
    client::Context,
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::{channel::Message, id::UserId},
    prelude::{RwLock, TypeMapKey},
};

#[group]
#[prefix = "quotes"]
#[commands(random, add, amount)]
struct Quotes;

pub struct DatabaseContainer;

impl TypeMapKey for DatabaseContainer {
    type Value = Arc<RwLock<sqlx::SqlitePool>>;
}

pub struct MemberContainer;

impl TypeMapKey for MemberContainer {
    type Value = Arc<RwLock<HashMap<String, String>>>;
}

#[help]
#[individual_command_tip = "If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[strikethrough_commands_tip_in_guild = ""]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[command]
#[description = "Adds a quote to QuoteBot, @Static only"]
#[allowed_roles("Static")]
pub async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let database = data
        .get::<DatabaseContainer>()
        .expect("Expected DatabaseContainer in TypeMap")
        .clone();
    let database = database.read().await;
    let author_display_name = find_author(ctx, args.clone()).await;
    if let Some(author) = author_display_name {
        let (_, quote) = args
            .message()
            .split_once(' ')
            .expect("Args length greater than 1");
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
    } else {
        msg.channel_id.say(&ctx.http, "[Add] Member not found.").await?;
    }
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
#[description = "Display amount of quotes for a given member or total amount if member is not given."]
pub async fn amount(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // Get database connection
    let data = ctx.data.read().await;
    let database = data
        .get::<DatabaseContainer>()
        .expect("Expected DatabaseContainer in TypeMap")
        .clone();
    let database = database.read().await;

    let author_display_name = find_author(ctx, args.clone()).await;

    if let Some(author) = author_display_name {
        let quotes_count = num_of_quotes(&Some(author.clone()), &*database).await;
        let reply = format!(r#"{} has {} quotes saved."#, author, quotes_count);
        msg.channel_id.say(&ctx.http, reply).await?
    } else {
        // Check to see if member is included in args, if empty display total
        // number of quotes
        let mut args = args;
        if args.single::<String>()?.is_empty() {
            let quotes_count = num_of_quotes(&None, &*database).await;
            let reply = format!("QuoteBot has {} quotes saved.", quotes_count);
            msg.channel_id.say(&ctx.http, reply).await?
        } else {
            msg.channel_id.say(&ctx.http, "[Amount] Member not found").await?
        }
    };
    Ok(())
}

async fn find_author(ctx: &Context, mut args: Args) -> Option<String> {
    // Get member hash map
    let data = ctx.data.read().await;
    let members = data
        .get::<MemberContainer>()
        .expect("Expected MemberContainer in TypeMap")
        .clone();

    // Pull out first word from args
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
#[description = "Display random quotes from QuoteBot."]
#[aliases("")]
pub async fn random(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let database = data
        .get::<DatabaseContainer>()
        .expect("Expected DatabaseContainer in TypeMap")
        .clone();
    let database = database.read().await;
    let author_display_name = find_author(ctx, args).await;

    let column_length = num_of_quotes(&author_display_name, &*database).await;
    let rand_rowid: i64 = (SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % column_length as u64)
        .try_into()?;

    if let Some(author) = author_display_name {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serenity::framework::standard::{Args, Delimiter};

    #[test]
    fn author_stripped_from_args() {
        let args = Args::new("fran meow meow", &[Delimiter::Single(' ')]);
        let (author, quote) = args
            .message()
            .split_once(' ')
            .expect("Args length greater than 1");
        assert_eq!(author, "fran");
        assert_eq!(quote, "meow meow");
    }

    #[test]
    fn author_can_be_found() {
        let mut args = Args::new("fran meow", &[Delimiter::Single(' ')]);
        let mut members = HashMap::new();
        members.insert("fran".to_string(), "Fran".to_string());
        if let Ok(first_arg) = args.single::<String>() {
            let author_display_name = members.get(&first_arg.to_ascii_lowercase()).unwrap();
            assert_eq!(author_display_name, "Fran");
        } else {
            panic!();
        }
    }
}
