use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

use crate::{
    channel_list::ChannelList,
    server::{spread_command_neighbors, Server},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeInfo {
    pub channel: Option<String>,
    pub nick: Option<String>,
    pub flag: String,
    pub limit: Option<String>,
    pub user: Option<String>,
    pub ban_mask: Option<String>,
    pub set: bool,
}

impl ModeInfo {
    /// crea un nuevo Mode Info para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> Result<ModeInfo, String> {
        if parametros[0].as_str().starts_with('&') || parametros[0].as_str().starts_with('#') {
            return initialize_with_channel(parametros);
        }
        Err(String::from(
            "No esta disponible el mode info para usuarios",
        ))
        // return initialize_with_nick(parametros);
    }
}

/// Dependiendo el flag que se le manda, inicializa los distintos mode disponibles.
fn initialize_with_channel(parametros: Vec<String>) -> Result<ModeInfo, String> {
    if parametros.len() < 2 {
        return Err(String::from("ERR_NEEDMOREPARAMS"));
    }
    match parametros[1].as_str() {
        "+o" => give_take_operator_priv(parametros, true),
        "-o" => give_take_operator_priv(parametros, false),
        "+b" => initialize_ban_mask(parametros, true),
        "-b" => initialize_ban_mask(parametros, false),
        "+i" => initialize_type_mode_channel(parametros, true),
        "-i" => initialize_type_mode_channel(parametros, false),
        "+s" => initialize_type_mode_channel(parametros, true),
        "-s" => initialize_type_mode_channel(parametros, false),
        "+p" => initialize_type_mode_channel(parametros, true),
        "-p" => initialize_type_mode_channel(parametros, false),
        _ => Err(String::from("ERR_UNKNOWNMODE")),
    }
}
/// inicializa el Mode info dependiendo si se quiere o no dar los privilegios de operador
fn give_take_operator_priv(parametros: Vec<String>, set_op: bool) -> Result<ModeInfo, String> {
    if parametros.len() < 3 {
        return Err(String::from("ERR_NEEDMOREPARAMS"));
    }
    Ok(ModeInfo {
        channel: Some(parametros[0].clone()),
        nick: Some(parametros[2].clone()),
        flag: "o".to_string(),
        limit: None,
        user: None,
        ban_mask: None,
        set: set_op,
    })
}

/// inicialiaza el mode dependiendo si se quiere banear, sacar el ban o la lista completa de ban
fn initialize_ban_mask(parametros: Vec<String>, set_op: bool) -> Result<ModeInfo, String> {
    if parametros.len() == 2 {
        return Ok(ModeInfo {
            channel: Some(parametros[0].clone()),
            nick: None,
            flag: "b".to_string(),
            limit: None,
            user: None,
            ban_mask: None,
            set: set_op,
        });
    }
    Ok(ModeInfo {
        channel: Some(parametros[0].clone()),
        nick: None,
        flag: "b".to_string(),
        limit: None,
        user: None,
        ban_mask: Some(parametros[2].clone()),
        set: set_op,
    })
}
/// inicializa el tipo de mode del canal, dependiendo si se quiere cambiar a secret,private,invite o public
fn initialize_type_mode_channel(parametros: Vec<String>, set: bool) -> Result<ModeInfo, String> {
    if parametros.len() > 2 {
        return Err(String::from("NOMOREPARAMETERS"));
    }
    let flag = parametros[1].clone().remove(1);
    Ok(ModeInfo {
        channel: Some(parametros[0].clone()),
        nick: None,
        flag: flag.to_string(),
        limit: None,
        user: None,
        ban_mask: None,
        set,
    })
}
/// setea a un nickname como operador, devolviendo un error si no se puede setearlo.
pub fn set_operator_on_channel(
    data_channels: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    mode_info: ModeInfo,
    nickname: &String,
) -> Result<(), String> {
    if let Some(channel) = mode_info.channel {
        if let Some(name) = mode_info.nick {
            if let Some(channel_list) = data_channels.get(&channel) {
                if channel_list.operators.contains(nickname) {
                    if channel_list.joined_list.contains(&name) {
                        let mut operators = channel_list.operators.clone();
                        if mode_info.set && !operators.contains(&name) {
                            operators.push(name);
                        } else if !mode_info.set {
                            operators.retain(|oper_nicknames| *oper_nicknames != name);
                        }
                        let channel_list_clone = ChannelList {
                            invited_list: channel_list.invited_list.clone(),
                            joined_list: channel_list.joined_list.clone(),
                            operators,
                            topic: channel_list.topic.clone(),
                            ban_mask: channel_list.ban_mask.clone(),
                            secret: channel_list.secret,
                            private: channel_list.private,
                        };
                        data_channels.remove(&channel);
                        data_channels.entry(channel).or_insert(channel_list_clone);
                        return Ok(());
                    }
                    return Err(String::from("NICKNOTONCHANNEL"));
                }
                return Err(String::from("NOTOPERATOR"));
            }
            return Err(String::from("NOSUCHCHANEL"));
        }
        return Err(String::from("NOCHANEL"));
    }
    Err(String::from("NOCHANEL"))
}

/// añade a la lista de ban o devuelve la lista completa dependiendo el nick que se le manda en el modeinfo.
pub fn set_ban_mask(
    data_channels: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    mode_info: ModeInfo,
    nickname: &String,
) -> Result<String, String> {
    if let Some(channel) = mode_info.channel {
        if let Some(channel_list) = data_channels.clone().get(&channel) {
            if channel_list.operators.contains(nickname) {
                if let Some(ban_mask) = mode_info.ban_mask {
                    return set_new_ban_mask(
                        ban_mask,
                        data_channels,
                        channel,
                        channel_list.clone(),
                        mode_info.set,
                    );
                }
                return Ok(return_the_ban_mask(channel_list.clone()));
            }
            return Err(String::from("ERR_NOTOPERATOR"));
        }
        return Err(String::from("ERR_NOSUCHCHANEL"));
    }
    Err(String::from("ERR_NOCHANEL"))
}
/// Añade a un usuario en la lista de ban de un canal
fn set_new_ban_mask(
    ban_mask: String,
    data_channels: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    channel: String,
    channel_list: ChannelList,
    set_op: bool,
) -> Result<String, String> {
    if let Some(mut previous_ban_mask) = channel_list.ban_mask {
        if set_op {
            if previous_ban_mask.contains(&ban_mask) {
                return Ok(String::from("RPLBANLIST"));
            }
            previous_ban_mask.push(ban_mask);
        } else {
            if !previous_ban_mask.contains(&ban_mask) {
                return Ok(String::from("RPLBANLIST"));
            }
            previous_ban_mask.retain(|mask| mask != &ban_mask);
        }
        let new_channel_list = ChannelList {
            invited_list: channel_list.invited_list.clone(),
            joined_list: channel_list.joined_list.clone(),
            operators: channel_list.operators.clone(),
            topic: channel_list.topic.clone(),
            ban_mask: Some(previous_ban_mask),
            secret: channel_list.secret,
            private: channel_list.private,
        };
        data_channels.remove(&channel);
        data_channels.entry(channel).or_insert(new_channel_list);
        return Ok(String::from("RPLBANLIST"));
    }
    let new_ban_mask = vec![ban_mask];
    let new_channel_list = ChannelList {
        invited_list: channel_list.invited_list.clone(),
        joined_list: channel_list.joined_list.clone(),
        operators: channel_list.operators.clone(),
        topic: channel_list.topic.clone(),
        ban_mask: Some(new_ban_mask),
        secret: channel_list.secret,
        private: channel_list.private,
    };
    data_channels.remove(&channel);
    data_channels.entry(channel).or_insert(new_channel_list);
    Ok(String::from("RPLBANLIST"))
}

/// Devuelve la lista completa de usuarios baneados del canal
fn return_the_ban_mask(channel_list: ChannelList) -> String {
    if let Some(ban_mask) = channel_list.ban_mask {
        let mut answer = String::from("BAN ");
        for mask in ban_mask {
            answer.push_str(mask.as_str());
            answer.push(',')
        }
        answer.pop();
        return answer;
    }
    String::from("")
}

pub fn set_invite_channel(
    data_channels: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    mode_info: ModeInfo,
    nickname: &String,
) -> Result<(), String> {
    set_type_channel(data_channels, mode_info, nickname, false, false)
}

pub fn set_secret_channel(
    data_channels: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    mode_info: ModeInfo,
    nickname: &String,
) -> Result<(), String> {
    set_type_channel(data_channels, mode_info, nickname, true, false)
}

pub fn set_private_channel(
    data_channels: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    mode_info: ModeInfo,
    nickname: &String,
) -> Result<(), String> {
    set_type_channel(data_channels, mode_info, nickname, false, true)
}

pub fn set_type_channel(
    data_channels: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    mode_info: ModeInfo,
    nickname: &String,
    secret: bool,
    private: bool,
) -> Result<(), String> {
    if let Some(channel_name) = mode_info.channel {
        if let Some(channel_list) = data_channels.get(&channel_name) {
            if channel_list.operators.contains(nickname) {
                let previous_channel_name_with_first = channel_name.clone();
                let new_channel_list;

                if private {
                    new_channel_list = ChannelList {
                        invited_list: Some(Vec::new()),
                        joined_list: channel_list.joined_list.clone(),
                        operators: channel_list.operators.clone(),
                        topic: channel_list.topic.clone(),
                        ban_mask: channel_list.ban_mask.clone(),
                        secret: channel_list.secret,
                        private: mode_info.set,
                    };
                } else if secret {
                    new_channel_list = ChannelList {
                        invited_list: Some(Vec::new()),
                        joined_list: channel_list.joined_list.clone(),
                        operators: channel_list.operators.clone(),
                        topic: channel_list.topic.clone(),
                        ban_mask: channel_list.ban_mask.clone(),
                        secret: mode_info.set,
                        private: channel_list.private,
                    };
                } else {
                    if mode_info.set {
                        return change_to_invite_or_public(
                            channel_name.clone(),
                            channel_list.clone(),
                            previous_channel_name_with_first,
                            data_channels,
                            '&',
                            '#',
                        );
                    }
                    return change_to_invite_or_public(
                        channel_name.clone(),
                        channel_list.clone(),
                        previous_channel_name_with_first,
                        data_channels,
                        '#',
                        '&',
                    );
                }
                data_channels.remove(&previous_channel_name_with_first);
                data_channels
                    .entry(channel_name)
                    .or_insert(new_channel_list);
                return Ok(());
            }
            return Err(String::from("ERR_NOTANOPERATOR"));
        }
        return Err(String::from("ERR_NOSUCHCHANEL"));
    }
    Err(String::from("ERR_NOCHANNEL"))
}

fn change_to_invite_or_public(
    mut previous_channel_name: String,
    channel_list: ChannelList,
    previous_channel_name_with_first: String,
    data_channels: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    from: char,
    to: char,
) -> Result<(), String> {
    if previous_channel_name.remove(0) == from {
        let mut new_channel = String::from(to);
        new_channel.push_str(&previous_channel_name);
        let invited: Option<Vec<String>> = if from == '&' { Some(Vec::new()) } else { None };
        let new_channel_list = ChannelList {
            invited_list: invited,
            joined_list: channel_list.joined_list.clone(),
            operators: channel_list.operators.clone(),
            topic: channel_list.topic.clone(),
            ban_mask: channel_list.ban_mask.clone(),
            secret: channel_list.secret,
            private: channel_list.private,
        };
        data_channels.remove(&previous_channel_name_with_first);
        data_channels.entry(new_channel).or_insert(new_channel_list);
        return Ok(());
    }
    Err(String::from("ALREADYTHISTYPE"))
}

pub fn spread_mode_neighbors(
    nickname: String,
    mode_info: ModeInfo,
    server: Arc<RwLock<Server>>,
    stream: &TcpStream,
) -> Result<(), String> {
    let mut string_to_spread = String::from(" MODE ");
    let mut string: String = String::from("");
    match mode_info.flag.as_str() {
        "b" => string.push_str(&construct_string_for_ban(mode_info.clone())?),
        "o" => string.push_str(&construct_string_for_oper(mode_info.clone())?),
        _ => string.push_str(&match_with_flag(&mode_info)?),
    };
    string_to_spread.push_str(&string);
    spread_command_neighbors(nickname, server, stream, string_to_spread.as_str())
}

fn construct_string_for_ban(mode_info: ModeInfo) -> Result<String, String> {
    match mode_info.ban_mask.clone() {
        Some(name) => {
            if let Some(channel) = mode_info.channel.clone() {
                match mode_info.set {
                    true => {
                        let string = format!("{} +{} {}", channel, mode_info.flag, name);
                        return Ok(string);
                    }
                    false => {
                        let string = format!("{} -{} {}", channel, mode_info.flag, name);
                        return Ok(string);
                    }
                }
            }
            Err(String::from("NO se pudo extraer"))
        }
        None => {
            if let Some(channel) = mode_info.channel {
                let string = format!("{} {}", channel, mode_info.flag);
                return Ok(string);
            }
            Err(String::from("NO se pudo extraer"))
        }
    }
}
fn construct_string_for_oper(mode_info: ModeInfo) -> Result<String, String> {
    match mode_info.nick.clone() {
        Some(name) => {
            if let Some(channel) = mode_info.channel.clone() {
                match mode_info.set {
                    true => {
                        let string = format!("{} +{} {}", channel, mode_info.flag, name);
                        return Ok(string);
                    }
                    false => {
                        let string = format!("{} -{} {}", channel, mode_info.flag, name);
                        return Ok(string);
                    }
                }
            }
            Err(String::from("NO se pudo extraer"))
        }
        None => Err(String::from("NO se pudo extraer")),
    }
}

fn match_with_flag(mode_info: &ModeInfo) -> Result<String, String> {
    if let Some(channel) = mode_info.channel.clone() {
        match mode_info.set {
            true => {
                let string = format!("{} +{}", channel, mode_info.flag);
                return Ok(string);
            }
            false => {
                let string = format!("{} -{}", channel, mode_info.flag);
                return Ok(string);
            }
        }
    }
    Err(String::from("NO se pudo extraer"))
}
