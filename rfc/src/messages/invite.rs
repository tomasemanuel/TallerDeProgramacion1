use std::collections::HashMap;

use crate::{channel_list::ChannelList, datauser::DataUserFile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InviteInfo {
    pub channel: String,
    pub nick: String,
}

impl InviteInfo {
    /// crea un nuevo Invite Info para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> Result<InviteInfo, String> {
        if parametros.len() < 2 {
            return Err(String::from("ERR_NEEDMOREPARAMS"));
        }
        if parametros.len() > 2 {
            return Err(String::from("Less parameters needed on invite command"));
        }
        Ok(InviteInfo {
            nick: parametros[0].to_string(),
            channel: parametros[1].to_string(),
        })
    }
}
/// funcion que se usa en channels.rs para invitar a un nick a un canal, devuelve un Result porque tambien maneja los errores de logeo
pub fn invite_channel(
    invite_info: InviteInfo,
    data_base: &mut HashMap<String, DataUserFile>,
    data_channels: &mut HashMap<String, ChannelList>,
    nickname: &String,
) -> Result<String, String> {
    if !data_base.contains_key(&invite_info.nick) {
        return Err(String::from("ERR_NOSUCHNICK"));
    }
    if !data_channels.contains_key(&invite_info.channel) {
        // crear un nuevo canal e invitar
        return Ok(String::from("ERR_NOSUCHCHANNEL"));
    }
    if let Some(channel_list) = data_channels.get(&invite_info.channel) {
        let joined_list = channel_list.joined_list.clone();
        if joined_list.contains(&invite_info.nick) {
            return Err(String::from("ERR_USERONCHANNEL"));
        }
        match channel_list.invited_list.clone() {
            Some(mut invited_list) => {
                if channel_list.operators.contains(nickname) {
                    if invited_list.contains(&invite_info.nick) {
                        return Err(String::from("Already invited"));
                    }
                    invited_list.push(invite_info.nick.clone());
                    let channel_list = ChannelList {
                        invited_list: Some(invited_list),
                        joined_list,
                        operators: channel_list.operators.clone(),
                        topic: channel_list.topic.clone(),
                        ban_mask: channel_list.ban_mask.clone(),
                        secret: channel_list.secret,
                        private: channel_list.private,
                    };
                    data_channels.remove(&invite_info.channel);
                    data_channels
                        .entry(invite_info.channel.clone())
                        .or_insert(channel_list);
                    return Ok(format!(
                        "{}{}{}{}",
                        "RPL_INVITING ", invite_info.channel, " ", invite_info.nick
                    ));
                }
                return Err(String::from("ERR_NOTANOPERATOR"));
            }
            None => return Err(String::from("ERR_PUBLICCHANNEL")),
        }
    }
    Err(String::from("couldnt get the key to hash"))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn invite_with_less_parameters() {
        let parametros = vec!["param1".to_string()];
        if let Err(error_msg) = InviteInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_NEEDMOREPARAMS".to_string());
        }
    }

    #[test]
    fn invite_with_to_much_parameters() {
        let parametros = vec![
            "param1".to_string(),
            "param2".to_string(),
            "param3".to_string(),
        ];
        if let Err(error_msg) = InviteInfo::new(parametros) {
            assert_eq!(
                error_msg,
                "Less parameters needed on invite command".to_string()
            );
        }
    }

    #[test]
    fn create_an_invite_with_nick_and_channel() {
        let parametros = vec!["PlinPlin".to_string(), "&canal_god".to_string()];
        if let Ok(invite) = InviteInfo::new(parametros) {
            assert_eq!(invite.channel, "&canal_god".to_string());
            assert_eq!(invite.nick, "PlinPlin".to_string());
        }
    }

    #[test]
    fn successful_invite_channel_returns_rpl_inviting() {
        if let Ok(invite_info) =
            InviteInfo::new(vec!["Maradona".to_string(), "&escaloneta".to_string()])
        {
            let mut data_base: HashMap<String, DataUserFile> = HashMap::new();
            data_base
                .entry("Maradona".to_string())
                .or_insert(DataUserFile {
                    password: "marado".to_string(),
                    nickname: "Maradona".to_string(),
                    nickname_actualizado: "same".to_string(),
                    username: "marado123".to_string(),
                    hostname: "127.0.0.1".to_string(),
                    servername: "sv-name".to_string(),
                    realname: "realname".to_string(),
                    away: None,
                });
            let nickname = "Maradona".to_string();
            let mut data_channels: HashMap<String, ChannelList> = HashMap::new();
            let channel_list = ChannelList::new("escaloneta".to_string(), true);
            data_channels
                .entry("&escaloneta".to_string())
                .or_insert(channel_list);

            if let Ok(invite_response) =
                invite_channel(invite_info, &mut data_base, &mut data_channels, &nickname)
            {
                assert_eq!(invite_response, "RPL_INVITING".to_string());
            }
        }
    }
}
