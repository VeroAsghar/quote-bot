use chrono::Utc;
use std::collections::HashMap;

type Author = String;
type Args = String;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Command {
    Add,
    Remove,
    List,
    Random,
    DNE,
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
    pub async fn parse_message(&self, message: String) -> (Author, Command, Option<Args>) {
        let mut author = String::new();
        let mut args: Option<String> = None;
        let mut command: Option<Command> = None;
        if let Some(message) = message.strip_prefix("!quotes") {
            let message = message.trim();
            let mut message: Vec<&str> = message.split(' ').collect();
            if let Some(value) = self.members.get(&message[0].to_ascii_lowercase()) {
                author = value.to_string();
                message.remove(0);
            }
            if let Some(value) = self.commands.get(&message[0].to_ascii_lowercase()) {
                message.remove(0);
                args = Some(message.join(" ").to_string());
                command = Some(*value);
            }
        }
        println!("{:?}", command.unwrap());
        (author, command.unwrap(), args)
    }
    pub async fn add_quote(
        author: String,
            quote: String,
        database: Option<&sqlx::SqlitePool>,
    ) -> String {
        let quote = quote.trim();
        let date = Utc::now().date().to_string();
        let id = sqlx::query!(
            r#"INSERT INTO quotes ( quote, author, date ) VALUES ( ?, ?, ? )"#,
            quote,
            author,
            date
        )
        .execute(database.unwrap())
        .await
        .unwrap()
        .last_insert_rowid();
        format!("Added quote #{}.", id)
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
        let (author, command, args) = bot.parse_message(message).await;
        assert_eq!("Fran", author);
        assert_eq!(Command::Add, command);
        assert_eq!("blah", args.unwrap());
    }
}
