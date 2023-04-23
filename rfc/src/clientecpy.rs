use std::collections::HashMap;
use std::sync::{RwLock, Arc};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
// use std::ops::Deref;
// use std::sync::mpsc::{Sender, Receiver};

// use std::time::Duration;
use std::{env::args, io::Read, io::Write, net::TcpStream}; //esto es para probar el codigo
static CLIENT_ARGS: usize = 3;
use std::{io, thread};
// use std::io::{BufReader, BufRead};
use rfc::server::{manage_server, Server};
// use rfc::mylines::MyLines;
// use rfc::parser::Message;
use std::str;

use rfc::client_parser::{parse_answer, Answer};

fn main() {
    //let _socket =  SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0 , 0, 1)), 80);
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        let app_name = &argv[0];
    }
    let address = argv[1].clone() + ":" + &argv[2];
    let (tx, rx) = mpsc::channel();
    if let Ok(mut stream) = TcpStream::connect(address) {
        let _result = stream.set_read_timeout(Some(Duration::new(0, 10000000)));
        take_user_input(tx).unwrap();
        let mut reader = stream.try_clone().unwrap();
        escribir_mensaje(&mut stream, &mut reader, &rx);
    } else {
        println!("No se pudo conectar...");
    }
}

fn escribir_mensaje(writer: &mut TcpStream, reader: &mut TcpStream, rx: &Receiver<String>) {
    let mut new_port:u32 = 8095;
    loop {
        let d = Duration::from_nanos(500000);
        let read = rx.recv_timeout(d);
        if let Ok(mut message) = read {
            message.push('\r');
            message.push('\n');
            match writer.write(message.as_bytes()) {
                Err(_) => println!("Fallo conexion con servidor"),
                Ok(_) => {
                    if writer.flush().is_err() {
                        println!("Error con flush")
                    }
                }
            }
        }
        let mut buffer = [0; 500];
        if let Ok(consumed) = reader.read(&mut buffer) {
            
            let answer: &str = str::from_utf8(&buffer[..consumed]).unwrap();
            if answer.is_empty() {
                println!("Connection finished");
                break;
            }

            let answer_split = answer.split(' ');   
            let mut  vec = vec![];
            for element in answer_split {
                vec.push(element.to_string());
            } 
           let answer_struct: Answer = parse_answer(vec);
           if let Answer::Server(server_answer) = answer_struct{
                new_port+=1;
               return init_server(reader, server_answer.server_name_to_connect,server_answer.server_name,new_port)
           }
        }
    }
}

fn take_user_input(tx: Sender<String>) -> Result<(), String> {
    // Esto tendria que ser lo que se escribe en la UI

    let _join_handler = thread::spawn(move || {
        loop {
            let mut command: String = String::new();
            io::stdin()
                .read_line(&mut command)
                .expect("Failed to read line");
            command.remove(command.len() - 1);
            command.push('\r');
            command.push('\n');
            //Envio el mensaje al Ts (thread con el stream)
            tx.send(command).unwrap();
        }
    });
    Ok(())
}


pub fn init_server(reader: &mut TcpStream, neighbour_name: String, server_name: String,port:u32){
    
    let server = Arc::new(RwLock::new(Server {name:server_name, neighbours: HashMap::new(), password:"fiuba".to_string(),data_base:None,
    data_file_path: "".to_string(),
    joined_channels_path: "".to_string(),
    data_channels_path: "".to_string(),
    users_coneccted_path: "".to_string(),
}));
    
    if let Ok(mut server) = server.write(){
        if let Ok(stream) = reader.try_clone() {
            if let Err(e) = server.add_neighbour(stream,neighbour_name){
                println!("Error:{:?}", e);
            }
        }
    }
    let mut new_port = String::from("127.0.0.1:");
    new_port.push_str(port.to_string().as_str());
    match manage_server(server, new_port.to_string()) {
        Err(error) => println!("Error: {}", error),
        _ => println!("Todo ok"),
    }
    
}