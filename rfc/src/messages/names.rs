use std::{
    collections::HashMap,
    sync::{Arc, RwLockReadGuard},
};

use crate::{channel_list::ChannelList, data_base::DataBase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamesInfo {
    pub channel_list: Vec<String>,
}

impl NamesInfo {
    /// crea un nuevo Names Info para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> Result<NamesInfo, String> {
        if parametros.is_empty() {
            return Ok(NamesInfo {
                channel_list: parametros,
            });
        }
        let split = parametros[0].split(',');
        let channel_list: Vec<String> = split.into_iter().map(|x| x.to_string()).collect();
        Ok(NamesInfo { channel_list })
    }
}

/// devuelve la lista completa de los nombres de los usuarios en el canal o una lista reducida de los canales con sus usuarios dependiendo
/// la si la channel_list es vacia
pub fn names(
    channels_data_base: RwLockReadGuard<HashMap<String, ChannelList>>,
    joined_channels: RwLockReadGuard<HashMap<String, Vec<String>>>,
    channel_list: Vec<String>,
    data_base: Arc<DataBase>,
) -> Result<String, String> {
    if channel_list.is_empty() {
        return complete_list_of_channels(channels_data_base, joined_channels, data_base);
    };
    Ok(filter_list_of_channels(channels_data_base, channel_list))
}

/// Devuelve una lista de los canales con los usuarios unidos, y tambien una lista de los usuarios que no esta unidos a ningun canal
fn filter_list_of_channels(
    data_base: RwLockReadGuard<HashMap<String, ChannelList>>,
    channel_vector: Vec<String>,
) -> String {
    let mut answer = String::from("NAMES ");
    for channel in channel_vector.iter() {
        answer.push_str(channel);
        match data_base.get(channel) {
            Some(channel_list) => {
                answer.push_str(": ");
                let joined_list = channel_list.joined_list.clone();
                for nickname in joined_list {
                    answer.push_str(nickname.as_str());
                    answer.push(',');
                }
                answer.pop(); // saco el ultimo espacio
                answer.push_str(", ");
            }
            None => answer.push_str("ERR_NOSUCHCHANNEL"),
        }
    }
    answer.pop(); // saco el ultimo espacio
    if answer != *"NAMES" {
        answer.pop(); // saco la coma
    }
    answer
}

/// Hace una lista completa de los canales reconocidos en la red para pasarsela a la funcion
/// filter_list_of_channels que devuelve un string con los canales y sus usuarios
fn complete_list_of_channels(
    channels_data_base: RwLockReadGuard<HashMap<String, ChannelList>>,
    joined_channels: RwLockReadGuard<HashMap<String, Vec<String>>>,
    data_base: Arc<DataBase>,
) -> Result<String, String> {
    let mut channel_vector: Vec<String> = Vec::new();
    let channel_hash = channels_data_base.clone();
    for (canal, _) in channel_hash.into_iter() {
        channel_vector.push(canal);
    }
    let mut users_in_channels = filter_list_of_channels(channels_data_base, channel_vector);
    if users_in_channels != *"NAMES" {
        users_in_channels.push(',')
    }
    users_in_channels.push_str(&users_not_in_any_channel(data_base, joined_channels)?);
    Ok(users_in_channels)
}

/// Filtra a aquellos usuarios que no se encuentren en ningun canal
/// y devuele un vector con los mismos
fn users_not_in_any_channel(
    data_base: Arc<DataBase>,
    joined_channels: RwLockReadGuard<HashMap<String, Vec<String>>>,
) -> Result<String, String> {
    let mut respuesta = String::from(" &*: ");
    match data_base.data_connected_all_servers.read() {
        Ok(connected_users) => {
            for user in connected_users.clone() {
                if joined_channels.get(&user).is_none() {
                    // if user_joined_channels.is_empty() || user_joined_channels[0] == ""{
                    respuesta.push_str(&user);
                    respuesta.push(',');
                    // }
                }
            }
            respuesta.pop();
        }
        Err(_) => return Err(String::from("Error deslockeando el hash")),
    }
    Ok(respuesta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_witn_empty_parameters() {
        let parametros: Vec<String> = vec![];
        let expected: Vec<String> = vec![];

        if let Ok(answer) = NamesInfo::new(parametros) {
            assert_eq!(expected, answer.channel_list)
        }
    }

    #[test]
    fn create_with_parameters() {
        let parametros: Vec<String> = vec!["neymar,messi,jordi-alba".to_string()];
        let expected: Vec<String> = vec![
            "neymar".to_string(),
            "messi".to_string(),
            "jordi-alba".to_string(),
        ];

        if let Ok(answer) = NamesInfo::new(parametros) {
            assert_eq!(answer.channel_list, expected);
        }
    }
}
