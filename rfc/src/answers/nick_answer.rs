#[derive(Debug, Clone)]
pub struct NickAnswer {
    pub new_nick: String,
}

impl NickAnswer {
    pub fn new(parametros: Vec<String>) -> NickAnswer {
        NickAnswer {
            new_nick: parametros[1].clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_nick_answer() {
        let parametros: Vec<String> = vec!["NICK".to_string(), "user_1".to_string()];
        let nick_answer = NickAnswer::new(parametros);

        assert_eq!(nick_answer.new_nick, "user_1".to_string());
    }
}
