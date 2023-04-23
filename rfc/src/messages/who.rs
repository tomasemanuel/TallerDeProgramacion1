// use crate::datauser::DataUserFile;

use std::{collections::HashMap, sync::RwLockReadGuard};

use crate::{channel_list::ChannelList, datauser::DataUserFile};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhoInfo {
    pub name: Option<String>,
    pub operator: bool,
}

impl WhoInfo {
    /// crea un nuevo WHoinfo para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> Result<WhoInfo, String> {
        match parametros.len() {
            2 => {
                if parametros[1].as_str() == "o" {
                    return Ok(WhoInfo {
                        name: Some(parametros[0].to_owned()),
                        operator: true,
                    });
                }
                Ok(WhoInfo {
                    name: Some(parametros[0].to_owned()),
                    operator: false,
                })
            }
            1 => {
                if parametros[0].as_str() == "o" {
                    return Ok(WhoInfo {
                        name: None,
                        operator: true,
                    });
                }
                Ok(WhoInfo {
                    name: Some(parametros[0].to_owned()),
                    operator: false,
                })
            }
            0 => Ok(WhoInfo {
                name: None,
                operator: false,
            }),
            _ => Err(String::from("ERR_TOOMANYARGUMENTS")),
        }
    }
}

pub fn who_function(
    data_channels: RwLockReadGuard<HashMap<String, ChannelList>>,
    data_joined_channels: RwLockReadGuard<HashMap<String, Vec<String>>>,
    who_info: WhoInfo,
    data_registered: RwLockReadGuard<HashMap<String, DataUserFile>>,
    nickname: String,
) -> String {
    match who_info.name {
        Some(_name_to_search) => names_outside_same_channel(
            data_channels,
            data_joined_channels,
            data_registered,
            nickname,
            _name_to_search,
            who_info.operator,
        ),
        None => names_outside_same_channel(
            data_channels,
            data_joined_channels,
            data_registered,
            nickname,
            String::from(""),
            who_info.operator,
        ),
    }
}

/// devuelve los nombres que se encuentran en todos los canales afuera de los canales donde se encuentra un usuario
fn names_outside_same_channel(
    data_channels: RwLockReadGuard<HashMap<String, ChannelList>>,
    data_joined_channels: RwLockReadGuard<HashMap<String, Vec<String>>>,
    data_registered: RwLockReadGuard<HashMap<String, DataUserFile>>,
    nickname: String,
    name: String,
    operator: bool,
) -> String {
    let mut names_in_same_channel: Vec<String> = Vec::new();
    if let Some(joined_channels) = data_joined_channels.get(&nickname) {
        names_in_same_channel =
            names_through_diff_channels(joined_channels.clone(), data_channels, operator);
    }
    let mut respuesta = String::from("WHO ");
    for (user, data_user_file) in data_registered.clone().into_iter() {
        if name.is_empty()
            && (!names_in_same_channel.contains(&user)
                || matches_an_atribute(data_user_file, &name))
        {
            respuesta.push_str(user.as_str());
            respuesta.push_str(", ")
        }
    }
    if respuesta.len() > 2 {
        respuesta.pop();
        respuesta.pop(); // elimino el espacio y la coma
    }
    respuesta
}
/// devuelve un bool para utilizar en el matcheo del who
fn matches_an_atribute(data_user_file: DataUserFile, name: &String) -> bool {
    let nickname: String = data_user_file.nickname;
    let username: String = data_user_file.username;
    let realname: String = data_user_file.realname;
    if nickname.contains(name) || username.contains(name) || realname.contains(name) {
        return true;
    }
    false
}

/// Devuelve un vector de los usuarios no operadores y que se encuentran en otros canales
fn names_through_diff_channels(
    joined_channels: Vec<String>,
    data_channels: RwLockReadGuard<HashMap<String, ChannelList>>,
    operator: bool,
) -> Vec<String> {
    let mut names_in_same_channel: Vec<String> = Vec::new();
    for channel_name in joined_channels {
        if let Some(channel_list) = data_channels.get(&channel_name) {
            for user in channel_list.joined_list.clone() {
                if operator && !channel_list.operators.contains(&user) {
                    continue;
                }
                if !names_in_same_channel.contains(&user) {
                    names_in_same_channel.push(user);
                }
            }
        }
    }
    names_in_same_channel
}
