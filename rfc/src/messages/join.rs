use std::{
    collections::HashMap,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::channel_list::ChannelList;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinInfo {
    pub channel_list: Vec<String>,
    pub channel_key: Option<Vec<String>>,
}

impl JoinInfo {
    /// crea un nuevo Join Info para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> Result<JoinInfo, String> {
        if parametros.is_empty() {
            return Err(String::from("ERR_NEEDMOREPARAMS"));
        }
        let split = parametros[0].split(',');
        let channel_list: Vec<String> = split.into_iter().map(|x| x.to_string()).collect();
        let channel_key: Vec<String>;
        match parametros.len() {
            2 => {
                let split = parametros[1].split(',');
                channel_key = split.into_iter().map(|x| x.to_string()).collect();
                for canal in channel_list.iter() {
                    let char_vec: Vec<char> = canal.chars().collect();
                    let ch = char_vec[0];
                    if ch != '&' && ch != '#' {
                        return Err(String::from("ERR_NOTPUBLICNORINVITE"));
                    }
                }
                if channel_key.len() > channel_list.len() {
                    return Err(String::from("More keys than channels"));
                }
                Ok(JoinInfo {
                    channel_list,
                    channel_key: Some(channel_key),
                })
            }
            1 => {
                for canal in channel_list.iter() {
                    let char_vec: Vec<char> = canal.chars().collect();
                    let ch = char_vec[0];
                    if ch != '&' && ch != '#' {
                        return Err(String::from("ERR_NOTPUBLICNORINVITE"));
                    };
                }
                Ok(JoinInfo {
                    channel_list,
                    channel_key: None,
                })
            }
            _ => Err(String::from("ERR_TOOMANYPARAMS")),
        }
    }
}

/// funcion que chequea los canales, dependiendo si existe o no, esta funcion devuelve un result indicando si se
/// pudo unir a dicho canal
pub fn check_channels(
    join_info: JoinInfo,
    data_base: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    joined_channels: &mut RwLockWriteGuard<HashMap<String, Vec<String>>>,
    nickname: String,
    operators: RwLockReadGuard<Vec<String>>,
) -> Result<String, String> {
    let base_datos_clonada = data_base.clone();
    let mut respuesta: String = String::from("");
    for channel_name in join_info.channel_list {
        //let data_user_nick = nickname.clone();
        // let data_user_nick = nickname.clone();
        match base_datos_clonada.get(&channel_name) {
            None => {
                let respuesta_canal =
                    create_new_channel(channel_name.clone(), data_base, &nickname);
                respuesta.push_str(&respuesta_canal);
            }
            Some(channel_list) => {
                let mut channel_list_clone = channel_list.clone();
                let respuesta_canal: String = join_existing_channel(
                    channel_name.clone(),
                    data_base,
                    &mut channel_list_clone,
                    &nickname,
                    operators.clone(),
                )?;
                respuesta.push_str(&respuesta_canal);
            }
        }
        respuesta.push_str(", ");
        match joined_channels.remove(&nickname) {
            Some(mut joined_existing_channels) => {
                joined_existing_channels.push(channel_name.clone());
                joined_channels
                    .entry(nickname.clone())
                    .or_insert(joined_existing_channels);
            }
            None => {
                let joined_existing_channels: Vec<String> = vec![channel_name];
                joined_channels
                    .entry(nickname.clone())
                    .or_insert(joined_existing_channels);
            }
        }
    }
    respuesta.pop(); // saco el ultimo espacio
    respuesta.pop(); // saco la coma
    Ok(respuesta)
}

/// Se llama a esta funcion si ya existe el canal, se unira al canal solo si esta funcion se lo permite, sino devuelve un error
fn join_existing_channel(
    channel_name: String,
    data_base: &mut HashMap<String, ChannelList>,
    channel_list: &mut ChannelList,
    data_user_nick: &String,
    operators: Vec<String>,
) -> Result<String, String> {
    if !public_channel(channel_name.clone())
        && !user_in_invited_list(channel_list.clone(), data_user_nick.clone())
        && !operators.contains(data_user_nick)
    {
        return Err(String::from("ERR_INVITEONLYCHAN"));
    }
    if let Some(ban_mask) = channel_list.ban_mask.clone() {
        for mask in ban_mask {
            if data_user_nick.contains(&mask) {
                return Err(String::from("ERR_BANNEDFROMCHAN"));
            }
        }
    }
    let joined_users = &channel_list.joined_list.clone();
    if joined_users.contains(data_user_nick) {
        return Err(String::from("ERR_ALREADYINCHANNEL"));
    }
    data_base.remove(&channel_name);
    let data_user_nick = data_user_nick.clone();
    channel_list.add_nickname(data_user_nick.clone());
    let new_channel_list =
        new_channel_list(channel_name.clone(), channel_list.clone(), &data_user_nick);
    let channel_name_clone = channel_name.clone();
    data_base
        .entry(channel_name.clone())
        .or_insert(new_channel_list);
    let mut string_principal = String::from("CHANNELJOINED ");
    string_principal.push_str(&channel_name_clone);
    string_principal.push_str(" USER: ");
    for user in joined_users {
        string_principal.push_str(user);
        string_principal.push(',')
    }
    string_principal.pop();
    string_principal.pop();
    Ok(string_principal)
}

/// devuelve un bool si el usuario se encuentra en la lista de invitados, se usa en la funcion join_existing_channel
fn user_in_invited_list(channel_list: ChannelList, data_user_nick: String) -> bool {
    if let Some(channel_invited_list) = channel_list.invited_list {
        return channel_invited_list.contains(&data_user_nick);
    }
    false
}

/// devuelve un bool indicando si el canal es publico o no
fn public_channel(channel_name: String) -> bool {
    let char_vec: Vec<char> = channel_name.chars().collect();
    let ch = char_vec[0];
    ch == '&'
}
/// se crea un nuevo canal cuando este no existe en la base de datos, con el usuario de operador
fn create_new_channel(
    channel_name: String,
    data_base: &mut HashMap<String, ChannelList>,
    nickname: &str,
) -> String {
    let nickname = nickname.to_owned();
    let char_vec: Vec<char> = channel_name.chars().collect();
    let invited = char_vec[0] == '#';
    let channel_list = ChannelList::new(nickname, invited);
    data_base
        .entry(channel_name.clone())
        .or_insert(channel_list);
    format!("{}{}", "CHANNELJOINED ", channel_name)
}
/// crea una nueva channel_list para simplificar un poco el codigo, se la llama en join_existing_channel, sacando al usuario de la lista de invitados
fn new_channel_list(
    channel_name: String,
    channel_list: ChannelList,
    nickname: &String,
) -> ChannelList {
    let new_channel_list;
    if !public_channel(channel_name) {
        if let Some(mut invited_list) = channel_list.invited_list.clone() {
            invited_list.retain(|users| users != nickname);
            new_channel_list = ChannelList {
                joined_list: channel_list.joined_list.clone(),
                invited_list: Some(invited_list),
                operators: channel_list.operators.clone(),
                topic: channel_list.topic.clone(),
                ban_mask: channel_list.ban_mask.clone(),
                secret: channel_list.secret,
                private: channel_list.private,
            };
        } else {
            new_channel_list = channel_list;
        }
    } else {
        new_channel_list = channel_list;
    }
    new_channel_list
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn join_with_less_parameters() {
        let parametros = vec![];
        if let Err(error_msg) = JoinInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_NEEDMOREPARAMS".to_string());
        }
    }

    #[test]
    fn create_with_no_channel_format() {
        let parametros = vec!["canal1".to_string()];
        if let Err(error_msg) = JoinInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_NOTPUBLICNORINVITE".to_string());
        }

        let parametros = vec!["canal1, &canal2".to_string()];
        if let Err(error_msg) = JoinInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_NOTPUBLICNORINVITE".to_string());
        }

        let parametros = vec!["canal1, canal2, #canal3".to_string()];
        if let Err(error_msg) = JoinInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_NOTPUBLICNORINVITE".to_string());
        }
    }

    #[test]
    fn create_with_list_of_channels() {
        let parametros = vec!["&canal1, &canal2, &canal3, &canal4, &canal5".to_string()];
        if let Ok(join) = JoinInfo::new(parametros) {
            assert_eq!(join.channel_list[0], "&canal1".to_string());
            assert_eq!(join.channel_list[0], "&canal2".to_string());
            assert_eq!(join.channel_list[0], "&canal3".to_string());
            assert_eq!(join.channel_list[0], "&canal4".to_string());
            assert_eq!(join.channel_list[0], "&canal5".to_string());
        }
    }

    #[test]
    fn create_with_list_of_channels_and_keys() {
        let parametros = vec![
            "#canal1,#canal2,#canal3,#canal4,#canal5".to_string(),
            "contra1,contra2,contra3,contra4,contra5".to_string(),
        ];

        if let Ok(join) = JoinInfo::new(parametros) {
            assert_eq!(join.channel_list[0], "#canal1".to_string());
            assert_eq!(join.channel_list[1], "#canal2".to_string());
            assert_eq!(join.channel_list[2], "#canal3".to_string());
            assert_eq!(join.channel_list[3], "#canal4".to_string());
            assert_eq!(join.channel_list[4], "#canal5".to_string());

            if let Some(key) = join.channel_key {
                assert_eq!(key[0], "contra1".to_string());
                assert_eq!(key[1], "contra2".to_string());
                assert_eq!(key[2], "contra3".to_string());
                assert_eq!(key[3], "contra4".to_string());
                assert_eq!(key[4], "contra5".to_string());
            }
        }
    }

    #[test]
    fn create_with_more_keys_than_channels() {
        let parametros = vec![
            "#canal1,#canal2,#canal3,#canal4,#canal5".to_string(),
            "contra1,contra2,contra3,contra4,contra5,contra6".to_string(),
        ];
        if let Err(e) = JoinInfo::new(parametros) {
            assert_eq!(e, "More keys than channels".to_string());
        }
    }
}
