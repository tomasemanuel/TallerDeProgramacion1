#[derive(Debug, Clone)]
pub struct ServerAnswer {
    pub server_name_to_connect: String,
    pub server_name: String,
}

impl ServerAnswer {
    /// crea un nuevo Server Answer para su uso en client_parser.rs
    pub fn new(parametros: Vec<String>) -> ServerAnswer {
        ServerAnswer {
            server_name_to_connect: parametros[1].clone(),
            server_name: parametros[2].clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_server_answer() {
        let parametros: Vec<String> = vec![
            "SERVER ".to_string(),
            "nombre_conectar".to_string(),
            "nombre".to_string(),
        ];
        let server_answer = ServerAnswer::new(parametros);
        assert_eq!(
            server_answer.server_name_to_connect,
            String::from("nombre_conectar")
        );
        assert_eq!(server_answer.server_name, String::from("nombre"));
    }
}
