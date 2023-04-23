#[derive(Debug, Clone)]
pub struct P2PAnswer {
    pub message: String,
    pub sender: String,
}

impl P2PAnswer {
    pub fn new(parametros: Vec<String>) -> P2PAnswer {
        P2PAnswer {
            message: parametros[1].clone(),
            sender: parametros[2].clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_p2p_answer() {
        let parametros: Vec<String> = vec![
            "P2P".to_string(),
            "message".to_string(),
            "user_1".to_string(),
        ];
        let p2p_answer = P2PAnswer::new(parametros);

        assert_eq!(p2p_answer.message, "message".to_string());
        assert_eq!(p2p_answer.sender, "user_1".to_string());
    }
}
