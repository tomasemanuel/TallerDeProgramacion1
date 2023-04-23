use crate::datauser::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

//use std::net::{SocketAddr,Ipv4Addr, IpAddr};

static MAX_PARAMS_NICK: usize = 2;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NickInfo {
    pub nick: String,
}

impl NickInfo {
    /// crea un nuevo Nick Info para su uso en parser.rs
    pub fn new(parametro: Vec<String>) -> Result<NickInfo, String> {
        if parametro.is_empty() {
            return Err(String::from("ERR_NONICKNAMEGIVEN"));
        }
        if parametro.len() > MAX_PARAMS_NICK {
            return Err(String::from("ERR_TOOMANYARGUMENTS"));
        }
        Ok(NickInfo {
            nick: parametro[0].to_string(),
        })
    }
}
/// Dependiendo si se quiere cambiar, registrar, o levantar a un usuario caido. Dicha funcion
/// chequea la contraseña del usuario caido, o ve que un usuario no se registre o cambie el nickname a
/// un nickname ya registrado
pub fn set_nick_name(
    nick_info: NickInfo,
    data_user: &mut DataUserFile,
    hash_lock: &Arc<RwLock<HashMap<String, DataUserFile>>>,
    users_connected: &Arc<RwLock<Vec<String>>>,
) -> Result<String, String> {
    // el prefijo es solo para la segunda parte....
    if data_user.password == "-1" {
        return Err(String::from("ERR_NOPASSWORDINITIALIZED"));
    }

    match hash_lock.read() {
        Ok(hash_map) => {
            match users_connected.read() {
                Ok(users_connected) => {
                    if data_user.nickname != "-1" && data_user.contains_all_values() {
                        // ya esta registrado, quiero cambiar el nick
                        if hash_map.contains_key(&nick_info.nick) {
                            // nuevo nickname esta en el hash
                            return Err(String::from("ERR_NICKNAMEINUSE"));
                        }
                        let nickname_anterior = data_user.nickname.clone();
                        data_user.nickname = nickname_anterior;
                        data_user.nickname_actualizado = nick_info.nick.clone();
                        // cambiar el hash_nicknames
                        return Ok(format!("{}{}", "NICK ", nick_info.nick));
                    }

                    if hash_map.contains_key(&nick_info.nick) {
                        if !users_connected.contains(&nick_info.nick) {
                            if let Some(data_user_nick) = hash_map.get(&nick_info.nick) {
                                if data_user_nick.password != data_user.password {
                                    return Err(String::from("ERR_PASSWORDINCORRECT"));
                                }
                            }
                            if let Some(data) = hash_map.get(&nick_info.nick) {
                                data_user.clone_data_user(data);
                            }

                            return Ok(format!("{}{}", "Welcome! ", nick_info.nick));
                        } else {
                            return Err(String::from("ERR_NICKNAMEINUSE"));
                        }
                    }
                    data_user.nickname = nick_info.nick.clone();
                    data_user.nickname_actualizado = String::from("same");
                    Ok(format!("{}{}", "REGISTER! ", nick_info.nick))
                }
                Err(_) => Err(String::from("Fallo read del users_connected")),
            }
        }
        Err(_) => Err(String::from("Fallo el read del hash_lock")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::RwLock;

    #[test]
    fn create_witht_empty_vector() {
        let parametros: Vec<String> = vec![];
        let esperado = Err("ERR_NONICKNAMEGIVEN".to_string());

        assert_eq!(NickInfo::new(parametros), esperado);
    }

    #[test]
    fn new_con_muchos_parametros() {
        let parametros: Vec<String> = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let esperado = Err("ERR_TOOMANYARGUMENTS".to_string());

        assert_eq!(NickInfo::new(parametros), esperado);
    }

    #[test]
    fn successful_create() {
        let parametros: Vec<String> = vec!["nick1".to_string()];
        if let Ok(nick_info) = NickInfo::new(parametros) {
            assert_eq!(nick_info.nick, "nick1".to_string());
        }
    }

    #[test]
    fn set_nick_name_without_password() {
        let mut datauser = DataUserFile::default_for_clients();
        let nick = NickInfo {
            nick: "bob".to_string(),
        };
        let data_hash = HashMap::new();
        let hash_lock = Arc::new(RwLock::new(data_hash));
        let vec = Arc::new(RwLock::new(Vec::new()));

        let esperado = Err("ERR_NOPASSWORDINITIALIZED".to_string());
        assert_eq!(
            set_nick_name(nick, &mut datauser, &hash_lock, &vec),
            esperado
        );
    }
}
