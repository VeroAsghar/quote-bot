use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::collections::HashMap;
use std::time::SystemTime;

pub struct ParsedMessage {
    pub command: Command,
    pub author: Option<String>,
    pub args: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub members: Vec<Member>,
}

#[derive(Deserialize, Serialize)]
pub struct Member {
    pub name: String,
    pub display_name: String,
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
    pub async fn insert_member(&mut self, name: String, display_name: String) {
        self.members.insert(name, display_name);
    }
    pub async fn insert_command(&mut self, name: String, value: Command) {
        self.commands.insert(name, value);
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
    pub async fn add_quote(author: String, quote: String, database: &sqlx::SqlitePool) -> String {
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
    pub async fn length(author: Option<String>, database: &sqlx::SqlitePool) -> String {
        if author.is_some() {
            let quotes_count = Bot::num_of_quotes(&author, database).await;
            format!(r#"{} has {} quotes saved."#, author.unwrap(), quotes_count)
        } else {
            let quotes_count = Bot::num_of_quotes(&author, database).await;
            format!("Bot has {} quotes saved.", quotes_count)
        }
    }
    pub async fn random(author: Option<String>, database: &sqlx::SqlitePool) -> String {
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
                .format("%b %d, %Y").to_string();
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
                .format("%b %d, %Y").to_string();
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
    #[test]
    fn parse_member_from_json() {
        let json_data = r#"
            {
                "name": "fran",
                "display_name": "Fran"
            }"#;
        let member: Member = serde_json::from_str(json_data).unwrap();
        assert_eq!(member.name, "fran".to_string());
        assert_eq!(member.display_name, "Fran".to_string());
    }
    #[test]
    fn parse_member_list_from_json() {
        let json_data = r#"
            {
                "members": [
                    {
                        "name": "fran",
                        "display_name": "Fran"
                    },
                    {
                        "name": "varek",
                        "display_name": "Varek"
                    }
                ]
            }"#;
        let config: Config = serde_json::from_str(json_data).unwrap();
        assert_eq!(config.members[0].name, "fran".to_string());
        assert_eq!(config.members[1].display_name, "Varek".to_string());
    }
    #[tokio::test]
    async fn parse_member_list_from_json_into_bot() {
        let json_data = r#"
            {
                "members": [
                    {
                        "name": "fran",
                        "display_name": "Fran"
                    },
                    {
                        "name": "varek",
                        "display_name": "Varek"
                    }
                ]
            }"#;
        let config: Config = serde_json::from_str(json_data).unwrap();
        let mut bot = Bot::new();
        for Member { name, display_name } in config.members {
            bot.insert_member(name, display_name).await;
        }
        assert_eq!(*bot.members.get("fran").unwrap(), "Fran".to_string());
        assert_eq!(*bot.members.get("varek").unwrap(), "Varek".to_string());
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
