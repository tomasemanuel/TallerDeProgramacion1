use std::{collections::HashMap, sync::RwLockReadGuard};

use crate::channel_list::ChannelList;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListInfo {
    pub channel_list: Option<Vec<String>>,
}

impl ListInfo {
    /// crea un nuevo List Info para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> Result<ListInfo, String> {
        if parametros.is_empty() {
            return Ok(ListInfo { channel_list: None });
        }
        let split = parametros[0].split(',');
        let channel_list: Vec<String> = split.into_iter().map(|x| x.to_string()).collect();
        Ok(ListInfo {
            channel_list: Some(channel_list),
        })
    }
}
/// funcion que se llama en channels.rs, llama a sus propias funciones dependiendo si se quiere la lista
/// completa de canales con sus topics o solo una lista limitada de los canales
pub fn list_of_channels(
    data_base: RwLockReadGuard<HashMap<String, ChannelList>>,
    channel_list: Option<Vec<String>>,
    nickname: &String,
) -> Result<String, String> {
    match channel_list {
        Some(channel_list) => Ok(filter_list_of_channels(data_base, channel_list, nickname)),
        None => Ok(complete_list_of_channels(data_base, nickname)),
    }
}
/// a partir de un vector de canales, devuelve una lista de dichos canales con sus topics, tomando en
/// cuenta si el canal es privado, secreto o publico
fn filter_list_of_channels(
    data_base: RwLockReadGuard<HashMap<String, ChannelList>>,
    channel_vector: Vec<String>,
    nickname: &String,
) -> String {
    let mut answer = String::from("LIST ");
    for channel in channel_vector.iter() {
        match data_base.get(channel) {
            Some(channel_list) => {
                if !channel_list.secret || channel_list.joined_list.contains(nickname) {
                    answer.push_str(channel);
                    answer.push(':');
                    if !channel_list.private || channel_list.joined_list.contains(nickname) {
                        if let Some(topic) = channel_list.topic.clone() {
                            answer.push_str(topic.as_str());
                        }
                    } else {
                        answer.push_str("Prv");
                    }
                    answer.push(',');
                }
            }
            None => answer.push_str(": ERR_NOSUCHCHANNEL, "),
        }
    }
    answer.pop(); // saco la coma
    answer
}
/// Arma una lista completa de los canales disponibles en la red para despues llamar
/// a la funcion de filter list of channels con esa lista de canales
fn complete_list_of_channels(
    data_base: RwLockReadGuard<HashMap<String, ChannelList>>,
    nickname: &String,
) -> String {
    let mut channel_vector: Vec<String> = Vec::new();
    let channel_hash = data_base.clone();
    for (canal, _) in channel_hash.into_iter() {
        channel_vector.push(canal);
    }
    filter_list_of_channels(data_base, channel_vector, nickname)
}

#[cfg(test)]
mod tests {
    use std::sync::RwLock;

    use super::*;
    #[test]
    fn list_with_no_parameter() {
        let parametros = vec![];
        if let Ok(list_info) = ListInfo::new(parametros) {
            assert_eq!(list_info.channel_list, None);
        }
    }

    #[test]
    fn create_list_info_with_two_channels() {
        let parametros = vec!["&canal_1,&canal_2".to_string()];

        if let Ok(list_info) = ListInfo::new(parametros) {
            if let Some(channel_list) = list_info.channel_list {
                assert_eq!(channel_list[0], String::from("&canal_1"));
                assert_eq!(channel_list[1], String::from("&canal_2"));
            }
        }
    }

    #[test]
    fn creates_correct_list_of_one_channel_with_topic() {
        let parametros = vec!["&canal_1".to_string()];
        let list_info = ListInfo::new(parametros).unwrap();
        let nickname = "Shakira".to_string();
        let data_base: RwLock<HashMap<String, ChannelList>> = RwLock::new(HashMap::new());
        let joined = vec![nickname.clone()];
        let channel_list = ChannelList {
            invited_list: None,
            joined_list: joined.clone(),
            operators: joined,
            topic: Some(String::from("esta es la nueva topic")),
            ban_mask: None,
            secret: false,
            private: false,
        };
        if let Ok(mut data_base) = data_base.write() {
            data_base
                .entry(String::from("&canal_1"))
                .or_insert(channel_list);
        };
        if let Ok(data_base) = data_base.read() {
            let respuesta =
                filter_list_of_channels(data_base, list_info.channel_list.unwrap(), &nickname);
            assert_eq!(
                respuesta,
                String::from("LIST &canal_1:esta es la nueva topic")
            );
        };
    }

    #[test]
    fn create_correct_list_two_channels_with_topic() {
        let parametros = vec!["&canal_1,&canal_2".to_string()];
        let list_info = ListInfo::new(parametros).unwrap();
        let nickname = "Shakira".to_string();
        let data_base: RwLock<HashMap<String, ChannelList>> = RwLock::new(HashMap::new());
        let joined = vec![nickname.clone()];
        let channel_list = ChannelList {
            invited_list: None,
            joined_list: joined.clone(),
            operators: joined.clone(),
            topic: Some(String::from("esta es la nueva topic")),
            ban_mask: None,
            secret: false,
            private: false,
        };
        let channel_list_2 = ChannelList {
            invited_list: None,
            joined_list: joined.clone(),
            operators: joined,
            topic: Some(String::from("esta es la segunda topic")),
            ban_mask: None,
            secret: false,
            private: false,
        };
        if let Ok(mut data_base) = data_base.write() {
            data_base
                .entry(String::from("&canal_1"))
                .or_insert(channel_list);
            data_base
                .entry(String::from("&canal_2"))
                .or_insert(channel_list_2);
        };
        if let Ok(data_base) = data_base.read() {
            let respuesta =
                filter_list_of_channels(data_base, list_info.channel_list.unwrap(), &nickname);
            assert_eq!(
                respuesta,
                String::from(
                    "LIST &canal_1:esta es la nueva topic,&canal_2:esta es la segunda topic"
                )
            );
        };
    }

    #[test]
    fn list_with_private_channel() {
        let parametros = vec!["&canal_1".to_string()];
        let list_info = ListInfo::new(parametros).unwrap();
        let nickname_en_canal = "Shakira".to_string();
        let nickname_no_en_canal = "Pique".to_string();
        let data_base: RwLock<HashMap<String, ChannelList>> = RwLock::new(HashMap::new());
        let joined = vec![nickname_en_canal.clone()];
        let channel_list = ChannelList {
            invited_list: None,
            joined_list: joined.clone(),
            operators: joined.clone(),
            topic: Some(String::from("NO se va a mostrar este topic")),
            ban_mask: None,
            secret: false,
            private: true,
        };
        if let Ok(mut data_base) = data_base.write() {
            data_base
                .entry(String::from("&canal_1"))
                .or_insert(channel_list);
        };
        if let Ok(data_base) = data_base.read() {
            let respuesta = filter_list_of_channels(
                data_base,
                list_info.channel_list.unwrap(),
                &nickname_no_en_canal,
            );
            assert_eq!(respuesta, String::from("LIST &canal_1:Prv"));
        };
    }

    #[test]
    fn list_with_private_but_user_in_channel() {
        let parametros = vec!["&canal_1".to_string()];
        let list_info = ListInfo::new(parametros).unwrap();
        let nickname_en_canal = "Shakira".to_string();
        let data_base: RwLock<HashMap<String, ChannelList>> = RwLock::new(HashMap::new());
        let joined = vec![nickname_en_canal.clone()];
        let channel_list = ChannelList {
            invited_list: None,
            joined_list: joined.clone(),
            operators: joined.clone(),
            topic: Some(String::from("se va a mostrar este topic")),
            ban_mask: None,
            secret: false,
            private: true,
        };
        if let Ok(mut data_base) = data_base.write() {
            data_base
                .entry(String::from("&canal_1"))
                .or_insert(channel_list);
        };
        if let Ok(data_base) = data_base.read() {
            let respuesta = filter_list_of_channels(
                data_base,
                list_info.channel_list.unwrap(),
                &nickname_en_canal,
            );
            assert_eq!(
                respuesta,
                String::from("LIST &canal_1:se va a mostrar este topic")
            );
        };
    }

    #[test]
    fn list_with_secret_channel() {
        let parametros = vec!["&canal_1".to_string()];
        let list_info = ListInfo::new(parametros).unwrap();
        let nickname_en_canal = "Shakira".to_string();
        let nickname_no_en_canal = "Pique".to_string();
        let data_base: RwLock<HashMap<String, ChannelList>> = RwLock::new(HashMap::new());
        let joined = vec![nickname_en_canal.clone()];
        let channel_list = ChannelList {
            invited_list: None,
            joined_list: joined.clone(),
            operators: joined.clone(),
            topic: Some(String::from("No se va a mostrar este topic")),
            ban_mask: None,
            secret: true,
            private: false,
        };
        if let Ok(mut data_base) = data_base.write() {
            data_base
                .entry(String::from("&canal_1"))
                .or_insert(channel_list);
        };
        if let Ok(data_base) = data_base.read() {
            let respuesta = filter_list_of_channels(
                data_base,
                list_info.channel_list.unwrap(),
                &nickname_no_en_canal,
            );
            assert_eq!(respuesta, String::from("LIST"));
        };
    }

    #[test]
    fn list_with_secret_but_in_channel() {
        let parametros = vec!["&canal_1".to_string()];
        let list_info = ListInfo::new(parametros).unwrap();
        let nickname_en_canal = "Shakira".to_string();
        let data_base: RwLock<HashMap<String, ChannelList>> = RwLock::new(HashMap::new());
        let joined = vec![nickname_en_canal.clone()];
        let channel_list = ChannelList {
            invited_list: None,
            joined_list: joined.clone(),
            operators: joined.clone(),
            topic: Some(String::from("se va a mostrar este topic")),
            ban_mask: None,
            secret: true,
            private: false,
        };
        if let Ok(mut data_base) = data_base.write() {
            data_base
                .entry(String::from("&canal_1"))
                .or_insert(channel_list);
        };
        if let Ok(data_base) = data_base.read() {
            let respuesta = filter_list_of_channels(
                data_base,
                list_info.channel_list.unwrap(),
                &nickname_en_canal,
            );
            assert_eq!(
                respuesta,
                String::from("LIST &canal_1:se va a mostrar este topic")
            );
        };
    }
}
