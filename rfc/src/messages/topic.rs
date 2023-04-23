use std::{
    collections::HashMap,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::channel_list::ChannelList;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicInfo {
    pub channel: String,
    pub topic: Option<String>,
}

impl TopicInfo {
    /// crea un nuevo Topic Info para su uso en parser.rs
    pub fn new(parametro: Vec<String>) -> Result<TopicInfo, String> {
        match parametro.len() {
            1 => Ok(TopicInfo {
                channel: parametro[0].clone(),
                topic: None,
            }),
            0 => Err(String::from("ERR_NEEDMOREPARAMS")),
            _ => {
                let slice = &parametro[1..parametro.len()];

                let mut topic = slice.join(" ");
                if slice[0] == *":" {
                    //if slice[0] == ":".to_string(){
                    topic.remove(0);
                }
                Ok(TopicInfo {
                    channel: parametro[0].clone(),
                    topic: Some(topic),
                })
            }
        }
    }
}

/// Chequea si tiene que poner o devolver un topic de un canal
pub fn give_or_receive_topic(
    topic_info: TopicInfo,
    joined_channels: RwLockReadGuard<HashMap<String, Vec<String>>>,
    data_channels: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    nickname: &String,
) -> Result<String, String> {
    if let Some(channel_joined) = joined_channels.get(nickname) {
        if !channel_joined.contains(&topic_info.channel) {
            return Err(String::from("ERR_NOTONCHANNEL"));
        }
        if let Some(channel_list) = data_channels.clone().get(&topic_info.channel) {
            match topic_info.topic {
                Some(new_topic) => {
                    return set_new_topic(
                        new_topic,
                        data_channels,
                        channel_list,
                        &topic_info.channel,
                        nickname,
                    )
                }
                None => return Ok(return_channel_topic(channel_list)),
            }
        }
    }
    Err(String::from("ERR_NOTONCHANNEL"))
}

/// A partir de los privilegios de Operador del canal, el usuario puede setear un nuevo topic
fn set_new_topic(
    new_topic: String,
    data_channels: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    channel_list: &ChannelList,
    channel_name: &String,
    nickname: &String,
) -> Result<String, String> {
    if !channel_list.operators.contains(nickname) {
        return Err(String::from("ERR_NOTANOPERATOR"));
    }
    let channel_list = ChannelList {
        invited_list: channel_list.invited_list.clone(),
        joined_list: channel_list.joined_list.clone(),
        operators: channel_list.operators.clone(),
        topic: Some(new_topic.clone()),
        ban_mask: channel_list.ban_mask.clone(),
        secret: channel_list.secret,
        private: channel_list.private,
    };
    data_channels.remove(channel_name);
    data_channels
        .entry(channel_name.clone())
        .or_insert(channel_list);
    Ok(format!("{}{}", "RPL_TOPIC ", new_topic))
}

/// Devuelve el topic del canal actual
fn return_channel_topic(channel_list: &ChannelList) -> String {
    match channel_list.topic.clone() {
        Some(current_topic) => format!("{}{}", "RPL_TOPIC ", current_topic),
        None => String::from("RPL_NOTOPIC"),
    }
}
