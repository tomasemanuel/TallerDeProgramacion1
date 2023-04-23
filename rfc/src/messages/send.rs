use std::io::{BufRead, BufReader};
use std::sync::{Arc, RwLock};
use std::{fs::File, io::Write};

use crate::channels::Channels;
use crate::data_base::DataBase;
use crate::server::{
    update_channels_data_base, update_channels_joined_channels_data_base,
    update_users_connected_all_servers, Server,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendInfo {
    pub name: String,
    pub content: String,
}

impl SendInfo {
    /// crea un nuevo SendInfo para su uso en parser.rs
    pub fn new(name: String, content: String) -> SendInfo {
        SendInfo { name, content }
    }
}

/// A partir de un send info, crea o updatea el file que se le mando en la carpeta con su propio nombre.
pub fn load_file(
    send_info: SendInfo,
    data_base: Arc<DataBase>,
    channels: Arc<Channels>,
    server: Arc<RwLock<Server>>,
) -> Result<String, String> {
    let mut path = "./src/".to_string();
    path.push_str(send_info.name.as_str());

    if let Ok(mut file) = File::create(path) {
        let mut reader = BufReader::new(send_info.content.as_bytes());
        let mut buf: Vec<u8> = vec![];
        if reader.read_until(b'\n', &mut buf).is_err() {
            println!("FAILED");
        }
        while !buf.is_empty() {
            let b = &buf; // b: &Vec<u8>
            if file.write_all(b).is_err() {
                println!("WRITE FAILED");
            }
            buf.clear();
            if reader.read_until(b'\n', &mut buf).is_err() {
                println!("FAILED");
            }
        }
    }
    //LLAMAR A LA FUNCION QUE ACTUALICE LAS BASES DE DATOS
    update_users_connected_all_servers(data_base, server.clone())?;
    update_channels_data_base(channels.clone(), server.clone())?;
    update_channels_joined_channels_data_base(channels, server)?;

    Ok("DB_UPDATED".to_string())
}

pub fn init_file(send_info: SendInfo) -> Result<String, String> {
    // creamos el path que queremos
    let mut path = "./src/".to_string();
    path.push_str(send_info.name.as_str());

    if let Ok(mut file) = File::create(path) {
        let mut reader = BufReader::new(send_info.content.as_bytes());
        let mut buf: Vec<u8> = vec![];
        if reader.read_until(b'\n', &mut buf).is_err() {
            println!("FAILED");
        }
        while !buf.is_empty() {
            let b = &buf; // b: &Vec<u8>
            if file.write_all(b).is_err() {
                println!("WRITE FAILED");
            }
            buf.clear();
            if reader.read_until(b'\n', &mut buf).is_err() {
                println!("FAILED");
            }
        }
    }

    Ok("DB_UPDATED".to_string())
}

// pub fn send_updates_to_neighbours(vec_bytes:Vec<u8> , server: Arc<RwLock<Server>>)->Result<(),String>{

//     if let Ok(server) = server.write() {
//         let bytes: &[u8] = &vec_bytes;
//         for mut server in &server.neighbours {
//             if let Err(_) = server.1.write(bytes) {
//                 return Err(String::from("No se pudo escribir por el buffer"))
//             }
//             return Ok(());
//         }
//     }
//     Err(String::from("NO se pudo escirbir el server"))

// }
