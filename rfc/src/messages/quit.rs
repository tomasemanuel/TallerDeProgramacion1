// use std::io::Read;
use crate::data_base::DataBase;
use crate::server::Server;
use std::io::Write;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::RwLock;
// use crate::datauser::DataUserFile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuitInfo {
    pub msg: String,
}

impl QuitInfo {
    /// crea un nuevo QuitInfo para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> QuitInfo {
        let len = parametros.len();
        let slice = &parametros[0..len];
        let message = slice.join(" ");
        QuitInfo { msg: message }
    }
}

/// Corta la conexion de un usuario en un servidor, chequeando que se pueda desconectar.
pub fn quit_message(
    nickname: &String,
    quit_info: QuitInfo,
    mut stream: TcpStream,
    data_base: Arc<DataBase>,
    prefix: Option<String>,
) -> Result<String, String> {
    match data_base.data_connected.write() {
        Ok(mut connected_users) => {
            let mut answer = String::from("QUIT ");
            answer.push_str(quit_info.msg.as_str());

            if prefix.is_none() && stream.write(answer.as_bytes()).is_err() {
                return Err(String::from("Fallo write del socket"));
            }
            connected_users.remove(nickname);
            match data_base.data_connected_all_servers.write() {
                Ok(mut connected_all_servers) => {
                    connected_all_servers.retain(|users| users != nickname)
                }
                Err(_) => return Err(String::from("no se puede escribir la base de datos")),
            }
            if prefix.is_none() {
                match stream.shutdown(Shutdown::Both) {
                    Err(_) => return Err(String::from("Shutdown failed")),
                    Ok(_) => return Ok(String::from("Connection Dropped")),
                };
            }
            Ok(String::from("Connection Dropped"))
        }
        Err(_) => Err(String::from("Error leyendo el data_base.data_connected")),
    }
}

/// Esparce el mensaje de quit a todos los servidores dando a entender que se quiteo a un usuario de ese servidor
pub fn spread_quit_neighbors(
    nickname: &str,
    quit_info: QuitInfo,
    server_lock: Arc<RwLock<Server>>,
    socket_address_sender: SocketAddr,
    flag_no_return: bool,
) -> Result<(), String> {
    if let Ok(server) = server_lock.write() {
        for mut server in &server.neighbours {
            if let Ok(socket_address_receiver) = server.1.peer_addr() {
                if flag_no_return && socket_address_receiver == socket_address_sender {
                    continue;
                }
                let mut message = ":".to_string();
                message.push_str(nickname);
                message.push_str(" QUIT :");
                message.push_str(&quit_info.msg);
                message.push('\r');
                message.push('\n');
                if server.1.write(message.as_bytes()).is_err() {
                    return Err(String::from("Neighbour write failed"));
                }
                continue;
            }

            return Err(String::from("NO se pudo sacar el address"));
        }
        return Ok(());
    }
    Err(String::from("Err no server write"))
}

#[cfg(test)]
mod tests {
    // use super::*;

    // use std::{sync::{Arc, RwLock}, collections::HashMap};

    use crate::messages::quit::QuitInfo;
    // use super::quit_message;

    #[test]
    fn quit_with_message() {
        let parametros = vec![
            "me".to_string(),
            "Voy".to_string(),
            "A".to_string(),
            "mi".to_string(),
            "casa".to_string(),
        ];
        let quit_info = QuitInfo::new(parametros);
        assert_eq!(quit_info.msg, "me Voy A mi casa".to_string());
    }
    #[test]
    fn quit_without_message() {
        let quit_info = QuitInfo::new(Vec::new());
        assert_eq!(quit_info.msg, "".to_string());
    }
}

// #[test]
// fn quit_(){
//     let quit_info = QuitInfo::new(Vec::new());
//     let nickname = "Messi".to_string();
//     let joined_channels: RwLock<HashMap<String, Vec<String>>> = RwLock::new(HashMap::new());
//     if let Ok(mut joined_channels) = joined_channels.write(){
//         joined_channels.entry(String::from("Messi")).or_insert(vec!["&canal_1".to_string()]);
//     };
//     let channel_list = ChannelList {
//             invited_list: None,
//             joined_list: vec!["Messi".to_string(),"Shakira".to_string()],
//             operators: vec!["Messi".to_string(),"Shakira".to_string()],
//             topic: None,
//             ban_mask: None,
//             secret: false,
//             private: false,
//         };
//     let data_base: RwLock<HashMap<String, ChannelList>> = RwLock::new(HashMap::new());
//     if let Ok(mut data_base) = data_base.write(){
//         data_base.entry(String::from("&canal_1")).or_insert(channel_list);
//     };
//     let channels = Channels {
//         data_base,
//         joined_channels,
//     };
// if let Ok(respuesta) = quit_message(&nickname,quit_info,stream,&data_connected,channels){

//     }

// }

// struct MockTcpStream {
//     read_data: Vec<u8>,
//     write_data: Vec<u8>,
// }

//     impl Read for MockTcpStream {
//         /// Lee bytes del stream hasta completar el buffer y devuelve cuantos bytes fueron leidos
//         fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//             self.read_data.as_slice().read(buf)
//         }
//     }

//     impl Write for MockTcpStream {
//         /// Escribe el valor del buffer en el stream y devuelve cuantos bytes fueron escritos
//         fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//             self.write_data.write(buf)
//         }

//         fn flush(&mut self) -> io::Result<()> {
//             self.write_data.flush()
//         }
//     }
