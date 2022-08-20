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

    #[test]
    fn parse_member_from_json() {
        let json_data = r#"
            {
                "name": "fran",
                "display_name": "Fran"
            }"#;
        let member: Member = serde_json::from_str(json_data).unwrap();
        assert_eq!(member.name, "fran");
        assert_eq!(member.display_name, "Fran");
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
        assert_eq!(config.members[0].name, "fran");
        assert_eq!(config.members[1].display_name, "Varek");
    }
}
