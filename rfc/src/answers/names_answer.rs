#[derive(Debug, Clone)]
pub struct NamesAnswer {
    pub nickname_list: Vec<String>,
    pub channel_list: Vec<String>,
}

impl NamesAnswer {
    /// crea un nuevo Names Answer para su uso en client_parser.rs
    pub fn new(parametros: Vec<String>) -> NamesAnswer {
        let mut nickname_list: Vec<String> = Vec::new();
        let mut channel_list: Vec<String> = Vec::new();
        let mut i = 1;
        while i < parametros.len() - 1 {
            channel_list.push(parametros[i].clone());
            nickname_list.push(parametros[i + 1].clone());
            i += 2;
        }
        let mut split_nicklist: Vec<String> = Vec::new();
        for name in nickname_list {
            let new = name.split(',');
            for i in new {
                split_nicklist.push(i.to_string());
            }
        }
        NamesAnswer {
            nickname_list: split_nicklist,
            channel_list,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_names_answer() {
        let parametros: Vec<String> = vec![
            "NAMES".to_string(),
            "chanel_1:".to_string(),
            "nickname_1".to_string(),
            "chanel_2:".to_string(),
            "nickname_2".to_string(),
        ];
        let names_answer = NamesAnswer::new(parametros);

        assert_eq!(
            names_answer.nickname_list,
            vec!["nickname_1".to_string(), "nickname_2".to_string()]
        );
    }
}
