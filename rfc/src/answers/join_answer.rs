#[derive(Debug, Clone)]
pub struct JoinAnswer {
    pub channel_name: String,
    pub channel_users: Option<Vec<String>>,
}

impl JoinAnswer {
    /// crea un nuevo Join Answer para su uso en client_parser.rs
    pub fn new(parametros: Vec<String>) -> JoinAnswer {
        let channel_name = parametros[1].clone();
        let mut channel_users = None;
        if parametros.len() > 2 {
            channel_users = Some(parametros[3].split(',').map(String::from).collect());
        }
        JoinAnswer {
            channel_name,
            channel_users,
        }
    }
}
//channel_users = Some(parametros[3].split(',').map(|s| String::from(s)).collect());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_join_answer_successfully() {
        let parametros = vec!["user1,user2,user3".to_string(), "channel1".to_string()];

        let join_answer = JoinAnswer::new(parametros);
        assert_eq!(join_answer.channel_name, "channel1".to_string());

        if let Some(channel_users) = join_answer.channel_users {
            assert_eq!(channel_users[0], "user1".to_string());
            assert_eq!(channel_users[1], "user2".to_string());
            assert_eq!(channel_users[2], "user3".to_string());
        }
    }
}
