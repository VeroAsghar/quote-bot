
use quote_bot::config::Config;
use serenity::framework::StandardFramework;
use serenity::{async_trait, model::prelude::*, prelude::*};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use quote_bot::commands::{DatabaseContainer, MemberContainer};
use std::collections::HashMap;

use quote_bot::commands::QUOTES_GROUP;
use quote_bot::commands::HELP;

struct Handler;


#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}



#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let database = std::env::var("DATABASE_FILE").expect("Expected a database file");
    let members = std::env::var("MEMBERS_FILE").expect("Expected a members json file");

    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(database)
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");
    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Couldn't run database migrations");

    let config_file = File::open(members).expect("Expected member file");
    let reader = BufReader::new(config_file);
    let config: Config =
        serde_json::from_reader(reader).expect("Failed to deserialize members from member file");

    let mut members = HashMap::with_capacity(10);
    for quote_bot::config::Member { name, display_name } in config.members {
        members.insert(name, display_name);
    }

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(">"))
        .help(&HELP)
        .group(&QUOTES_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .type_map_insert::<DatabaseContainer>(Arc::new(RwLock::new(database)))
        .type_map_insert::<MemberContainer>(Arc::new(RwLock::new(members)))
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}
