// use std::collections::HashMap;
use crate::datauser::DataUserFile;
// use crate::parser::MessageCommand;

static MAX_PASS_ARGS: usize = 1;
const VACIO: &str = "-1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassInfo {
    pub pass: String,
}

impl PassInfo {
    /// crea un nuevo Pass Info para su uso en parser.rs
    pub fn new(parametro: Vec<String>) -> Result<PassInfo, String> {
        if parametro.len() > MAX_PASS_ARGS {
            return Err(String::from("ERR_TOOMANYARGUMENTS"));
        }
        Ok(PassInfo {
            pass: parametro[0].to_string(),
        })
    }
}
/// setea una nueva contraseña en el data user_file para su uso posterior
pub fn set_password(pass_info: PassInfo, data_user: &mut DataUserFile) -> Result<(), String> {
    //Si tiene un nick o un username NO puede poner la password!!
    match (data_user.username.as_str(), data_user.nickname.as_str()) {
        (VACIO, VACIO) => data_user.password = pass_info.pass,
        _ => return Err(String::from("ERR_ALREADYREGISTRED")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_pass_with_to_much_parameters() {
        let parametros = vec!["pass1".to_string(), "pass2".to_string()];
        if let Err(error_msg) = PassInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_TOOMANYARGUMENTS".to_string());
        }
    }

    #[test]
    fn create_pass_successfully() {
        let parametros = vec!["pass1".to_string()];
        if let Ok(pass_info) = PassInfo::new(parametros) {
            assert_eq!(pass_info.pass, "pass1".to_string());
        }
    }

    #[test]
    fn set_password_successfully() {
        let parametros = vec!["pass1".to_string()];

        let mut data_user = DataUserFile::default_for_clients();

        if let Ok(pass_info) = PassInfo::new(parametros) {
            if let Ok(_) = set_password(pass_info, &mut data_user) {
                assert_eq!(data_user.password, "pass1".to_string());
            }
        };
    }

    #[test]
    fn set_password_having_username_and_nickname() {
        let parametros = vec!["pass1".to_string()];

        let mut data_user = DataUserFile::default_for_clients();
        data_user.nickname = "nick1".to_string();
        data_user.username = "user1".to_string();

        if let Ok(pass_info) = PassInfo::new(parametros) {
            if let Err(err) = set_password(pass_info, &mut data_user) {
                assert_eq!(err, "ERR_ALREADYREGISTRED".to_string());
            }
        };
    }

    #[test]
    fn set_password_having_nickname() {
        let parametros = vec!["pass1".to_string()];

        let mut data_user = DataUserFile::default_for_clients();
        data_user.nickname = "nick1".to_string();

        if let Ok(pass_info) = PassInfo::new(parametros) {
            if let Err(err) = set_password(pass_info, &mut data_user) {
                assert_eq!(err, "ERR_ALREADYREGISTRED".to_string());
            }
        };
    }

    #[test]
    fn set_password_having_username() {
        let parametros = vec!["pass1".to_string()];

        let mut data_user = DataUserFile::default_for_clients();
        data_user.username = "user1".to_string();

        if let Ok(pass_info) = PassInfo::new(parametros) {
            if let Err(err) = set_password(pass_info, &mut data_user) {
                assert_eq!(err, "ERR_ALREADYREGISTRED".to_string());
            }
        };
    }
}
