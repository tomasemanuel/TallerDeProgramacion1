#[derive(Debug, Clone)]
pub struct KickAnswer {
    pub channel: String,
    pub comment: Option<String>,
}

impl KickAnswer {
    /// crea un nuevo Kick Answer para su uso en client_parser.rs
    pub fn new(parametros: Vec<String>) -> KickAnswer {
        let channel = parametros[1].clone();
        let mut comment = None;
        if parametros.len() > 2 {
            let slice = &parametros[2..parametros.len()];
            comment = Some(slice.join(" "));
        }
        KickAnswer { channel, comment }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_kick_answer() {
        let parametros: Vec<String> = vec![
            "KICK".to_string(),
            "channel_1".to_string(),
            "user_1".to_string(),
            "comment".to_string(),
        ];
        let kick_answer = KickAnswer::new(parametros);

        assert_eq!(kick_answer.channel, "channel_1".to_string());
        assert_eq!(kick_answer.comment, Some("user_1 comment".to_string()));
    }
}
