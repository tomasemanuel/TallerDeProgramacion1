use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{Arc, RwLock},
};

use crate::server::Server;

use super::quit::{spread_quit_neighbors, QuitInfo};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SQuitRequestInfo {
    pub server_name: String,
    pub server_message: String,
}

impl SQuitRequestInfo {
    /// crea un nuevo SQuitRequestInfo para su uso en parser.rs
    pub fn new(parameters: Vec<String>) -> SQuitRequestInfo {
        let slice = &parameters[1..parameters.len()];
        let mut message = slice.join(" ");
        message.remove(0);

        SQuitRequestInfo {
            server_name: parameters[0].clone(),
            server_message: message,
        }
    }
}

/// Si queremos eliminar un servidor de la base de servidores se llama a esta funcion. Chequea que este sea el servidor que se quiere eliminar.
pub fn quit_server(
    server_quit_request_info: SQuitRequestInfo,
    server: Arc<RwLock<Server>>,
    data_connected: Arc<RwLock<HashMap<String, TcpStream>>>,
    stream: &TcpStream,
) -> Result<String, String> {
    let server_name = match server.read() {
        Ok(server_struct) => server_struct.name.clone(),
        Err(_) => return Err(String::from("NO se pudo leer el arc de server")),
    };
    if server_name == server_quit_request_info.server_name {
        return issue_quit_request_from_server(server, data_connected, stream);
    }

    let mut message = String::from("Se elimino a ");
    message.push_str(&server_quit_request_info.server_name);
    message.push_str(" de la base de datos de servidores");
    Ok(message)
}

/// A partir de los usuarios conectados a ese canal, se esparce el mensaje de quit a todos los otros servidores.
fn issue_quit_request_from_server(
    server: Arc<RwLock<Server>>,
    data_connected: Arc<RwLock<HashMap<String, TcpStream>>>,
    stream: &TcpStream,
) -> Result<String, String> {
    let socket = match stream.peer_addr() {
        Ok(socket) => socket,
        Err(_) => return Err(String::from("no se pudo leer el socket address")),
    };
    if let Ok(data_connected) = data_connected.read() {
        for (nickname, _) in data_connected.iter() {
            let quit_msg = String::from("QUit from a server quit");
            let quit_info = QuitInfo { msg: quit_msg };
            spread_quit_neighbors(nickname, quit_info, server.clone(), socket, false)?;
        }
        return Ok(String::from("OK"));
    }
    Err(String::from("NO se pudo leer el data connected"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_squit_request_info() {
        let parameters: Vec<String> = vec!["server_1".to_string(), ":comment".to_string()];
        let squit_request_info = SQuitRequestInfo::new(parameters);

        assert_eq!(squit_request_info.server_name, "server_1".to_string());
        assert_eq!(squit_request_info.server_message, "comment".to_string());
    }
}
