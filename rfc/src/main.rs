use rfc::answers::server_answer::ServerAnswer;
// use rfc::answers::server_answer::ServerAnswer;
use rfc::client_parser::{parse_answer, Answer};
use rfc::messages::send::init_file;
use rfc::send_file::from_bytes;
// use rfc::messages::send::{load_file, init_file};
// use rfc::mylines::MyLines;
// use rfc::send_file::from_bytes;
use rfc::config_file::*;
use rfc::server::{manage_server, Server};

use std::env;
use std::io::{BufReader, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use std::sync::Arc;
use std::sync::RwLock;

use std::collections::HashMap;

/// Funcion main que levanta servidores secundarios o el servidor principal
fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    if let Ok(configf) = ConfigFile::new("./src/config_file".to_string()) {
        let config_clone = configf.clone();
        let server = Arc::new(RwLock::new(Server {
            name: config_clone.server_name,
            neighbours: HashMap::new(),
            password: config_clone.server_password,
            data_base: None,
            data_file_path: config_clone.data_file_path,
            joined_channels_path: config_clone.joined_channels_path,
            data_channels_path: config_clone.data_channels_path,
            users_coneccted_path: config_clone.users_connected_path,
        }));
        select_server_type(configf.server_type.clone(), server, configf)
    }
}

/// Inicializa un servidor principal o uno secundario segun la informacion de los campos en configf
fn select_server_type(server_type: String, server: Arc<RwLock<Server>>, configf: ConfigFile) {
    if server_type == *"MAIN" {
        match manage_server(server, configf.main_port) {
            Err(error) => println!("Error: {error}"),
            _ => println!("Todo ok"),
        }
    } else if server_type == "SECONDARY" {
        init_secondary_server(configf);
    }
}

/// Inicializa un servidor secundario mandando el mensaje SERVER al servidor principal
fn init_secondary_server(configf: ConfigFile) {
    if let Ok(mut stream) = TcpStream::connect(configf.main_port.clone()) {
        let mut message = "SERVER ".to_string();
        message.push_str(&configf.server_name);
        message.push_str(" fiuba \r\n");
        if stream.write(message.as_bytes()).is_err() {
            println!("WRITE FAILED");
        }
        init_connection(stream, configf);
    }
}

///  Inicia la ejecucion del servidor llamando a la funcion manage_server ya sea para un servidor secundario o principal
pub fn init_server(reader: TcpStream, neighbour_name: String, configf: ConfigFile) {
    let server = Arc::new(RwLock::new(Server {
        name: configf.server_name,
        neighbours: HashMap::new(),
        password: configf.server_password,
        data_base: None,
        data_file_path: configf.data_file_path,
        joined_channels_path: configf.joined_channels_path,
        data_channels_path: configf.data_channels_path,
        users_coneccted_path: configf.users_connected_path,
    }));

    if let Ok(mut server) = server.write() {
        if let Ok(stream) = reader.try_clone() {
            if let Err(e) = server.add_neighbour(stream, neighbour_name) {
                println!("el error es {e}");
            }
        }
    }
    match manage_server(server, configf.secondary_port) {
        Err(error) => println!("Error: {error}"),
        _ => println!("Todo ok"),
    }
}

/// Inicializa el servidor secundario recibiendo y guardando en primer lugar las bases de datos del servidor original
pub fn init_connection(stream: TcpStream, configf: ConfigFile) {
    let mut server_answer_1 = ServerAnswer {
        server_name_to_connect: "".to_string(),
        server_name: "".to_string(),
    };
    let reader = BufReader::new(&stream);
    let iter = rfc::mylines2::MyLines2::new(reader);
    for lines in iter.flatten() {
        let consumed = lines.1 as usize;
        let bytes: &[u8] = &lines.0;

        if &bytes[..4] == "SEND".as_bytes() {
            let send_info = from_bytes(bytes[..consumed].to_vec());
            if let Err(e) = init_file(send_info) {
                println!("{e:?}");
            }
            continue;
        }

        if &bytes[..6] == "SERVER".as_bytes() {
            if let Ok(answer) = from_utf8(&bytes[..consumed]) {
                let answer_split = answer.split(' '); //["PRIVMSG","remitente","mensaje1","mensaje2","mensaje" n];
                let mut vec = vec![];
                for element in answer_split {
                    vec.push(element.to_string());
                }
                let answer_struct: Answer = parse_answer(vec);
                if let Answer::Server(server_answer) = answer_struct {
                    server_answer_1 = server_answer;
                    break;
                }
            }
        }
    }
    if server_answer_1.server_name_to_connect == *"" || server_answer_1.server_name == *"" {
        println!("ERROR INITIALIZING DATABASE");
    }
    init_server(stream, server_answer_1.server_name_to_connect, configf);
}
