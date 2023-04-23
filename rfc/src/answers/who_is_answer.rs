#[derive(Debug, Clone)]
pub struct WhoIsAnswer {
    pub operator: bool,
    pub joined_channels: Option<String>,
    pub away: bool,
}

impl WhoIsAnswer {
    /// crea un nuevo Whois Answer para su uso en client_parser.rs
    pub fn new(parametros: Vec<String>) -> WhoIsAnswer {
        let op_split = parametros[1].split(':');
        let op_vec = op_split.collect::<Vec<&str>>();

        let op_answer = match op_vec[1] {
            "true" => true,
            "false" => false,
            _ => false,
        };

        let joined_channels_split = parametros[2][3..parametros[2].len()].to_string();

        let joined_channels_split_str = joined_channels_split.split(';');
        let joined_channels_vec_str = joined_channels_split_str.collect::<Vec<&str>>();

        let mut joined_channels_answer = String::from("");
        for channel in joined_channels_vec_str.iter() {
            joined_channels_answer.push_str(channel);
            joined_channels_answer.push(',');
        }
        if !joined_channels_answer.is_empty() {
            joined_channels_answer.pop();
        }
        let away_split = parametros[3].split(':');
        let away_vec = away_split.collect::<Vec<&str>>();

        let away_answer = match away_vec[1] {
            "true" => true,
            "false" => false,
            _ => false,
        };
        if joined_channels_answer.is_empty() {
            return WhoIsAnswer {
                operator: op_answer,
                joined_channels: None,
                away: away_answer,
            };
        }
        WhoIsAnswer {
            operator: op_answer,
            joined_channels: Some(joined_channels_answer),
            away: away_answer,
        }
    }
}
