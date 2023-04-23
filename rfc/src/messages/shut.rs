use std::{
    net::Shutdown,
    sync::{Arc, RwLock},
};

use crate::server::Server;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShutInfo {
    pub server_name: String,
}

impl ShutInfo {
    /// crea un nuevo ShutInfo para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> ShutInfo {
        ShutInfo {
            server_name: parametros[0].clone(),
        }
    }
}

/// Corta la conexion con el servidor que se le envio el request. Este comando siempre se va a enviar luego de un Server quit
pub fn shut_connection_from_server(
    shut_info: ShutInfo,
    server: Arc<RwLock<Server>>,
) -> Result<String, String> {
    match server.read() {
        Ok(server) => {
            for server in &server.neighbours {
                if server.0 == &shut_info.server_name {
                    match server.1.shutdown(Shutdown::Both) {
                        Err(_) => return Err(String::from("Shutdown failed")),
                        Ok(_) => return Ok(String::from("Connection Dropped")),
                    };
                }
            }
            Ok(String::from("recibido"))
        }
        Err(_) => Err(String::from("NO se pudo leer el server")),
    }
}
