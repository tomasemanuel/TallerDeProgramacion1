use std::{
    collections::HashMap,
    io::Write,
    net::TcpStream,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::channel_list::ChannelList;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KickInfo {
    pub channel: String,
    pub nick: String,
    pub comment: Option<String>,
}

impl KickInfo {
    /// crea un nuevo Kick Info para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> Result<KickInfo, String> {
        if parametros.len() < 2 {
            return Err(String::from("ERR_NEEDMOREPARAMS"));
        }

        let len = parametros.len();
        if parametros.len() > 2 {
            let slice = &parametros[2..len];
            let mut message = slice.join(" ");
            let ch = &message[0..1];
            if ch != ":" {
                return Err(String::from("ERR_NEEDMOREPARAMS"));
            }
            message.remove(0);
            return Ok(KickInfo {
                channel: parametros[0].clone(),
                nick: parametros[1].clone(),
                comment: Some(message),
            });
        }
        Ok(KickInfo {
            channel: parametros[0].clone(),
            nick: parametros[1].clone(),
            comment: None,
        })
    }
}

/// funcion que se llama en channels.rs, chequea si se puede eliminar a un usuario de un canal,
/// si se puede entonces devuelve un reply eliminando el nick de la base de datos, sino se devuelve el error
pub fn kick_nick(
    data_channels: &mut RwLockWriteGuard<HashMap<String, ChannelList>>,
    joined_channels: &mut RwLockWriteGuard<HashMap<String, Vec<String>>>,
    kick_info: KickInfo,
    nickname: String,
    data_connected: RwLockReadGuard<HashMap<String, TcpStream>>,
) -> Result<String, String> {
    match data_connected.get(&kick_info.nick) {
        Some(mut stream) => {
            if let Some(mut channel_list) = data_channels.remove(&kick_info.channel) {
                if !channel_list.operators.contains(&nickname) {
                    return Err(String::from("ERR_NOTANOPERATOR"));
                }
                if let Some(mut vec_joined_channels) = joined_channels.remove(&kick_info.nick) {
                    channel_list
                        .joined_list
                        .retain(|names| names != &kick_info.nick);
                    vec_joined_channels.retain(|names| names != &kick_info.channel);
                    data_channels
                        .entry(kick_info.channel.clone())
                        .or_insert(channel_list);
                    joined_channels
                        .entry(kick_info.nick.clone())
                        .or_insert(vec_joined_channels);
                    let mut text = format!("{}{}", "KICK ", kick_info.channel);
                    if let Some(comment) = kick_info.comment {
                        text.push_str(&comment);
                    }
                    if stream.write(text.as_bytes()).is_err() {
                        return Err(String::from("Fallo write"));
                    }
                }
            }
            Ok(String::from("RPL_KICK"))
        }
        None => Err(String::from("ERR_NOTONCHANNEL")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invite_with_less_parameters() {
        let parametros = vec!["param1".to_string(), "param2".to_string()];
        if let Err(error_msg) = KickInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_NEEDMOREPARAMS".to_string());
        }
    }

    #[test]
    fn invite_with_enough_parameters_but_with_no_message_format() {
        let parametros = vec![
            "param1".to_string(),
            "param2".to_string(),
            "param3".to_string(),
        ];
        if let Err(error_msg) = KickInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_NEEDMOREPARAMS".to_string());
        }
    }

    #[test]
    fn create_kick_info_with_no_comment() {
        let parametros = vec![
            "&retiro_espiritual2022".to_string(),
            "MadreTeresa".to_string(),
        ];
        if let Ok(kick_info) = KickInfo::new(parametros) {
            assert_eq!(kick_info.channel, "&retiro_espiritual2022");
            assert_eq!(kick_info.nick, "MadreTeresa");
            assert_eq!(kick_info.comment, None);
        }
    }

    #[test]
    fn create_kick_info_with_comment() {
        let parametros = vec![
            "&retiro_espiritual2022".to_string(),
            "MadreTeresa".to_string(),
            ":No respeta las normas de convivencia".to_string(),
        ];
        if let Ok(kick_info) = KickInfo::new(parametros) {
            assert_eq!(kick_info.channel, "&retiro_espiritual2022");
            assert_eq!(kick_info.nick, "MadreTeresa");
            if let Some(comment) = kick_info.comment {
                assert_eq!(comment, "No respeta las normas de convivencia".to_string());
            }
        }
    }

    // #[test]
    // fn successful_kick_nick_returns_RPL_KICK(){

    //         let parametros = vec!["&apertura_mundial".to_string(),"Shakira".to_string()];
    //         let kick_info = KickInfo::new(parametros).unwrap();
    //         let nickname = "Shakira".to_string();
    //         let mut data_base: RwLock<HashMap<String, ChannelList>> = RwLock::new(HashMap::new());
    //         let channel_list = ChannelList::new("&apertura_qatar".to_string(), true);

    //         if let Ok(mut db) = data_base.write() {

    //             db.entry("&apertura_qatar".to_string()).or_insert(channel_list);

    //             let mut joined_channels: RwLock<HashMap<String, Vec<String>>> = RwLock::new(HashMap::new());

    //             if let Ok(jc) = joined_channels.write(){

    //                 jc.entry("Shakira".to_string()).or_insert(vec!["&aperutura_qatar".to_string()]);

    //                 let mut  data_connected:RwLock<HashMap<String, MockTcpStream>> = RwLock::new(HashMap::new());
    //                 if let Ok(dc) = data_connected.read(){

    //                     dc.entry("Shakira".to_string()).or_insert(MockTcpStream{read_data: vec![], write_data: vec![]});
    //                     kick_nick(&mut db, &mut jc, kick_info, nickname,dc );
    //                 }

    //             }

    //      }
    // // }
    //     struct MockTcpStream {
    //         read_data: Vec<u8>,
    //         write_data: Vec<u8>,
    //     }

    //         impl Read for MockTcpStream {
    //             /// Lee bytes del stream hasta completar el buffer y devuelve cuantos bytes fueron leidos
    //             fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    //                 self.read_data.as_slice().read(buf)
    //             }
    //         }

    //         impl Write for MockTcpStream {
    //             /// Escribe el valor del buffer en el stream y devuelve cuantos bytes fueron escritos
    //             fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    //                 self.write_data.write(buf)
    //             }

    //             fn flush(&mut self) -> io::Result<()> {
    //                 self.write_data.flush()
    //             }
    //         }
}
