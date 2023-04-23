#[derive(Debug, Clone)]
pub struct InviteAnswer {
    pub channel_name: String,
    pub nick: String,
}

impl InviteAnswer {
    /// crea un nuevo Invite Answer para su uso en client_parser.rs
    pub fn new(parametros: Vec<String>) -> InviteAnswer {
        if parametros.is_empty() {
            return InviteAnswer {
                channel_name: "".to_owned(),
                nick: "".to_owned(),
            };
        }
        InviteAnswer {
            channel_name: parametros[1].clone(),
            nick: parametros[2].clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_invite_answer() {
        let parametros: Vec<String> = vec![
            "INVITE".to_string(),
            "channel_1".to_string(),
            "user_1".to_string(),
        ];
        let invite_answer = InviteAnswer::new(parametros);

        assert_eq!(invite_answer.channel_name, "channel_1".to_string());
        assert_eq!(invite_answer.nick, "user_1".to_string());
    }

    #[test]
    fn create_empty_invite_answer() {
        let parametros: Vec<String> = vec![];
        let invite_answer = InviteAnswer::new(parametros);

        assert_eq!(invite_answer.channel_name, "".to_string());
        assert_eq!(invite_answer.nick, "".to_string());
    }
}
