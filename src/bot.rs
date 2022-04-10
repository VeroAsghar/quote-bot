use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use std::time::SystemTime;

pub struct Bot {
    pub members: HashMap<String, String>,
    commands: HashMap<String, Command>,
}
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Command {
    Add,
    Remove,
    Length,
    Random,
    IgnoreMsg,
}
pub struct ParsedMessage<'bot> {
    pub command: Command,
    pub author: Option<&'bot str>,
    pub args: Option<String>,
}
impl Bot {
    pub fn new() -> Self {
        Bot {
            members: HashMap::new(),
            commands: HashMap::new(),
        }
    }
    pub async fn insert_member(&mut self, name: &str, display_name: &str) {
        self.members.insert(name.to_string(), display_name.to_string());
    }
    pub async fn insert_command(&mut self, name: &str, value: Command) {
        self.commands.insert(name.to_string(), value);
    }
        pub async fn parse_message<'bot>(&'bot self, message: &str) -> Option<ParsedMessage<'bot>> {
        let mut author = None;
        let mut args = None;
        let mut command = Command::IgnoreMsg;
        if let Some(message) = message.strip_prefix("!quotes") {
            let message = message.trim();
            let mut message: Vec<&str> = message.split(' ').collect();
            if let Some(value) = self.members.get(&message[0].to_ascii_lowercase()) {
                author = Some(value.as_str());
                message.remove(0);
            }
            if !message.is_empty() && message[0] != "" {
                if let Some(value) = self.commands.get(&message[0].to_ascii_lowercase()) {
                    message.remove(0);
                    args = Some(message.join(" "));
                    command = *value;
                }
            } else {
                command = Command::Random;
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
    pub async fn add_quote(author: &str, quote: &str, database: &sqlx::SqlitePool) -> String {
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
    pub async fn num_of_quotes(author: &Option<&str>, database: &sqlx::SqlitePool) -> i32 {
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
        pub async fn length(author: Option<&str>, database: &sqlx::SqlitePool) -> String {
        if author.is_some() {
            let quotes_count = Bot::num_of_quotes(&author, database).await;
            format!(r#"{} has {} quotes saved."#, author.unwrap(), quotes_count)
        } else {
            let quotes_count = Bot::num_of_quotes(&author, database).await;
            format!("Bot has {} quotes saved.", quotes_count)
        }
    }
    pub async fn random(author: Option<&str>, database: &sqlx::SqlitePool) -> String {
        let column_length = Bot::num_of_quotes(&author, database).await;
        let rand_rowid: i64 = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            % column_length as u64)
            .try_into()
            .unwrap();

        if let Some(author) = author {
            let quotes = sqlx::query!(r#"SELECT quote, date FROM quotes WHERE author = ?"#, author)
                .fetch_all(database)
                .await
                .unwrap();
            let date = quotes.get(rand_rowid as usize).unwrap().date.clone();
            let date = NaiveDate::parse_from_str(&date, "%Y-%m-%dUTC")
                .unwrap()
                .format("%b %d, %Y")
                .to_string();
            format!(
                r#""{}" - {} ({})"#,
                quotes.get(rand_rowid as usize).unwrap().quote,
                author,
                date,
            )
        } else {
            let quotes = sqlx::query!(r#"SELECT quote, author, date FROM quotes"#)
                .fetch_all(database)
                .await
                .unwrap();
            let date = quotes.get(rand_rowid as usize).unwrap().date.clone();
            let date = NaiveDate::parse_from_str(&date, "%Y-%m-%dUTC")
                .unwrap()
                .format("%b %d, %Y")
                .to_string();
            format!(
                r#""{}" - {} ({})"#,
                quotes.get(rand_rowid as usize).unwrap().quote,
                quotes.get(rand_rowid as usize).unwrap().author,
                date,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parse_random_command_from_empty_tail() {
        let mut bot = Bot::new();
        bot.insert_member("fran", "Fran")
            .await;
        bot.insert_command("add", Command::Add).await;
        bot.insert_command("", Command::Random).await;

        let message = "!quotes";
        if let Some(parsed_msg) = bot.parse_message(message).await {
            assert_eq!(None, parsed_msg.author);
            assert_eq!(Command::Random, parsed_msg.command);
            assert_eq!(None, parsed_msg.args);
        } else {
            panic!();
        }
    }
    #[tokio::test]
    async fn parse_author_and_random_command_from_empty_tail() {
        let mut bot = Bot::new();
        bot.insert_member("fran", "Fran")
            .await;
        bot.insert_command("add", Command::Add).await;
        bot.insert_command("", Command::Random).await;

        let message = "!quotes fran";
        if let Some(parsed_msg) = bot.parse_message(message).await {
            assert_eq!("Fran", parsed_msg.author.unwrap());
            assert_eq!(Command::Random, parsed_msg.command);
            assert_eq!(None, parsed_msg.args);
        } else {
            panic!();
        }
    }

    #[tokio::test]
    async fn parse_author_command_and_quote_from_message() {
        let mut bot = Bot::new();
        bot.insert_member("fran", "Fran")
            .await;
        bot.insert_command("add", Command::Add).await;

        let message = "!quotes fran add blah";
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
        bot.insert_member("fran", "Fran")
            .await;
        bot.insert_command("add", Command::Add).await;

        let message = "meow";
        if let Some(_parsed_msg) = bot.parse_message(message).await {
            panic!();
        }
    }
    #[test]
    fn string_date_can_be_parsed() {
        let string_date = Utc::now().date().to_string();
        let date = NaiveDate::parse_from_str(&string_date, "%Y-%m-%dUTC")
            .unwrap()
            .to_string();
        assert_eq!(string_date, format!("{}UTC", date));
    }
    #[test]
    fn string_date_can_be_parsed_into_correct_format() {
        let string_date = NaiveDate::from_ymd(2015, 9, 5).to_string();
        let date = NaiveDate::parse_from_str(&string_date, "%Y-%m-%d")
            .unwrap()
            .format("%b %d, %Y");
        assert_eq!("Sep 05, 2015", format!("{}", date));
    }
}
