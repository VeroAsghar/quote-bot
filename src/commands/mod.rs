use serenity::{
    client::Context,
    framework::standard::{
        help_commands, macros::help, Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::{channel::Message, id::UserId},
    prelude::{RwLock, TypeMapKey},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub mod images;
pub mod quotes;

pub use images::IMAGES_GROUP;
pub use quotes::QUOTES_GROUP;

pub struct DatabaseContainer;

impl TypeMapKey for DatabaseContainer {
    type Value = Arc<RwLock<sqlx::SqlitePool>>;
}

pub struct MemberContainer;

impl TypeMapKey for MemberContainer {
    type Value = Arc<RwLock<HashMap<String, String>>>;
}

mod helper {
    use serenity::{client::Context, framework::standard::Args};

    use crate::commands::MemberContainer;

    pub async fn find_author(ctx: &Context, mut args: Args) -> Option<String> {
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

    pub async fn num_of_quotes(author: &Option<String>, database: &sqlx::SqlitePool) -> i32 {
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

    pub async fn num_of_images(author: &Option<String>, database: &sqlx::SqlitePool) -> i32 {
        if let Some(author) = author.as_deref() {
            let images = sqlx::query!(
                r"SELECT COUNT(*) as count FROM images WHERE author = ?",
                author
            )
            .fetch_one(database)
            .await
            .unwrap();
            images.count
        } else {
            let images = sqlx::query!(r"SELECT COUNT(*) as count FROM images")
                .fetch_one(database)
                .await
                .unwrap();
            images.count
        }
    }
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
