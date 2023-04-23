use crate::datauser::DataUserFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwayInfo {
    pub away: Option<String>,
}

impl AwayInfo {
    /// crea un nuevo Away Info para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> AwayInfo {
        let len = parametros.len();
        if len > 1 {
            let slice = &parametros[0..len];
            let mut message = slice.join(" ");
            message.remove(0);
            return AwayInfo {
                away: Some(message),
            };
        }
        AwayInfo { away: None }
    }
}

/// Recibe una instancia de AwayInfo y de DataUserFile, si el mensaje de AwayInfo no es none se actualiza el
/// el campo away del data user y se devuelve una copia del mismo
pub fn away(away_info: AwayInfo, datauser: &mut DataUserFile) -> DataUserFile {
    match away_info.away {
        Some(message) => datauser.away = Some(message),
        None => datauser.away = None,
    }
    datauser.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{datauser::DataUserFile, messages};
    use messages::away::away;

    #[test]
    fn initialize_away_with_empty_message() {
        let parametros = vec![];
        let away = AwayInfo::new(parametros);
        assert_eq!(away.away, None);
    }

    #[test]
    fn initialize_away_with_some_message() {
        let parametros = vec!["Me fui a comer".to_string()];
        let away = AwayInfo::new(parametros);

        // Se inicializo con el away message correcto
        if let Some(message) = away.away {
            assert_eq!(message, "Me fui a comer".to_string());
        }
    }

    #[test]
    fn data_user_updates_when_away_function_is_called() {
        let away_info = AwayInfo::new(vec!["Me fui a duchar".to_string()]);
        let mut data_user = DataUserFile::default_for_clients();
        let data_user2 = away(away_info, &mut data_user);

        // La copia percibe los cambios
        if let Some(away_message) = data_user2.away {
            assert_eq!(away_message, "Me fui a duchar".to_string());
        };

        // El original percibe los cambios
        if let Some(away_message) = data_user.away {
            assert_eq!(away_message, "Me fui a duchar".to_string());
        };
    }
}
