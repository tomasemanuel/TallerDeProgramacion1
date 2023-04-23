#[derive(Debug, Clone)]
pub struct BanAnswer {
    pub ban_list: Vec<String>,
}

impl BanAnswer {
    pub fn new(parametros: Vec<String>) -> BanAnswer {
        BanAnswer {
            ban_list: parametros[1..].to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_ban_answer() {
        let parametros: Vec<String> = vec![
            "BAN".to_string(),
            "user_1".to_string(),
            "user_2".to_string(),
        ];
        let ban_answer = BanAnswer::new(parametros);

        assert_eq!(
            ban_answer.ban_list,
            vec!["user_1".to_string(), "user_2".to_string()]
        );
    }
}
