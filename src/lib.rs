use chrono::Utc;
use std::collections::HashMap;
use std::time::SystemTime;

type Author = String;
type Args = String;

pub struct ParsedMessage {
    pub command: Command,
    pub author: Option<Author>,
    pub args: Option<Args>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Command {
    Add,
    Remove,
    Length,
    Random,
    IgnoreMsg,
}

pub struct Bot {
    members: HashMap<String, String>,
    commands: HashMap<String, Command>,
}

impl Bot {
    pub fn new() -> Self {
        Bot {
            members: HashMap::new(),
            commands: HashMap::new(),
        }
    }
    pub async fn insert_member(&mut self, key: String, value: String) {
        self.members.insert(key, value);
    }
    pub async fn insert_command(&mut self, key: String, value: Command) {
        self.commands.insert(key, value);
    }
    pub async fn parse_message(&self, message: String) -> Option<ParsedMessage> {
        let mut author = None;
        let mut args = None;
        let mut command = Command::IgnoreMsg;
        if let Some(message) = message.strip_prefix("!quotes") {
            let message = message.trim();
            let mut message: Vec<&str> = message.split(' ').collect();
            if let Some(value) = self.members.get(&message[0].to_ascii_lowercase()) {
                author = Some(value.to_string());
                message.remove(0);
            }
            if let Some(value) = self.commands.get(&message[0].to_ascii_lowercase()) {
                message.remove(0);
                args = Some(message.join(" ").to_string());
                command = *value;
            }
        }
        if command == Command::IgnoreMsg {
            None
        } else {
            Some(ParsedMessage {
                command,
                author,
                args,
            })
        }
    }
    pub async fn add_quote(author: Author, quote: String, database: &sqlx::SqlitePool) -> String {
        let quote = quote.trim();
        let date = Utc::now().date().to_string();
        let id = sqlx::query!(
            r#"INSERT INTO quotes ( quote, author, date ) VALUES ( ?, ?, ? )"#,
            quote,
            author,
            date
        )
        .execute(database)
        .await
        .unwrap()
        .last_insert_rowid();
        format!("Added quote #{}.", id)
    }
    pub async fn num_of_quotes(author: &Option<Author>, database: &sqlx::SqlitePool) -> i32 {
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
    pub async fn length(author: Option<Author>, database: &sqlx::SqlitePool) -> String {
        if author.is_some() {
            let quotes_count = Bot::num_of_quotes(&author, database).await;
            format!(r#"{} has {} quotes saved."#, author.unwrap(), quotes_count)
        } else {
            let quotes_count = Bot::num_of_quotes(&author, database).await;
            format!("Bot has {} quotes saved.", quotes_count)
        }
    }
    pub async fn random(author: Option<Author>, database: &sqlx::SqlitePool) -> String {
        let column_length = Bot::num_of_quotes(&author, database).await;
        let rand_rowid: i64 = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            % column_length as u64)
            .try_into()
            .unwrap();

        if let Some(author) = author {
            let quotes = sqlx::query!(r#"SELECT quote FROM quotes WHERE author = ?"#, author)
                .fetch_all(database)
                .await
                .unwrap();
            let mut actual_quotes = Vec::new();
            for quote in quotes.iter() {
                actual_quotes.push(quote.quote.clone());
            }
            format!(
                r#""{}" - {}"#,
                actual_quotes.get(rand_rowid as usize).unwrap(),
                author
            )
        } else {
            let quotes = sqlx::query!(r#"SELECT quote, author FROM quotes"#)
                .fetch_all(database)
                .await
                .unwrap();
            format!(
                r#""{}" - {}"#,
                quotes.get(rand_rowid as usize).unwrap().quote,
                quotes.get(rand_rowid as usize).unwrap().author,
            )
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn strip_prefix_author_and_command_from_message() {
        let mut bot = Bot::new();
        bot.insert_member("fran".to_string(), "Fran".to_string())
            .await;
        bot.insert_command("add".to_string(), Command::Add).await;

        let message = "!quotes fran add blah".to_string();
        if let Some(parsed_msg) = bot.parse_message(message).await {
            assert_eq!("Fran", parsed_msg.author.unwrap());
            assert_eq!(Command::Add, parsed_msg.command);
            assert_eq!("blah", parsed_msg.args.unwrap());
        } else {
            panic!();
        }
    }

    #[tokio::test]
    async fn ignore_message_without_parsable_command() {
        let mut bot = Bot::new();
        bot.insert_member("fran".to_string(), "Fran".to_string())
            .await;
        bot.insert_command("add".to_string(), Command::Add).await;

        let message = "meow".to_string();
        if let Some(_parsed_msg) = bot.parse_message(message).await {
            panic!();
        }
    }
}
