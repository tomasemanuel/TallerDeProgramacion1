#[derive(Debug, Clone)]
pub struct DCCCloseAnswer {
    pub name: String,
}

impl DCCCloseAnswer {
    pub fn new(parametros: Vec<String>) -> DCCCloseAnswer {
        DCCCloseAnswer {
            name: parametros[1].clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_dcc_close_answer() {
        let parametros: Vec<String> = vec!["DCC_CLOSE".to_string(), "user_1".to_string()];
        let dcc_close_answer = DCCCloseAnswer::new(parametros);

        assert_eq!(dcc_close_answer.name, "user_1".to_string());
    }
}
