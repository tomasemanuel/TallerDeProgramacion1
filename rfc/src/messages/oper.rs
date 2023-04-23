use std::sync::{Arc, RwLock};

use crate::datauser::DataUserFile;

static MAX_ARG_OPER: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct OperInfo {
    pub nick: String,
    pub pass: String,
}

impl OperInfo {
    /// crea un nuevo Oper Info para su uso en parser.rs
    pub fn new(parametro: Vec<String>) -> Result<OperInfo, String> {
        if parametro.len() < MAX_ARG_OPER {
            return Err(String::from("ERR_NEEDMOREPARAMS"));
        }
        if parametro.len() > MAX_ARG_OPER + 1 {
            return Err(String::from("ERR_NEEDMOREPARAMS"));
        }
        Ok(OperInfo {
            nick: parametro[0].to_string(),
            pass: parametro[1].to_string(),
        })
    }
}

/// Dependiendo la contraseña y el nickname, añade a un usuario a la lista de operadores del server
/// dandole todos sus privilegios
pub fn set_operator(
    data_user: DataUserFile,
    oper_info: OperInfo,
    server_operators: &Arc<RwLock<Vec<String>>>,
) -> Result<String, String> {
    if oper_info.nick != data_user.nickname || oper_info.pass != data_user.password {
        return Err(String::from("ERR_PASSWDMISMATCH"));
    }
    match server_operators.write() {
        Ok(mut server_operators) => {
            if server_operators.contains(&data_user.nickname) {
                return Err(String::from("ALREADY AN OPERATOR"));
            }
            server_operators.push(data_user.nickname);
            Ok(String::from("RPL_YOUREOPER"))
        }
        Err(_) => Err(String::from("NO se pudo leer el vector de operadores!!!")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oper_with_less_parameters() {
        let parametros = vec!["oper1".to_string()];
        if let Err(error_msg) = OperInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_NEEDMOREPARAMS".to_string());
        }
    }

    #[test]
    fn oper_with_to_much_parameters() {
        let parametros = vec![
            "oper1".to_string(),
            "pass1".to_string(),
            "oper2".to_string(),
        ];
        if let Err(error_msg) = OperInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_MOREPARAMS".to_string());
        }
    }

    #[test]
    fn create_oper_info_successfully() {
        let parametros = vec!["oper1".to_string(), "pass1".to_string()];
        if let Ok(oper) = OperInfo::new(parametros) {
            assert_eq!(oper.nick, "oper1".to_string());
            assert_eq!(oper.pass, "pass1".to_string());
        }
    }

    #[test]
    fn set_operator_a_new_operator() {
        let parametros = vec!["oper2".to_string(), "pass2".to_string()];
        if let Ok(oper) = OperInfo::new(parametros) {
            let server_operators: Arc<RwLock<Vec<String>>> =
                Arc::new(RwLock::new(vec!["oper1".to_string()]));
            let mut data_user = DataUserFile::default_for_clients();
            data_user.nickname = "oper2".to_string();
            data_user.password = "pass2".to_string();

            if let Ok(response) = set_operator(data_user, oper, &server_operators) {
                assert_eq!("RPL_YOUREOPER".to_string(), response);
            }
        }
    }

    #[test]
    fn set_operator_that_already_is_an_operator() {
        let parametros = vec!["oper1".to_string(), "pass1".to_string()];
        if let Ok(oper) = OperInfo::new(parametros) {
            let server_operators: Arc<RwLock<Vec<String>>> =
                Arc::new(RwLock::new(vec!["oper1".to_string()]));
            let mut data_user = DataUserFile::default_for_clients();
            data_user.nickname = "oper1".to_string();
            data_user.password = "pass1".to_string();

            if let Err(response) = set_operator(data_user, oper, &server_operators) {
                assert_eq!("ALREADY AN OPERATOR".to_string(), response);
            }
        }
    }

    #[test]
    fn set_operator_with_wrong_password() {
        let parametros = vec!["oper1".to_string(), "pass3".to_string()];
        if let Ok(oper) = OperInfo::new(parametros) {
            let server_operators: Arc<RwLock<Vec<String>>> =
                Arc::new(RwLock::new(vec!["oper1".to_string()]));
            let mut data_user = DataUserFile::default_for_clients();
            data_user.nickname = "oper1".to_string();
            data_user.password = "pass1".to_string();

            if let Ok(response) = set_operator(data_user, oper, &server_operators) {
                assert_eq!("ERR_PASSWDMISMATCH".to_string(), response);
            }
        }
    }

    #[test]
    fn set_operator_with_wrong_nickname() {
        let parametros = vec!["oper1".to_string(), "pass1".to_string()];
        if let Ok(oper) = OperInfo::new(parametros) {
            let server_operators: Arc<RwLock<Vec<String>>> =
                Arc::new(RwLock::new(vec!["oper1".to_string()]));
            let mut data_user = DataUserFile::default_for_clients();
            data_user.nickname = "oper2".to_string();
            data_user.password = "pass1".to_string();

            if let Ok(response) = set_operator(data_user, oper, &server_operators) {
                assert_eq!("ERR_PASSWDMISMATCH".to_string(), response);
            }
        }
    }
}
