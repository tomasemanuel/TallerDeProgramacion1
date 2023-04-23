use crate::channel_list::ChannelList;
use crate::data_base::DataBase;
use crate::datauser::DataUserFile;
use crate::messages::connect::*;
use crate::messages::invite::*;
use crate::messages::join::*;
use crate::messages::kick::kick_nick;
use crate::messages::kick::KickInfo;
use crate::messages::list::*;
use crate::messages::mode::*;
use crate::messages::names::*;
use crate::messages::part::*;
use crate::messages::topic::*;
use crate::messages::who::*;
use crate::messages::whois::whois_function;
use crate::messages::whois::WhoIsInfo;
use crate::server::update_channels_data_base;
use crate::server::update_channels_joined_channels_data_base;
use crate::server::Server;

use std::net::TcpStream;
use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};

#[derive(Debug)]
pub struct Channels {
    pub data_base: RwLock<HashMap<String, ChannelList>>,
    pub joined_channels: RwLock<HashMap<String, Vec<String>>>,
}

/// Esta estructura guarda todos los channels creados por clientes
impl Channels {
    /// Devuelve un Channels default, con las bases de datos updateadas.
    pub fn default(server: Option<Arc<RwLock<Server>>>) -> Result<Arc<Channels>, String> {
        let data_base: RwLock<HashMap<String, ChannelList>> = RwLock::new(HashMap::new());
        let joined_channels: RwLock<HashMap<String, Vec<String>>> = RwLock::new(HashMap::new());
        let channels = Arc::new(Channels {
            data_base,
            joined_channels,
        });
        if let Some(server) = server {
            update_channels_data_base(channels.clone(), server.clone())?;
            update_channels_joined_channels_data_base(channels.clone(), server)?;
        }
        Ok(channels)
    }
    pub fn join_channel(
        &self,
        join_info: JoinInfo,
        nickname: String,
        operators: Arc<RwLock<Vec<String>>>,
    ) -> Result<String, String> {
        // esta registrado!!
        match self.data_base.write() {
            Ok(mut data_base) => match self.joined_channels.write() {
                Ok(mut joined_channels) => match operators.read() {
                    Ok(operators) => check_channels(
                        join_info,
                        &mut data_base,
                        &mut joined_channels,
                        nickname,
                        operators,
                    ),
                    Err(e) => Err(e.to_string()),
                },
                Err(_) => Err(String::from("Fallo read de channels")),
            },
            Err(_) => Err(String::from("Fallo read de channels")),
        }
    }

    pub fn leave_channel(&self, part_info: PartInfo, nickname: String) -> Result<String, String> {
        match self.data_base.write() {
            Ok(data_base) => match self.joined_channels.write() {
                Ok(joined_channels) => part(part_info, data_base, joined_channels, nickname),
                Err(_) => Err(String::from("Fallo read de channels")),
            },
            Err(_) => Err(String::from("Fallo read de channels")),
        }
    }
    pub fn names(&self, name_info: NamesInfo, data_base: Arc<DataBase>) -> Result<String, String> {
        match self.data_base.read() {
            Ok(channels_data_base) => match self.joined_channels.read() {
                Ok(joined_channels) => names(
                    channels_data_base,
                    joined_channels,
                    name_info.channel_list,
                    data_base,
                ),
                Err(_) => Err(String::from("Fallo read de joined channels")),
            },
            Err(_) => Err(String::from("Fallo read de channels")),
        }
    }
    pub fn list(&self, list_info: ListInfo, nickname: &String) -> Result<String, String> {
        match self.data_base.read() {
            Ok(data_base) => list_of_channels(data_base, list_info.channel_list, nickname),
            Err(_) => Err(String::from("Fallo read de channels")),
        }
    }
    pub fn who(
        &self,
        who_info: WhoInfo,
        data_registered: &Arc<RwLock<HashMap<String, DataUserFile>>>,
        nickname: String,
    ) -> Result<String, String> {
        match self.data_base.read() {
            Ok(data_channels) => match self.joined_channels.read() {
                Ok(joined_channels) => match data_registered.read() {
                    Ok(data_registered) => Ok(who_function(
                        data_channels,
                        joined_channels,
                        who_info,
                        data_registered,
                        nickname,
                    )),
                    Err(_) => Err(String::from("Fallo read de registrados")),
                },
                Err(_) => Err(String::from("Fallo read de channels")),
            },
            Err(_) => Err(String::from("Fallo read de channels")),
        }
    }
    pub fn who_info_of_user(
        &self,
        who_is_info: WhoIsInfo,
        data_base: Arc<DataBase>,
    ) -> Result<String, String> {
        match data_base.data_registered.read() {
            Ok(data_registered) => match data_registered.get(&who_is_info.name) {
                Some(data_user) => match data_base.operators.read() {
                    Ok(operators) => match &self.joined_channels.read() {
                        Ok(joined_channels) => {
                            whois_function(operators, data_user, joined_channels)
                        }
                        Err(e) => Err(e.to_string()),
                    },
                    Err(e) => Err(e.to_string()),
                },
                None => Err(String::from("NO_NICKNAME_PRESENT")),
            },
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn invite_to_channel(
        &self,
        invite_info: InviteInfo,
        data_base: &Arc<RwLock<HashMap<String, DataUserFile>>>,
        nickname: &String,
    ) -> Result<String, String> {
        match data_base.read() {
            Ok(data_base) => match self.data_base.write() {
                Ok(mut data_channels) => invite_channel(
                    invite_info,
                    &mut data_base.clone(),
                    &mut data_channels,
                    nickname,
                ),
                Err(_) => Err(String::from("Fallo read de channels")),
            },
            Err(_) => Err(String::from("Fallo read de la base de datos")),
        }
    }
    pub fn kick_user_from_channel(
        &self,
        kick_info: KickInfo,
        nickname: String,
        hash_nickname: &Arc<RwLock<HashMap<String, TcpStream>>>,
    ) -> Result<String, String> {
        match self.data_base.write() {
            Ok(mut data_channels) => match self.joined_channels.write() {
                Ok(mut joined_channels) => match hash_nickname.read() {
                    Ok(data_connected) => kick_nick(
                        &mut data_channels,
                        &mut joined_channels,
                        kick_info,
                        nickname,
                        data_connected,
                    ),
                    Err(_) => Err(String::from("Fallo read de registrados")),
                },
                Err(_) => Err(String::from("Fallo read de channels")),
            },
            Err(_) => Err(String::from("Fallo read de channels")),
        }
    }

    pub fn give_or_receive_topic(
        &self,
        topic_info: TopicInfo,
        nickname: &String,
    ) -> Result<String, String> {
        match self.joined_channels.read() {
            Ok(joined_channels) => match self.data_base.write() {
                Ok(mut data_channels) => {
                    give_or_receive_topic(topic_info, joined_channels, &mut data_channels, nickname)
                }
                Err(_) => Err(String::from("fallo el read")),
            },
            Err(_) => Err(String::from("fallo el read")),
        }
    }
    pub fn mode(&self, mode_info: ModeInfo, nickname: &String) -> Result<String, String> {
        match self.data_base.write() {
            Ok(mut data_channels) => {
                // everything_is_right(data_channels,nickname,mode_info)?;
                match mode_info.flag.as_str() {
                    "o" => set_operator_on_channel(&mut data_channels, mode_info, nickname)?,
                    "b" => return set_ban_mask(&mut data_channels, mode_info, nickname),
                    "i" => set_invite_channel(&mut data_channels, mode_info, nickname)?,
                    "s" => set_secret_channel(&mut data_channels, mode_info, nickname)?,
                    "p" => set_private_channel(&mut data_channels, mode_info, nickname)?,
                    _ => return Err(String::from("no se encontro nada")),
                }
                Ok(String::from("RPLOK"))
            }
            Err(_) => Err(String::from("Fallo read de channels")),
        }
    }
    pub fn return_connected_channels(&self, nickname: &String) -> Result<String, String> {
        match self.joined_channels.read() {
            Ok(joined_channels) => Ok(return_connected_channels(&joined_channels, nickname)),
            Err(_) => Err(String::from("fallo el read")),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create_channels() {
        let channels = Arc::new(Channels {
            data_base: RwLock::new(HashMap::new()),
            joined_channels: RwLock::new(HashMap::new()),
        });
        let channels_assert = Channels::default(None);
        match channels_assert {
            Ok(channels_assert) => {
                if let Ok(data_base_assert) = channels_assert.data_base.read() {
                    if let Ok(data_base_default) = channels.data_base.read() {
                        assert_eq!(data_base_default.clone(), data_base_assert.clone());
                    }
                }
                if let Ok(joined_channels_assert) = channels_assert.joined_channels.read() {
                    if let Ok(joined_channels_default) = channels.joined_channels.read() {
                        assert_eq!(
                            joined_channels_default.clone(),
                            joined_channels_assert.clone()
                        );
                    }
                }
            }
            Err(_) => assert_eq!(true, false),
        }
    }
    #[test]
    fn join_a_channel() {
        let channels = Channels::default(None);
        let join_info = JoinInfo::new(vec!["&canal1".to_string()]);
        let operators: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
        if let Ok(channels) = channels {
            if let Ok(join_info) = join_info {
                let respuesta =
                    channels.join_channel(join_info.clone(), "usuario_1".to_string(), operators);
                match respuesta {
                    Ok(respuesta) => assert_eq!("CHANNELJOINED &canal1".to_string(), respuesta),
                    Err(_) => assert_eq!(false, true),
                }
                if let Ok(joined_channels) = channels.joined_channels.read() {
                    let mut joined_chan: HashMap<String, Vec<String>> = HashMap::new();
                    joined_chan
                        .entry("usuario_1".to_string())
                        .or_insert(vec!["&canal1".to_string()]);
                    assert_eq!(joined_channels.clone(), joined_chan);
                }
            }
        }
    }

    #[test]
    fn leave_a_channel() {
        let channels = Channels::default(None);
        let join_info = JoinInfo::new(vec!["&canal1".to_string()]);
        let operators: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
        let part_info = PartInfo::new(vec!["&canal1".to_string()]);
        if let Ok(channels) = channels {
            if let Ok(join_info) = join_info {
                if let Ok(_) = channels.join_channel(join_info, "usuario_1".to_string(), operators)
                {
                    if let Ok(part_info) = part_info {
                        let respuesta =
                            channels.leave_channel(part_info.clone(), "usuario_1".to_string());
                        match respuesta {
                            Ok(respuesta) => assert_eq!("PART &canal1,".to_string(), respuesta),
                            Err(_) => assert_eq!(false, true),
                        }
                        if let Ok(joined_channels) = channels.joined_channels.read() {
                            let mut joined_chan: HashMap<String, Vec<String>> = HashMap::new();
                            joined_chan.entry("usuario_1".to_string()).or_insert(vec![]);
                            assert_eq!(joined_channels.clone(), joined_chan);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn names_given_correctly() {
        let channels = Channels::default(None);
        let name_info = NamesInfo::new(vec!["&canal1".to_string()]);
        let join_info = JoinInfo::new(vec!["&canal1".to_string()]);
        let operators: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
        let db_lock = Arc::new(RwLock::new(HashMap::new()));

        let data_connected: Arc<RwLock<HashMap<String, TcpStream>>> =
            Arc::new(RwLock::new(HashMap::new())); // nick: TcpStream
        let server_operators: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
        let data_connected_all_servers: Arc<RwLock<Vec<String>>> =
            Arc::new(RwLock::new(Vec::new()));
        let database = Arc::new(DataBase {
            data_registered: db_lock,
            data_connected,
            data_connected_all_servers,
            operators: server_operators,
        });
        if let Ok(channels) = channels {
            if let Ok(join_info) = join_info {
                if let Ok(_) = channels.join_channel(join_info, "usuario_1".to_string(), operators)
                {
                    if let Ok(name_info) = name_info {
                        let respuesta = channels.names(name_info, database);
                        match respuesta {
                            Ok(respuesta) => {
                                assert_eq!("NAMES &canal1: usuario_1".to_string(), respuesta)
                            }
                            Err(_) => assert_eq!(false, true),
                        }
                    }
                }
            }
        }
    }
    #[test]
    // list(&self, list_info: ListInfo, nickname: &String)
    fn list_correct() {
        let channels = Channels::default(None);
        let list_info = ListInfo::new(vec!["&canal1".to_string()]);
        let join_info = JoinInfo::new(vec!["&canal1".to_string()]);
        let operators: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
        if let Ok(channels) = channels {
            if let Ok(join_info) = join_info {
                if let Ok(_) = channels.join_channel(join_info, "usuario_1".to_string(), operators)
                {
                    if let Ok(list_info) = list_info {
                        let respuesta = channels.list(list_info, &"usuario_1".to_string());
                        match respuesta {
                            Ok(respuesta) => assert_eq!("LIST &canal1:".to_string(), respuesta),
                            Err(_) => assert_eq!(false, true),
                        }
                    }
                }
            }
        }
    }
    // #[test]
    // fn invite_correct() {
    //     let channels = Channels::default(None);
    //     let invite_info = InviteInfo::new(vec!["tomas".to_string(), "&canal1".to_string()]);
    //     let join_info = JoinInfo::new(vec!["&canal1".to_string()]);
    //     let operators: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
    //     let data_user_tomas = DataUserFile {
    //         password: "contra".to_string(),
    //         nickname: "tomas".to_string(),
    //         nickname_actualizado: "same".to_string(),
    //         username: "contra".to_string(),
    //         hostname: "contra".to_string(),
    //         servername: "contra".to_string(),
    //         realname: "contra".to_string(),
    //         away: None,
    //     };
    //     let mut hash = HashMap::new();
    //     hash.entry("tomas".to_string()).or_insert(data_user_tomas);
    //     let db_lock = Arc::new(RwLock::new(hash));
    //     let mode_info = ModeInfo {
    //         channel: Some("&canal1".to_string()),
    //         nick: None,
    //         flag: "i".to_string(),
    //         limit: None,
    //         user: None,
    //         ban_mask: None,
    //         set: true,
    //     };
    //     if let Ok(channels) = channels {
    //         if let Ok(join_info) = join_info {
    //             if let Ok(_) = channels.join_channel(join_info, "usuario_1".to_string(), operators)
    //             {
    //                 if let Ok(mut data_channels) = channels.data_base.write() {
    //                     if let Ok(_) = set_invite_channel(
    //                         &mut data_channels,
    //                         mode_info,
    //                         &"usuario_1".to_string(),
    //                     ) {
    //                         if let Ok(invite_info) = invite_info {
    //                             let respuesta = channels.invite_to_channel(
    //                                 invite_info,
    //                                 &db_lock,
    //                                 &"usuario_1".to_string(),
    //                             );
    //                             match respuesta {
    //                                 Ok(respuesta) => assert_eq!(
    //                                     "NAMES &canal1: usuario_1".to_string(),
    //                                     respuesta
    //                                 ),
    //                                 Err(e) => println!("e: {:?}", e),
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
}

// (&self, name_info: NamesInfo, data_base: Arc<DataBase>
