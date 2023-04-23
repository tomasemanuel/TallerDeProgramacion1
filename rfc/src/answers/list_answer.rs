#[derive(Debug, Clone)]
pub struct ListAnswer {
    pub channel_list: Vec<String>,
}

impl ListAnswer {
    /// crea un nuevo List Answer para su uso en client_parser.rs
    pub fn new(mut parametros: Vec<String>) -> ListAnswer {
        parametros.remove(0);
        let mut channel_list: Vec<String> = Vec::new();
        if !parametros.is_empty() {
            let parametros_split = parametros[0].split(',');
            channel_list = parametros_split
                .into_iter()
                .map(|a| a.replace(':', " "))
                .collect();
        }
        ListAnswer { channel_list }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_list_answer() {
        let parametros: Vec<String> = vec!["LIST".to_string(), "channel_1,channel_2".to_string()];
        let list_answer = ListAnswer::new(parametros);

        assert_eq!(
            list_answer.channel_list,
            vec!["channel_1".to_string(), "channel_2".to_string()]
        );
    }
}
