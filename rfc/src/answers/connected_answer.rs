#[derive(Debug, Clone)]
pub struct ChannelListAnswer {
    pub channel_list: Vec<String>,
}

impl ChannelListAnswer {
    /// crea un nuevo Channel_list Answer para su uso en client_parser.rs
    pub fn new(parametros: Vec<String>) -> ChannelListAnswer {
        if parametros.is_empty() {
            return ChannelListAnswer {
                channel_list: vec!["".to_string()],
            };
        }
        let channel_list = parametros[1]
            .split(',')
            .into_iter()
            .map(|str| str.to_owned())
            .collect();
        ChannelListAnswer { channel_list }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_empty_channel_list_answer() {
        let parametros: Vec<String> = vec![];
        let answer = ChannelListAnswer::new(parametros);

        let expected: Vec<String> = vec!["".to_string()];
        assert_eq!(answer.channel_list, expected);
    }

    #[test]
    fn create_channel_list_answer() {
        let parametros: Vec<String> = vec!["".to_string(), "neymar,messi,jordi-alba".to_string()];
        let answer = ChannelListAnswer::new(parametros);

        let expected: Vec<String> = vec![
            "neymar".to_string(),
            "messi".to_string(),
            "jordi-alba".to_string(),
        ];
        assert_eq!(answer.channel_list, expected);
    }
}
