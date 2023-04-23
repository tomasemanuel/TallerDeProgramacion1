#[derive(Debug, Clone)]
pub struct QuitAnswer {
    pub leave_message: String,
}

impl QuitAnswer {
    /// crea un nuevo Quit Answer para su uso en client_parser.rs
    pub fn new(parametros: Vec<String>) -> QuitAnswer {
        let slice = &parametros[1..parametros.len()];
        let leave_message = slice.join(" ");
        QuitAnswer { leave_message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_quit_answer() {
        let parametros: Vec<String> = vec![
            "QUIT".to_string(),
            "Chau".to_string(),
            "Me".to_string(),
            "Voy".to_string(),
            "Adios".to_string(),
        ];
        let quit_answer = QuitAnswer::new(parametros);
        assert_eq!(quit_answer.leave_message, String::from("Chau Me Voy Adios"));
    }
}
