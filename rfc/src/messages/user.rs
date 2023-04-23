static USER_ARGS: usize = 2;
use crate::{config_file::ConfigFile, datauser::DataUserFile};
static DEFAULT_REALNAME: &str = "Default realname";
static DEFAULT_SERVERNAME: &str = "Default servername";
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UserInfo {
    pub user: String,
    pub host: String,
    pub servername: String,
    pub realname: String,
}

impl UserInfo {
    /// crea un nuevo UserInfo para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> Result<UserInfo, String> {
        let len = parametros.len();
        if len < USER_ARGS {
            return Err(String::from("ERR_NEEDMOREPARAMS"));
        }
        Ok(UserInfo {
            user: parametros[0].clone(),
            host: parametros[1].clone(),
            servername: DEFAULT_SERVERNAME.to_string(),
            realname: DEFAULT_REALNAME.to_string(),
        })
    }
}

/// Setea el user a un usuario
pub fn set_user(user_info: UserInfo, data_user: &mut DataUserFile) -> Result<(), String> {
    if data_user.username != "-1" {
        return Err(String::from("ERR_ALREADYREGISTRED"));
    }

    if let Ok(configf) = ConfigFile::new("./src/config_file".to_string()) {
        if configf.server_type == "MAIN" {
            data_user.servername = configf.main_port
        } else {
            data_user.servername = configf.secondary_port
        }
    }
    data_user.username = user_info.user;
    data_user.hostname = user_info.host; // podes tener varios usuarios con el mismo user
    data_user.realname = user_info.realname;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_with_empty_vec() {
        let parametros: Vec<String> = vec![];

        if let Err(err) = UserInfo::new(parametros) {
            assert_eq!(err, "ERR_NEEDMOREPARAMS".to_string());
        }
    }

    #[test]
    fn create_user_successfully() {
        let parametros: Vec<String> = vec!["user1".to_string(), "host1".to_string()];

        if let Ok(user_info) = UserInfo::new(parametros) {
            assert_eq!(user_info.user, "user1".to_string());
            assert_eq!(user_info.host, "host1".to_string());
            assert_eq!(user_info.servername, DEFAULT_SERVERNAME.to_string());
            assert_eq!(user_info.realname, DEFAULT_REALNAME.to_string());
        }
    }

    #[test]
    fn set_user_successfully() {
        let parametros: Vec<String> = vec!["user1".to_string(), "host1".to_string()];

        if let Ok(user_info) = UserInfo::new(parametros) {
            let mut data_user = DataUserFile::default_for_clients();
            if let Ok(_) = set_user(user_info, &mut data_user) {
                assert_eq!(data_user.username, "user1".to_string());
                assert_eq!(data_user.hostname, "host1".to_string());
            }
        }
    }
    #[test]
    fn set_user_without_nickname() {
        let parametros: Vec<String> = vec!["user1".to_string(), "host1".to_string()];

        if let Ok(user_info) = UserInfo::new(parametros) {
            let mut data_user = DataUserFile::default_for_clients();
            if let Err(err) = set_user(user_info, &mut data_user) {
                assert_eq!(err, "INITIALIZE NICKNAME FIRST".to_string());
            }
        }
    }

    #[test]
    fn set_user_already_registered() {
        let parametros: Vec<String> = vec!["user1".to_string(), "host1".to_string()];

        if let Ok(user_info) = UserInfo::new(parametros) {
            let mut data_user = DataUserFile::default_for_clients();
            data_user.username = "user1".to_string();
            if let Err(err) = set_user(user_info, &mut data_user) {
                assert_eq!(err, "ERR_ALREADYREGISTRED".to_string());
            }
        }
    }
}
