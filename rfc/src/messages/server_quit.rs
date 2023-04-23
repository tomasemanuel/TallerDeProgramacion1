use std::{
    net::TcpStream,
    sync::{Arc, RwLock},
};

use crate::server::{spread_command_neighbors, Server};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SQuitInfo {
    pub server_name: String,
    pub server_message: String,
}

impl SQuitInfo {
    /// crea un nuevo SQuitInfo para su uso en parser.rs
    pub fn new(parameters: Vec<String>) -> Result<SQuitInfo, String> {
        if parameters.len() < 2 {
            return Err(String::from("ERR_NEEDMOREPARAMS"));
        }
        let slice = &parameters[1..parameters.len()];
        let mut message = slice.join(" ");
        message.remove(0);

        Ok(SQuitInfo {
            server_name: parameters[0].clone(),
            server_message: message,
        })
    }
}

/// Chequea que el usuario pueda esparcer el mensaje del Squit, este mensaje se esparce cuando queremos cortar la conexion con ese servidor
pub fn squit_request(
    squit_info: SQuitInfo,
    nickname: String,
    operators: &Arc<RwLock<Vec<String>>>,
    server: Arc<RwLock<Server>>,
    stream: &TcpStream,
) -> Result<String, String> {
    match operators.read() {
        Ok(operators) => {
            if operators.contains(&nickname) {
                return spread_squit_request_neighbors(squit_info, nickname, server, stream);
            }
            Err(String::from("ERR_NOTANOPERATOR"))
        }
        Err(_) => Err(String::from("NO se pudo leer el operators")),
    }
}
/// esparce el comando de Server quit
fn spread_squit_request_neighbors(
    squit_info: SQuitInfo,
    nickname: String,
    server: Arc<RwLock<Server>>,
    stream: &TcpStream,
) -> Result<String, String> {
    let mut string_command = String::from(" SERVERQ ");
    string_command.push_str(&squit_info.server_name);
    string_command.push_str(" :");
    string_command.push_str(&squit_info.server_message);
    spread_command_neighbors(nickname, server, stream, &string_command)?;
    Ok(String::from("ENVIADO"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squit_info_new() {
        let parameters = vec![
            String::from("server_name"),
            String::from(" :server_message"),
        ];
        let squit_info = SQuitInfo::new(parameters).unwrap();
        assert_eq!(
            squit_info,
            SQuitInfo {
                server_name: String::from("server_name"),
                server_message: String::from(":server_message"),
            }
        );
    }

    #[test]
    fn test_squit_info_new_error() {
        let parameters = vec![String::from("server_name")];
        let squit_info = SQuitInfo::new(parameters);
        assert_eq!(squit_info, Err(String::from("ERR_NEEDMOREPARAMS")),);
    }
}
