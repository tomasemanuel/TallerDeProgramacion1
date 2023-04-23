#[derive(Debug, Clone)]
pub struct StartDCCAnswer {
    pub nick: String,
}

impl StartDCCAnswer {
    pub fn new(parameters: Vec<String>) -> StartDCCAnswer {
        StartDCCAnswer {
            nick: parameters[0].clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_start_dcc_answer() {
        let parameters: Vec<String> = vec!["user_1".to_string()];
        let start_dcc_answer = StartDCCAnswer::new(parameters);

        assert_eq!(start_dcc_answer.nick, "user_1".to_string());
    }
}
