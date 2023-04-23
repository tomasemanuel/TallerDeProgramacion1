#[derive(Debug, Clone)]
pub struct ModeAnswer {
    pub flag: String,
    pub channel: String,
    pub new_channel_name: String,
}

impl ModeAnswer {
    pub fn new(parametros: Vec<String>) -> ModeAnswer {
        ModeAnswer {
            flag: parametros[1].clone(),
            channel: parametros[2].clone(),
            new_channel_name: parametros[3].clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_mode_answer() {
        let parametros: Vec<String> = vec![
            "MODE".to_string(),
            "+".to_string(),
            "#channel_1".to_string(),
            "channel_1".to_string(),
        ];
        let mode_answer = ModeAnswer::new(parametros);

        assert_eq!(mode_answer.flag, "+".to_string());
        assert_eq!(mode_answer.channel, "#channel_1".to_string());
        assert_eq!(mode_answer.new_channel_name, "channel_1".to_string());
    }
}
