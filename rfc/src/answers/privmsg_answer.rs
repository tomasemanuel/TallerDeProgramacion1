#[derive(Debug, Clone)]
pub struct PrivmsgAnswer {
    pub from_channel: Option<String>,
    pub from_user: String,
    pub message: String,
}

// DCC CHAT <protocolo><ip><port>
// DCC CHAT chat<ip><port>

impl PrivmsgAnswer {
    /// crea un nuevo PRIVMSG Answer para su uso en client_parser.rs
    pub fn new(parametros: Vec<String>) -> PrivmsgAnswer {
        let from_channel_split = parametros[1].split(',');
        let from_channel_vec = from_channel_split.collect::<Vec<&str>>();
        let slice = &parametros[2..parametros.len()];
        let message = slice.join(" ").replace(':', "");
        if from_channel_vec.len() > 1 {
            let channel_name = from_channel_vec[0].to_owned();
            return PrivmsgAnswer {
                from_channel: Some(channel_name),
                from_user: from_channel_vec[1].to_string(),
                message,
            };
        }
        PrivmsgAnswer {
            from_channel: None,
            from_user: from_channel_vec[0].to_string(),
            message,
        }
    }
}

// //["PRIVMSG","canal,remitente","mensaje1","mensaje2","mensaje n" n];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_privmsg_answer_successfully() {
        let parametros = vec![
            "PRIVMSG".to_string(),
            "remitente".to_string(),
            "Hola".to_string(),
            "que".to_string(),
            "tal".to_string(),
        ];

        let privmsg = PrivmsgAnswer::new(parametros);

        assert_eq!(privmsg.from_user, "remitente".to_string());
        assert_eq!(privmsg.message, "Hola que tal".to_string());
    }

    #[test]
    fn create_privmsg_answer_from_channel() {
        let parametros = vec![
            "PRIVMSG".to_string(),
            "canal1,remitente1".to_string(),
            "Hola".to_string(),
            "que".to_string(),
            "tal".to_string(),
        ];

        let privmsg = PrivmsgAnswer::new(parametros);

        assert_eq!(privmsg.from_user, "remitente1".to_string());
        assert_eq!(privmsg.message, "Hola que tal".to_string());

        if let Some(canal) = privmsg.from_channel {
            assert_eq!(canal, "canal1".to_string());
        }
    }
}
