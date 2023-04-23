#[derive(Debug, Clone)]
pub struct WhoAnswer {
    pub matches: String,
}

impl WhoAnswer {
    pub fn new(parametros: Vec<String>) -> WhoAnswer {
        if parametros.len() == 1 {
            WhoAnswer {
                matches: "".to_owned(),
            }
        } else {
            WhoAnswer {
                matches: parametros[1].clone(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_who_answer() {
        let parametros: Vec<String> = vec!["WHO".to_string(), "user_1".to_string()];
        let who_answer = WhoAnswer::new(parametros);

        assert_eq!(who_answer.matches, "user_1".to_string());
    }

    #[test]
    fn create_who_answer_empty() {
        let parametros: Vec<String> = vec!["WHO".to_string()];
        let who_answer = WhoAnswer::new(parametros);

        assert_eq!(who_answer.matches, "".to_string());
    }
}
