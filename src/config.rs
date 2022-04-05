use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub members: Vec<Member>,
}

#[derive(Deserialize, Serialize)]
pub struct Member {
    pub name: String,
    pub display_name: String,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::Bot;

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
}
