use std::{
    io::Write,
    net::TcpStream,
    sync::{Arc, RwLock},
};
// use crate::{send_file::*};
use crate::{send_file::FilePacket, server::Server};

// use std::thread;
// use std::time::Duration;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ServerInfo {
    pub server_name: String,
    pub server_pass: String,
}

impl ServerInfo {
    /// crea un nuevo ServerInfo para su uso en parser.rs
    pub fn new(parameters: Vec<String>) -> Result<ServerInfo, String> {
        if parameters.len() < 2 {
            return Err(String::from("ERR_NEEDMOREPARAMS"));
        }
        Ok(ServerInfo {
            server_name: parameters[0].clone(),
            server_pass: parameters[1].clone(),
        })
    }
}

/// A partir del request de un nuevo servidor, se lo agrega a sus vecinos y se le envia las bases de datos.
pub fn new_server_request(
    server_info: ServerInfo,
    mut stream: TcpStream,
    server: Arc<RwLock<Server>>,
) -> Result<String, String> {
    if let Ok(server) = server.read() {
        let f1 = FilePacket::new(
            server.users_coneccted_path.clone(),
            "users_connected".to_string(),
        );
        let f2 = FilePacket::new(server.data_file_path.clone(), "data_file".to_string());
        let f3 = FilePacket::new(
            server.data_channels_path.clone(),
            "data_channels".to_string(),
        );
        let f4 = FilePacket::new(
            server.joined_channels_path.clone(),
            "joined_channels".to_string(),
        );

        send_file_to_server(f1, &mut stream);
        send_file_to_server(f2, &mut stream);
        send_file_to_server(f3, &mut stream);
        send_file_to_server(f4, &mut stream);
    }
    if let Ok(mut server) = server.write() {
        if server_info.server_pass.as_str() != server.password.as_str() {
            return Err(String::from("WRONG_SERVER_PASSWORD"));
        }
        let mut answer = String::from("SERVER ");
        answer.push_str(&server.name);
        answer.push(' ');
        answer.push_str(server_info.server_name.as_str());
        if stream.write(answer.as_bytes()).is_err() {
            println!("INIT SHARED FILE FAILED"); //let mut server_clone = server.clone();
        }
        server.add_neighbour(stream, server_info.server_name.clone())?;
        return Ok(answer);
    }
    Err(String::from("Error desloqueando el hash")) //let mut server_clone = server.clone();
}

/// Envia un archivo a un servidor
fn send_file_to_server(file_packet: Result<FilePacket, String>, stream: &mut TcpStream) {
    if let Ok(file) = file_packet {
        if stream.write(&file.to_bytes()).is_err() {
            println!("INIT SHARED FILE FAILED"); //let mut server_clone = server.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_with_less_parameters() {
        let parametros = vec!["sv_name_1".to_string()];
        if let Err(error_msg) = ServerInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_NEEDMOREPARAMS".to_string());
        }
    }
    #[test]
    fn server_with_enough_parameters() {
        let parametros = vec!["sv_name_1".to_string(), "sv_pass1".to_string()];
        if let Err(error_msg) = ServerInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_NEEDMOREPARAMS".to_string());
        }
    }
}
