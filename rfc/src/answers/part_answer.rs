#[derive(Debug, Clone)]
pub struct PartAnswer {
    pub from_channels: Vec<String>,
}

impl PartAnswer {
    /// crea un nuevo Part Answer para su uso en client_parser.rs
    pub fn new(mut parametros: Vec<String>) -> PartAnswer {
        let mut from_channels: Vec<String> = Vec::new();
        parametros.remove(0);

        let channels = parametros[0].split(',');
        let channels_vec = channels.collect::<Vec<&str>>();
        for channel in channels_vec.iter() {
            from_channels.push(channel.to_string());
        }
        PartAnswer { from_channels }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_part_answer() {
        let parametros: Vec<String> = vec!["PART".to_string(), "#channel_1,#channel_2".to_string()];
        let part_answer = PartAnswer::new(parametros);
        assert_eq!(part_answer.from_channels, vec!["#channel_1", "#channel_2"]);
    }
}
