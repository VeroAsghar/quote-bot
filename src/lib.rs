use std::collections::HashMap;

type Command = fn(String) -> String;
struct Bot {
    members: HashMap<String, String>,
    commands: HashMap<String, Command>,
}

impl Bot {
    fn parse_message(&self, message: String) -> String {
        let mut response = "".to_string();
        if let Some(message) = message.strip_prefix("!quotes") {
            let message = message.trim();
            let mut message: Vec<&str> = message.split(' ').collect();
            if let Some(value) = self.members.get(&message[0].to_ascii_lowercase()) {
                let _author = value.to_string().clone();
                message.remove(0);
            }
            if let Some(command_func) = self.commands.get(message[0].to_ascii_lowercase().as_str())
            {
                message.remove(0);
                let args = message.join(" ").to_string();
                response = command_func(args);
            }
        }
        response
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_prefix_author_and_command_from_message() {
        let mut members = HashMap::new();
        let mut commands = HashMap::new();
        let command_dummy: Command = |s| { s };
        members.insert("fran".to_string(), "Fran".to_string());
        commands.insert("add".to_string(), command_dummy as Command);
        let bot = Bot { members, commands };
        let message = "!quotes fran add blah".to_string();
        let response = bot.parse_message(message);
        assert_eq!("blah", response);
    }
}
