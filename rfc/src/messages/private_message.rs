use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::thread;
use std::time::Duration;

use crate::channel_list::ChannelList;
use crate::channels::Channels;
use crate::data_base::DataBase;
use crate::datauser::DataUserFile;
use crate::server::Server;

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct PrivateInfo {
    pub receivers: Vec<String>,
    // pub receiver: String,
    pub message: String,
}

#[derive(Clone)]
struct DataPrivMsg {
    receiver: String,
    data_user: DataUserFile,
    message: String,
    server: Arc<RwLock<Server>>,
    channel: Option<String>,
}

impl PrivateInfo {
    /// crea un nuevo PrivateInfo para su uso en parser.rs
    pub fn new(parameter: Vec<String>) -> Result<PrivateInfo, String> {
        let len = parameter.len();
        if len == 1 {
            return Err(String::from("ERR_NORECIPIENT"));
        }
        let split = parameter[0].split(',');
        let receivers = split.into_iter().map(|x| x.to_string()).collect();
        let slice = &parameter[1..len];

        if parameter[1].as_bytes()[0] == b':' {
            let mut message = slice.join(" ");
            message.remove(0);
            return Ok(PrivateInfo { receivers, message });
        }
        Err(String::from("ERR_NOTEXTTOSEND"))
    }
}

/// Dependiendo si el usuario se encuentra en los usuarios conectados en toda la red, y dependiendo si se quiere enviar un mensaje por el canal, esta funcion
/// chequea caso por caso devolviendo su respectivo error si no se pudo enviar el mensaje
pub fn send_private_message(
    data_base: Arc<DataBase>,
    private_info: PrivateInfo,
    data_user: DataUserFile,
    channels: &Arc<Channels>,
    server: Arc<RwLock<Server>>, //[8097:stream,  8096:stream]
    stream: &TcpStream,
) -> Result<(), String> {
    match data_base.data_connected.read() {
        Ok(hash_conected) => {
            match channels.data_base.read() {
                Ok(hash_channels) => match data_base.data_registered.read() {
                    Ok(hash_registered) => {
                        for receiver in private_info.receivers.clone() {
                            if receiver_is_channel(receiver.clone()) {
                                if hash_channels.get(&receiver).is_none() {
                                    return Err(String::from("ERR_NOSUCHCHANNEL"));
                                }
                                continue;
                            }
                            if let Ok(users_connected_accross_servers) =
                                data_base.data_connected_all_servers.read()
                            {
                                if !users_connected_accross_servers.contains(&receiver) {
                                    return Err(String::from("ERR_NONICKNAME"));
                                }
                            }
                        }
                        for receiver in private_info.receivers {
                            if receiver_is_channel(receiver.clone()) {
                                let channel = receiver.clone();
                                let data_priv_msg = DataPrivMsg {
                                    receiver,
                                    data_user: data_user.clone(),
                                    message: private_info.message.clone(),
                                    server: server.clone(),
                                    channel: Some(channel),
                                };
                                send_channel_message(
                                    &hash_channels,
                                    &hash_conected,
                                    &hash_registered,
                                    data_priv_msg,
                                    stream,
                                    server.clone(),
                                )?;
                                continue;
                            }
                            let data_priv_msg = DataPrivMsg {
                                receiver,
                                data_user: data_user.clone(),
                                message: private_info.message.clone(),
                                server: server.clone(),
                                channel: None,
                            };
                            send_singular_message(
                                &hash_conected,
                                &hash_registered,
                                data_priv_msg.clone(),
                                server.clone(),
                                stream,
                            )?;
                            spread_private_message(
                                &data_priv_msg.channel,
                                &data_priv_msg.receiver,
                                &data_priv_msg.message,
                                &data_priv_msg.data_user.nickname,
                                data_priv_msg.server,
                                stream,
                            )?;
                        }
                        Ok(())
                    }
                    Err(_) => Err(String::from("Error Deslockeando el hash")),
                },
                Err(_) => Err(String::from("Error Deslockeando el hash")),
            }

            // CHEQUEAR SI
            // Si algun receiver no esta conectado o no existe, no se manda ningun mensaje y se devuelve error
        }
        Err(_) => Err(String::from("Error Deslockeando el hash")),
    }
}

/// En el caso de que se quiera mandar un mensaje por un canal, se envia un mensaje privado a cada usuario anteponiendo el canal por donde
/// provenga ese mensaje
fn send_channel_message(
    hash_channel: &RwLockReadGuard<HashMap<String, ChannelList>>,
    hash_conected: &RwLockReadGuard<HashMap<String, TcpStream>>,
    data_base: &RwLockReadGuard<HashMap<String, DataUserFile>>,
    mut data_priv_msg: DataPrivMsg,
    stream: &TcpStream,
    server: Arc<RwLock<Server>>,
) -> Result<(), String> {
    if let Some(channel_list) = hash_channel.get(&data_priv_msg.receiver) {
        if !channel_list
            .joined_list
            .contains(&data_priv_msg.data_user.nickname)
        {
            return Err(String::from("ERR_CANNOTSENDTOCHAN"));
        }
        for joined_user in &channel_list.joined_list {
            // tomas, franco
            if joined_user != data_priv_msg.data_user.nickname.clone().as_str() {
                data_priv_msg.receiver = joined_user.clone(); // xq?
                send_singular_message(
                    hash_conected,
                    data_base,
                    data_priv_msg.clone(),
                    server.clone(),
                    stream,
                )?;
            }
        }
        spread_private_message(
            &data_priv_msg.channel,
            &data_priv_msg.receiver,
            &data_priv_msg.message,
            &data_priv_msg.data_user.nickname,
            data_priv_msg.server,
            stream,
        )?;
    }
    Ok(())
}

/// dado el strea, se chequea si se puede enviar el mensaje, si el usuario no esta en dicho servidor, se esparce el mensaje a todos los servidores para
/// chequear si existe en dicho servidor
fn send_singular_message(
    hash_conected: &RwLockReadGuard<HashMap<String, TcpStream>>, //conectado a servidor
    data_base: &RwLockReadGuard<HashMap<String, DataUserFile>>,  //la información del user
    data_priv_msg: DataPrivMsg,
    server: Arc<RwLock<Server>>,
    stream_sender: &TcpStream,
) -> Result<(), String> {
    if let Some(stream) = hash_conected.get(&data_priv_msg.receiver) {
        //user esta en mi server?
        if let Some(data_user_file) = data_base.get(&data_priv_msg.receiver) {
            if data_priv_msg.channel.is_none() {
                if let Some(away_message) = data_user_file.away.clone() {
                    return send_away_message_from_receiver(
                        hash_conected,
                        &data_priv_msg.receiver,
                        &data_priv_msg.data_user.nickname,
                        away_message,
                        server,
                        stream_sender,
                    );
                }
            }
        }
        if let Ok(stream_clonado) = stream.try_clone() {
            send_generic_private_message(
                &data_priv_msg.channel,
                &data_priv_msg.data_user,
                stream_clonado,
                &data_priv_msg.message,
            )?;
        };
    }
    Ok(())
    // let mut channel = None;
    // if let Some(channel_name) = &data_priv_msg.channel {
    //     channel = Some(channel_name.clone());
    // }
    //spread_private_message(&data_priv_msg.channel,&data_priv_msg.receiver,&data_priv_msg.message,&data_priv_msg.data_user.nickname, data_priv_msg.server,stream) // the user is in another server!!!!!
}

/// Se manda un mensaje generico con el respectivo string del tipo PRIVMSG (canal,)remitente :mensaje
fn send_generic_private_message(
    channel: &Option<String>,
    data_user: &DataUserFile,
    mut stream: TcpStream,
    message: &str,
) -> Result<(), String> {
    let mut message_to_write: String = String::from("PRIVMSG ");
    if let Some(channel_name) = channel {
        message_to_write.push_str(channel_name.as_str());
        message_to_write.push(',');
    }
    message_to_write.push_str(data_user.nickname.as_str());
    message_to_write.push_str(" :");
    message_to_write.push_str(message);
    if stream.write(message_to_write.as_bytes()).is_err() {
        return Err(String::from("No se pudo escribir en el stream"));
    };
    thread::sleep(Duration::from_millis(1));

    Ok(())
}

/// Si le llega un mensaje de privmsg de otro servidor, dicha funcion devuelve el away message asociado a ese usuario
fn send_away_message_from_receiver(
    hash_conected: &RwLockReadGuard<HashMap<String, TcpStream>>,
    receiver: &str,
    sender: &String,
    away_message: String,
    server: Arc<RwLock<Server>>,
    server_sender: &TcpStream,
) -> Result<(), String> {
    if let Some(mut stream) = hash_conected.get(sender) {
        let mut message_to_write: String = String::from("PRIVMSG ");
        if *receiver == sender.clone() {
            message_to_write.push_str("Yo");
        } else {
            message_to_write.push_str(receiver);
        }
        message_to_write.push_str(" :");
        message_to_write.push_str(away_message.as_str());

        if stream.write(message_to_write.as_bytes()).is_err() {
            println!("Fallo el write del send away message from receiver"); //esto deberiamos mandarlo a un log quizas
        }
        return Ok(());
    }
    spread_private_message(
        &None,
        receiver,
        &away_message,
        sender,
        server,
        server_sender,
    )
}

fn receiver_is_channel(receiver: String) -> bool {
    receiver.starts_with('&') || receiver.starts_with('#')
}

/// Esparce el mensaje privado a todos los servidores en el caso de que un usuario que se quiso mandar un mensaje no
/// se encontrase en el servidor actual
fn spread_private_message(
    channel: &Option<String>,
    receiver: &str,
    message: &str,
    nickname: &str,
    server_lock: Arc<RwLock<Server>>,
    server_sender: &TcpStream,
) -> Result<(), String> {
    if let Ok(server) = server_lock.write() {
        for mut server in &server.neighbours {
            if let Ok(socket_address_receiver) = server.1.peer_addr() {
                if let Ok(socket_address_sender) = server_sender.peer_addr() {
                    if socket_address_receiver == socket_address_sender {
                        continue;
                    }
                    let mut message_to_spread = ":".to_string();
                    message_to_spread.push_str(nickname);
                    message_to_spread.push_str(" PRIVMSG ");

                    if let Some(channel_name) = channel {
                        message_to_spread.push_str(channel_name);
                    } else {
                        message_to_spread.push_str(receiver);
                    }

                    message_to_spread.push_str(" :");
                    message_to_spread.push_str(message);
                    message_to_spread.push('\r');
                    message_to_spread.push('\n');
                    if server.1.write(message_to_spread.as_bytes()).is_err() {
                        return Err(String::from("Neighbour write failed"));
                    }
                    continue;
                }
                return Err(String::from("NO se pudo ver el socket address"));
            }
            return Err(String::from("NO se pudo ver el socket address"));
        }
        return Ok(());
    }
    Err(String::from("Err no server write"))
}
