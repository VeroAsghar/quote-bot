use quote_bot::prelude::*;
use serenity::{async_trait, model::prelude::*, prelude::*};
use std::fs::File;
use std::io::BufReader;

struct Robot {
    database: sqlx::SqlitePool,
    bot: Bot,
}

const STATIC_ID: RoleId = RoleId(801823428457267240);

#[async_trait]
impl EventHandler for Robot {
    async fn message(&self, ctx: Context, msg: Message) {
        let sender = msg.author;
        let guild_id = msg.guild_id.unwrap();

        if let Some(parsed_msg) = self.bot.parse_message(&msg.content).await {
            match parsed_msg.command {
                Command::Add => {
                    if sender.has_role(&ctx, guild_id, STATIC_ID).await.unwrap() {
                        let response = Bot::add_quote(
                            parsed_msg.author.unwrap(),
                            &parsed_msg.args.unwrap(),
                            &self.database,
                        )
                        .await;
                        msg.channel_id.say(&ctx, response).await.unwrap();
                    } else {
                        msg.channel_id
                            .say(&ctx, "Insufficient Privileges")
                            .await
                            .unwrap();
                    }
                }
                Command::Length => {
                    let response = Bot::length(parsed_msg.author, &self.database).await;
                    msg.channel_id.say(&ctx, response).await.unwrap();
                }
                Command::Random => {
                    let response = Bot::random(parsed_msg.author, &self.database).await;
                    msg.channel_id.say(&ctx, response).await.unwrap();
                }
                _ => todo!(),
            }
            //                } else if let Some(id) = msg.content.strip_prefix("!manaquotes remove") {
            //                    let id = id.trim().parse::<i64>().unwrap();
            //                    sqlx::query!("DELETE FROM quotes WHERE rowid = ?", id)
            //                        .execute(&self.database)
            //                        .await
            //                        .unwrap();
            //                    let response = format!("Removed quote from the list");
            //                    msg.channel_id.say(&ctx, response).await.unwrap();
            //                }
            //            }
        }
    }
}
async fn setup_bot(bot: &mut Bot) {
    bot.insert_command("add", Command::Add).await;
    bot.insert_command("length", Command::Length)
        .await;
    bot.insert_command("random", Command::Random)
        .await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");
    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Couldn't run database migrations");
    let config_file = File::open("members.json")?;
    let reader = BufReader::new(config_file);
    let config: Config = serde_json::from_reader(reader)?;
    let mut bot = Bot::new();
    for quote_bot::config::Member { name, display_name } in config.members {
        bot.insert_member(&name, &display_name).await;
    }
    setup_bot(&mut bot).await;
    let robot = Robot { database, bot };
    let mut client = Client::builder(&token)
        .event_handler(robot)
        .await
        .expect("Error creating client");
    client.start().await.unwrap();
    Ok(())
}
