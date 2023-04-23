#[derive(Debug, Clone)]
pub struct TopicAnswer {
    pub topic: Option<String>,
}

impl TopicAnswer {
    /// crea un nuevo Topic Answer para su uso en client_parser.rs
    pub fn new(parametros: Vec<String>) -> TopicAnswer {
        if parametros.is_empty() {
            return TopicAnswer { topic: None };
        }

        let slice = &parametros[1..parametros.len()];
        let topic_message = slice.join(" ");
        if topic_message.as_str().contains("RPL_NOTOPIC") || topic_message.is_empty() {
            return TopicAnswer { topic: None };
        }
        TopicAnswer {
            topic: Some(topic_message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TopicAnswer;

    // use super::*;
    #[test]
    fn create_topic_answer_with_topic() {
        let parametros: Vec<String> = vec![
            "RPL_TOPIC ".to_string(),
            "este".to_string(),
            "es".to_string(),
            "nuevo".to_string(),
            "topic".to_string(),
        ];
        let topic_answer = TopicAnswer::new(parametros);
        match topic_answer.topic {
            Some(topic) => assert_eq!(topic, String::from("este es nuevo topic")),
            None => assert_eq!(true, false),
        }
    }
    #[test]
    fn create_topic_answer_without_topic() {
        let topic_answer = TopicAnswer::new(vec![]);
        match topic_answer.topic {
            Some(_topic) => assert_eq!(true, false),
            None => assert_eq!(true, true),
        }
    }

    #[test]
    fn create_topic_with_rpl() {
        let parametros: Vec<String> = vec!["TOPIC".to_string(), "RPL_NOTOPIC".to_string()];
        let topic_answer = TopicAnswer::new(parametros);
        match topic_answer.topic {
            Some(_topic) => assert_eq!(true, false),
            None => assert_eq!(true, true),
        }
    }
}
