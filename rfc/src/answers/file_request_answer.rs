#[derive(Debug, Clone)]
pub struct FileRequestAnswer {
    pub file_name: String,
    pub file_size: String,
    pub file_owner: String,
}

impl FileRequestAnswer {
    pub fn new(parametros: Vec<String>) -> FileRequestAnswer {
        if parametros.is_empty() {
            return FileRequestAnswer {
                file_name: "None".to_owned(),
                file_size: "0".to_owned(),
                file_owner: "None".to_string(),
            };
        }
        FileRequestAnswer {
            file_name: parametros[2].clone(),
            file_size: parametros[4].clone(),
            file_owner: "None".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_file_request_answer() {
        let parametros: Vec<String> = vec![
            "FILE_REQUEST".to_string(),
            "user_1".to_string(),
            "file_1".to_string(),
            "file_size".to_string(),
            "100".to_string(),
        ];
        let file_request_answer = FileRequestAnswer::new(parametros);

        assert_eq!(file_request_answer.file_name, "file_1".to_string());
        assert_eq!(file_request_answer.file_size, "100".to_string());
        assert_eq!(file_request_answer.file_owner, "None".to_string());
    }

    #[test]
    fn create_empty_file_request_answer() {
        let parametros: Vec<String> = vec![];
        let file_request_answer = FileRequestAnswer::new(parametros);

        assert_eq!(file_request_answer.file_name, "None".to_string());
        assert_eq!(file_request_answer.file_size, "0".to_string());
        assert_eq!(file_request_answer.file_owner, "None".to_string());
    }
}
