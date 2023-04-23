use rfc::answers::file_info_answer::FileInfoAnswer;
use rfc::answers::file_request_answer::FileRequestAnswer;
use rfc::answers::privmsg_answer::PrivmsgAnswer;
use rfc::answers::quit_answer::QuitAnswer;
use rfc::answers::start_dcc_answer::StartDCCAnswer;
use rfc::client_parser::*;
use rfc::comunicator::Comunicator;
use rfc::config_file::ConfigFile;
use rfc::file_map::FileMap;
use rfc::name_ip_map::NameIpMap;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::net::{Shutdown, SocketAddr, TcpListener, ToSocketAddrs};
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock};
// use std::time::{Duration, Instant};
use std::time::Duration;
use std::{io::Read, io::Write, net::TcpStream};
use std::{str, thread};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct P2PStruct {
    pub name: String,
    pub message: String,
}

// P2PCHAT juan :mensjae

pub fn init_client(ui_rx: Receiver<String>, ui_tx: glib::Sender<Answer>) {
    if let Ok(configf) = ConfigFile::new("./src/config_file".to_string()) {
        let address: String = if configf.server_type.as_str() == "MAIN" {
            configf.main_port
        } else {
            configf.secondary_port
        };

        if let Ok(mut stream) = TcpStream::connect(address) {
            let _result = stream.set_read_timeout(Some(Duration::new(0, 10000000)));

            if let Ok(stream_r) = stream.try_clone() {
                let mut reader = stream_r;
                escribir_mensaje(&mut stream, &mut reader, &ui_rx, &ui_tx);
            }
        } else if let Err(e) = ui_tx.send(Answer::ErrMsg(String::from("ERR_CANNOTCONNECTTOSERVER")))
        {
            println!("Error:{e:?}");
        }
    }
}

fn escribir_mensaje(
    writer: &mut TcpStream,
    reader: &mut TcpStream,
    ui_rx: &Receiver<String>,
    ui_tx: &glib::Sender<Answer>,
) {
    let mut comunicator = Comunicator::new();
    let mut name_ip_map = Arc::new(RwLock::new(NameIpMap::new()));
    let mut file_map = Arc::new(RwLock::new(FileMap::new()));
    loop {
        let d = Duration::from_nanos(50);
        let read = ui_rx.recv_timeout(d);
        if let Ok(mut message) = read {
            message.push('\r');
            message.push('\n');

            // // ES un DCC ???
            let index = message.as_str().find(':');
            if let Some(index) = index {
                let mut message_from_double_dots: String =
                    message[index + 1..message.len()].to_string();
                message_from_double_dots.pop();
                message_from_double_dots.pop(); // para eliminar el /r /n
                                                // let clone_message = message_from_double_dots.clone();

                // Si es p2p le enviamos el mensaje al hilo p2p correspondiente
                if let Some(p2pchat) = is_p2p_chat(message.clone(), index) {
                    let p2p_chat_name = format!("{} P2P", p2pchat.name);
                    if comunicator
                        .pass_message_to(p2p_chat_name, p2pchat.message)
                        .is_err()
                    {};
                    continue;
                }
                // si es send FILE ACCEPTED
                if let Some(message_parsed) = is_file_accepted(message.clone()) {
                    let destine = format!("{} SEND", message_parsed[1]);
                    if comunicator
                        .pass_message_to(destine, message_parsed[0].clone())
                        .is_err()
                        && ui_tx
                            .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                            .is_err()
                    {}
                    continue;
                }

                if is_dcc_command(message_from_double_dots.clone(), "DCC CHAT chat ", 14) {
                    // let ip_port = message_from_double_dots[14..].replace(' ', ":");
                    let mut port = 8010;
                    let parts: Vec<&str> = message_from_double_dots.split(' ').collect();
                    let ip = parts[3].to_owned();
                    //  PRIVMSG juan :DCC CHAT chat .....   <--- esto lo manda franco
                    let name = extract_name(&message, "PRIVMSG");
                    let new_p2p_chat = format!("{name} P2P");
                    let receiver = comunicator.new_comunication(new_p2p_chat);
                    let ui_tx_clone = ui_tx.clone();
                    loop {
                        let ip_port = format!("{ip}:{port}");
                        match TcpListener::bind(ip_port.as_str()) {
                            Ok(listener) => {
                                let _join_handler =
                                    thread::spawn(move || match listener.accept() {
                                        Ok((stream, _socket)) => {
                                            launch_dcc_listener(
                                                stream,
                                                receiver,
                                                ui_tx_clone,
                                                name,
                                            );
                                        }
                                        Err(_) => {
                                            if ui_tx_clone.send(Answer::P2PError(name)).is_err() {}
                                        }
                                    });
                                break;
                            }
                            Err(_) => port += 1,
                        }
                    }
                    // accept connections and process them serially

                    message = message_p2p_to_server(
                        message.clone(),
                        message_from_double_dots.clone(),
                        port.to_string().clone(),
                    )
                }
                // Este es el caso en el que yo quiero mandar un archivo
                if is_dcc_command(message_from_double_dots.clone(), "DCC SEND ", 9) {
                    // PRIVMSG name :DCC SEND pathabsoluto filename ip

                    // DCC SEND path_absoluto nombre_archivo ip
                    let file_info = get_file_info(&message_from_double_dots);

                    match get_ip_address(&message) {
                        Some(ip) => {
                            if let Ok(mut file_map) = file_map.write() {
                                file_map.add_path(file_info.2.as_str(), file_info.0.as_str());
                                // file_map.add_file(file_info.2.as_str(), file_info.1.as_str());
                            }
                            if let Ok(mut ip_map) = name_ip_map.write() {
                                ip_map.add_entry_file(file_info.2, ip.to_string());
                            }

                            let port = match send_file_over_tcp(file_info.0, 0, ip) {
                                Ok(port) => port,
                                Err(_) => {
                                    if ui_tx
                                        .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                                        .is_err()
                                    {};
                                    continue;
                                }
                            };

                            // el message al servidor va a tener el formato :
                            // PRIVMSG NICK :DCC SEND <filename><ip><port><file size>
                            message = message_send_to_server(
                                message,
                                message_from_double_dots,
                                port.to_string(),
                            );
                        }
                        None => {
                            if ui_tx
                                .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                                .is_err()
                            {}
                        }
                    }
                }

                // if is_dcc_chat(message_from_double_dots.clone()){
                // }
            } //franco sender y juan receiver

            //  PRIVMSG franco :franco;DCC CHAT chat .....  <--- esto recibe juan
            //  PRIVMSG franco :DCCCHAT close .....  <--- esto recibe juan
            match writer.write(message.as_bytes()) {
                Err(_) => println!("Fallo conexion con servidor"),
                Ok(_) => {
                    if writer.flush().is_err() {
                        println!("Error con flush")
                    }
                }
            }
        }

        // juancito quiere recibir un mensaje de franco
        let mut buffer = [0; 500];
        if let Ok(consumed) = reader.read(&mut buffer) {
            //let answer:&str = str::from_utf8(&buffer[..consumed]);
            if let Ok(answer) = str::from_utf8(&buffer[..consumed]) {
                if answer.is_empty()
                    && ui_tx
                        .send(Answer::Quit(QuitAnswer {
                            leave_message: String::from(""),
                        }))
                        .is_ok()
                {
                    break;
                }
                let answer_split = answer.split(' '); //["PRIVMSG","remitente","mensaje1","mensaje2","mensaje" n];
                let vec: Vec<String> = answer_split.into_iter().map(|a| a.to_string()).collect(); //esta agregamos por el clippy

                let answer_struct: Answer = parse_answer(vec);
                if let Answer::PrivMsg(privmsg) = answer_struct.clone() {
                    if is_dcc_command(privmsg.message.clone(), "DCC CHAT chat ", 14) {
                        // HABRIA QUE HACER UN NUEVO HASH QUE GUARDE LA IP
                        // HABRIA QUE TENER UNA ESTRUCTURA QUE GUARDE LOS NOMBRES DE LOS ARCHIVOS Y CUANTOS BYTES TIENEN DESCARGADOS
                        // JUAN P2P
                        let new_p2p = format!("{} P2P", privmsg.from_user.clone());
                        let rx = comunicator.new_comunication(new_p2p);
                        new_dcc_chat_request(
                            privmsg.message.clone(),
                            rx,
                            privmsg.from_user.clone(),
                            ui_tx.clone(),
                        ); // name seria juancito en este caso
                    }

                    if is_dcc_command(privmsg.message.clone(), "DCC CLOSE", 9) {
                        let close_p2p = format!("{} P2P", privmsg.from_user.clone());
                        if comunicator
                            .pass_message_to(close_p2p, privmsg.message.clone())
                            .is_err()
                        {};
                        continue;
                    }

                    // Esto es del lado que recibe la solicitud de que le quieren enviar un archivo
                    if is_dcc_command(privmsg.message.clone(), "DCC SEND ", 9) {
                        // guardar JUAN SEND
                        let new_send = format!("{} SEND", privmsg.from_user.clone());
                        let rx = comunicator.new_comunication(new_send);

                        new_dcc_send_request(
                            privmsg.message.clone(),
                            rx,
                            privmsg.from_user.clone(),
                            ui_tx.clone(),
                            &mut name_ip_map,
                            &mut file_map,
                        );
                    }

                    // Este accept es la confirmacion para seguir recibiendo el archivo que me habia quedado incompleto

                    // DCC ACCEPT
                    if is_dcc_command(privmsg.message.clone(), "DCC ACCEPT ", 11) {
                        let (filename, port, position) =
                            match file_name_port_position_parsed(privmsg.clone(), ui_tx.clone()) {
                                Ok((filename, port, position)) => (filename, port, position),
                                Err(_) => return,
                            };

                        if let Ok(name_ip_map) = name_ip_map.read() {
                            if let Ok(file_map) = file_map.read() {
                                match name_ip_map.get_ip(&privmsg.from_user) {
                                    Some(ip_addr) => {
                                        let socket_addr = SocketAddr::new(*ip_addr, port);
                                        match file_map.get_total_bytes(&filename) {
                                            Some(total_bytes) => {
                                                if read_tcp_to_file(
                                                    position,
                                                    filename,
                                                    socket_addr,
                                                    ui_tx.clone(),
                                                    total_bytes.to_string(),
                                                )
                                                .is_err()
                                                    && ui_tx
                                                        .send(Answer::SendError(
                                                            "ERR_NO_FILE_SENT".to_string(),
                                                        ))
                                                        .is_err()
                                                {
                                                };
                                            }
                                            None => {
                                                if ui_tx
                                                    .send(Answer::SendError(
                                                        "ERR_NO_FILE_SENT".to_string(),
                                                    ))
                                                    .is_err()
                                                {
                                                };
                                                continue;
                                            }
                                        }
                                    }
                                    None => {
                                        if ui_tx
                                            .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                                            .is_err()
                                        {};
                                        continue;
                                    }
                                }
                            }
                        }
                    }

                    // Esta solicitud llega para que yo siga enviando el archivo desde donde lo dejaron
                    // cuando lo recibo envio un ACCEPT con el puerto desde donde voy a estar mandando el archivo
                    // DCC RESUME filename port position
                    if is_dcc_command(privmsg.message.clone(), "DCC RESUME ", 11) {
                        let (filename, _port, start_position) =
                            match file_name_port_position_parsed(privmsg.clone(), ui_tx.clone()) {
                                Ok((filename, _port, start_position)) => {
                                    (filename, _port, start_position)
                                }
                                Err(_) => return,
                            };

                        if let Ok(file_map) = file_map.read() {
                            let file_path = match file_map.get_path(&filename) {
                                Some(file_path) => file_path,
                                None => {
                                    if ui_tx
                                        .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                                        .is_err()
                                    {};
                                    continue;
                                }
                            };

                            if let Ok(ip_map) = name_ip_map.read() {
                                if let Some(ip) = ip_map.get_file_ip(&filename) {
                                    let port = match send_file_over_tcp(
                                        file_path.to_string(),
                                        start_position,
                                        ip,
                                    ) {
                                        Ok(port) => port,
                                        Err(_) => {
                                            if ui_tx
                                                .send(Answer::SendError(
                                                    "ERR_NO_FILE_SENT".to_string(),
                                                ))
                                                .is_err()
                                            {
                                            };
                                            continue;
                                        }
                                    };

                                    let message = format!(
                                        "PRIVMSG {} :DCC ACCEPT {} {} {}\r\n",
                                        privmsg.from_user, filename, port, start_position
                                    );

                                    match writer.write(message.as_bytes()) {
                                        Err(_) => println!("Fallo conexion con servidor"),
                                        Ok(_) => {
                                            if writer.flush().is_err() {
                                                println!("Error con flush")
                                            }
                                        }
                                    }
                                }
                            }
                            //
                        }
                    }

                    // Esto lo recibe el que tiene qe mandar el archivo
                    // Posicion, Puerto, Ip? (no la tiene) pero la podemos guardar previamente
                    // Nombre Archivo
                    // Si esta todo ok -> mandar un dcc accept a traves de un privmsg
                    // if is_dcc_resume(){

                    // }

                    // Esto recibe un nombre un puerto, y una posicion, lo que hay que hacer es abrir el archivo
                    // (que ya existe) y empezar a leer del stream y escrbir lo leido a partir de posicion.
                    // Hay que conectarse nuevamente al puerto que nos indiquen
                    // if is_dcc_accept(){

                    // }
                }
                if ui_tx.send(answer_struct).is_err() {
                    println!("Fallo el send hacia la ui");
                }
            }
        }
    }
}

fn file_name_port_position_parsed(
    privmsg: PrivmsgAnswer,
    ui_tx: glib::Sender<Answer>,
) -> Result<(String, u16, u64), ()> {
    let filename: String = match parse_dcc_field(&privmsg.message, "filename") {
        Some(filename) => filename,
        None => {
            if ui_tx
                .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                .is_err()
            {};
            return Err(());
        }
    };

    let position: String = match parse_dcc_field(&privmsg.message, "position") {
        Some(position) => position,
        None => {
            if ui_tx
                .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                .is_err()
            {};
            return Err(());
        }
    };

    let port: String = match parse_dcc_field(&privmsg.message, "port") {
        Some(port) => port,
        None => {
            if ui_tx
                .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                .is_err()
            {};
            return Err(());
        }
    };

    let port: u16 = match port.parse() {
        Ok(port) => port,
        Err(_) => {
            if ui_tx
                .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                .is_err()
            {};
            return Err(());
        }
    };
    let position: u64 = match position.parse() {
        Ok(position) => position,
        Err(_) => {
            if ui_tx
                .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                .is_err()
            {};
            return Err(());
        }
    };
    Ok((filename, port, position))
}

// message : PRIVMSG nick :DCC SEND path_completo nombre_archivo ip tamaño_archivo
// message_From DOUBLE DOTS :   DCC SEND path_completo nombre_archivo ip tamaño_archivo
fn message_send_to_server(
    message: String,
    message_from_double_dots: String,
    port: String,
) -> String {
    let parts_message: Vec<&str> = message.split_whitespace().collect();
    let parts_message_from_double: Vec<&str> =
        message_from_double_dots.split_whitespace().collect();
    let destine = parts_message[1].to_owned();
    let filename = parts_message_from_double[3].to_owned();
    let ip = parts_message_from_double[4].to_owned();
    let tam = parts_message_from_double[5].to_owned();
    format!("PRIVMSG {destine} :DCC SEND {filename} {ip} {port} {tam}\r\n")
}

fn is_file_accepted(message: String) -> Option<Vec<String>> {
    let parts: Vec<&str> = message.split_whitespace().collect();
    if (parts[0] == "FILE_ACCEPTED") || (parts[0] == "FILE_DENIED") {
        let words: Vec<String> = message
            .split_whitespace()
            .into_iter()
            .map(|a| a.to_string())
            .collect();
        return Some(words);
    }
    None
}

// message : PRIVMSG nick :DCC CHAT chat ip
// message_From DOUBLE DOTS :   DCC CHAT chat ip
fn message_p2p_to_server(
    message: String,
    message_from_double_dots: String,
    port: String,
) -> String {
    let parts_message: Vec<&str> = message.split_whitespace().collect();
    let parts_message_from_double: Vec<&str> =
        message_from_double_dots.split_whitespace().collect();
    let destine = parts_message[1].to_owned();
    let ip = parts_message_from_double[3].to_owned();
    format!("PRIVMSG {destine} :DCC CHAT chat {ip} {port}\r\n")
}

fn get_ip_address(input_string: &str) -> Option<&str> {
    let prefix = "DCC SEND ";
    if let Some(index) = input_string.find(prefix) {
        let dcc_string = &input_string[index + prefix.len()..];
        let parts: Vec<&str> = dcc_string.split(' ').collect();
        return Some(parts[2]);
    }
    None
}

fn send_file_over_tcp(file_path: String, start_position: u64, ip: &str) -> std::io::Result<i32> {
    let mut initial_port = 8030;
    loop {
        let ip_port = format!("{ip}:{initial_port}");
        match TcpListener::bind(ip_port) {
            Ok(listener) => {
                let _join_handler: thread::JoinHandle<_> = thread::spawn(move || {
                    if let Ok((mut stream, _)) = listener.accept() {
                        if let Ok(mut file) = File::open(file_path) {
                            if file.seek(std::io::SeekFrom::Start(start_position)).is_err() {
                                println!("FileSeek failed");
                            }

                            let mut buffer = [0; 1024];
                            // Las lineas comentadas se usan para chequear el Dcc Resume, corta en un tiempo especifico que ponemos nosotros el stream.
                            // let mut last_read = Instant::now();
                            // let timeout = Duration::from_micros(20);
                            loop {
                                // let now = Instant::now();
                                // let start = Instant::now();
                                // if now.duration_since(last_read) > timeout {
                                //     println!("Timeout reached, closing stream.");
                                //     break;
                                // }

                                if let Ok(bytes_read) = file.read(&mut buffer) {
                                    if bytes_read == 0 {
                                        break;
                                    }
                                    if stream.write_all(&buffer[..bytes_read]).is_ok() {
                                        // println!("{bytes_read} bytes writes");
                                    };
                                }
                                // last_read = start;
                                // let _duration = start.elapsed();
                            }
                        }
                    }
                });

                return Ok(initial_port);
            }
            Err(_) => initial_port += 1,
        }
    }
}

fn launch_dcc_listener(
    mut stream: TcpStream,
    receiver: Receiver<String>,
    ui_tx: glib::Sender<Answer>,
    name: String,
) {
    let _result = stream.set_read_timeout(Some(Duration::new(0, 10000000)));
    loop {
        let mut buffer = [0; 500];
        // juan le envio un mensaje por stream a franco
        if let Ok(consumed) = stream.read(&mut buffer) {
            if let Ok(answer) = str::from_utf8(&buffer[..consumed]) {
                if answer.is_empty() {
                    continue;
                };
                if answer == "DCC CLOSE" {
                    break;
                }
                let param = vec!["P2PCHAT".to_string(), answer.to_string(), name.clone()];
                let answer_p2p = parse_answer(param);
                if ui_tx.send(answer_p2p).is_err() {};
            }
        }
        if let Ok(message) = receiver.recv_timeout(Duration::new(0, 10000000)) {
            if message == "DCC CLOSE" {
                let param = vec!["DCCCLOSE".to_string(), name.clone()];
                let answer_close_p2p = parse_answer(param);
                if ui_tx.send(answer_close_p2p).is_err() {};
                if stream.write_all(message.as_bytes()).is_err()
                    && ui_tx.send(Answer::P2PError(name)).is_err()
                {};
                if stream.shutdown(Shutdown::Both).is_err() {};
                break;
            }

            if stream.write_all(message.as_bytes()).is_err() {};
        }
    }
}
// P2PCHAT juan :mensjae
fn is_p2p_chat(message: String, index: usize) -> Option<P2PStruct> {
    if message.starts_with("P2PCHAT") {
        let name = extract_name(&message, "P2PCHAT");
        let message_from_user = &message[index + 1..message.len() - 2];
        return Some(P2PStruct {
            name,
            message: message_from_user.to_string(),
        });
    }

    None
}

fn extract_name(input: &str, first_word: &str) -> String {
    let mut name = String::new();
    let mut capturing_name = false;

    for word in input.split_whitespace() {
        if word.starts_with(first_word) {
            capturing_name = true;
            continue;
        }

        if capturing_name {
            name.push_str(word);
            name.push(' ');
            break;
        }
    }
    // Elimina el espacio final
    name.pop();
    name
}

fn is_dcc_command(message: String, command: &str, command_length: usize) -> bool {
    if message.len() < command_length {
        return false;
    }

    if &message[0..command_length] == command {
        return true;
    }
    false
}

fn new_dcc_chat_request(
    message: String,
    rx: Receiver<String>,
    name: String,
    ui_tx: glib::Sender<Answer>,
) {
    let _join_handler: thread::JoinHandle<_> = thread::spawn(move || {
        let ip_port = &message[14..].replace(' ', ":");
        let mut stream = match TcpStream::connect(ip_port.as_str()) {
            Ok(stream) => stream,
            Err(_) => {
                if ui_tx.send(Answer::P2PError(name.clone())).is_err() {};
                return;
            }
        };
        let _result = stream.set_read_timeout(Some(Duration::new(0, 10000000)));
        if ui_tx
            .send(Answer::StartDCC(StartDCCAnswer::new(vec![name.clone()])))
            .is_err()
        {};
        loop {
            let mut buffer = [0; 500];
            if let Ok(consumed) = stream.read(&mut buffer) {
                if let Ok(answer) = str::from_utf8(&buffer[..consumed]) {
                    if answer.is_empty() {
                        continue;
                    };
                    if answer == "DCC CLOSE" {
                        break;
                    }
                    let param = vec!["P2PCHAT".to_string(), answer.to_string(), name.clone()];
                    let answer_p2p = parse_answer(param);
                    if ui_tx.send(answer_p2p).is_err() {};
                }
            }

            if let Ok(message) = rx.recv_timeout(Duration::new(0, 10000000)) {
                // Recibe el mensaje para cerrar la conexion
                if message == "DCC CLOSE" {
                    let param = vec!["DCCCLOSE".to_string(), name.clone()];
                    let answer_close_p2p = parse_answer(param);
                    if ui_tx.send(answer_close_p2p).is_err() {};
                    if stream.write_all(message.as_bytes()).is_err() {
                        if ui_tx.send(Answer::P2PError(name.clone())).is_err() {};
                        break;
                    };
                    if stream.shutdown(Shutdown::Both).is_err() {};
                    break;
                }

                if stream.write_all(message.as_bytes()).is_err() {
                    if ui_tx.send(Answer::P2PError(name.clone())).is_err() {};
                    break;
                };
            }
        }
    });
    //

    // RESUME

    //start_dcc_chat(ip_port.clone(), arc);
    // if dcc_chat_accepted(){

    // }
}

fn new_dcc_send_request(
    message: String,
    rx: Receiver<String>,
    name: String,
    ui_tx: glib::Sender<Answer>,
    name_ip_map: &mut Arc<RwLock<NameIpMap>>,
    file_map: &mut Arc<RwLock<FileMap>>,
) {
    // Aca se debe enviar un Answer con la informacion del nombre del archivo, tamanio archivo para que el cliente decida si aceptarlo desde la ui
    let file_info = get_file_info(&message);

    // Aca se manda la solicitud a  la ui para saber si el usuario quiere recibir el archivo
    if ui_tx
        .send(rfc::client_parser::Answer::FileRequest(FileRequestAnswer {
            file_name: file_info.0.clone(),
            file_size: file_info.1.clone(),
            file_owner: name.clone(),
        }))
        .is_err()
    {
        return;
    };
    // Aca se tiene que recibir si la solicitud es aceptada o no
    let name_ip_clone = name_ip_map.clone();
    let file_map = file_map.clone();
    let ui_tx_clone = ui_tx.clone();
    let _join_handler: thread::JoinHandle<_> = thread::spawn(move || {
        let response = match rx.recv() {
            Ok(response) => response,
            Err(_) => {
                if ui_tx_clone
                    .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                    .is_err()
                {};
                return;
            }
        };

        if response == "FILE_ACCEPTED" {
            if let Ok(socket_addres) = get_socket_addr(&message) {
                if let Ok(mut name_ip_map) = name_ip_clone.write() {
                    if let Ok(mut file_map) = file_map.write() {
                        name_ip_map.add_entry(name, socket_addres.ip());
                        file_map.add_file(file_info.0.as_str(), file_info.1.as_str());
                        if read_tcp_to_file(0, file_info.0, socket_addres, ui_tx, file_info.1)
                            .is_err()
                        {};
                    }
                }
            }
        }
    });
    //DCC SEND prueba 127.0.0.1 8091 242
}

fn read_tcp_to_file(
    pos: u64,
    filename: String,
    socket_addr: SocketAddr,
    ui_tx: glib::Sender<Answer>,
    total_bytes: String,
) -> std::io::Result<()> {
    let _join_handler: thread::JoinHandle<_> = thread::spawn(move || {
        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename.clone())
        {
            Ok(file) => file,
            Err(_) => {
                if ui_tx
                    .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                    .is_err()
                {};
                return;
            }
        };

        if file.seek(SeekFrom::Start(pos)).is_err() {
            if ui_tx
                .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                .is_err()
            {};
            return;
        };

        let mut stream = match TcpStream::connect(socket_addr) {
            Ok(stream) => stream,
            Err(_) => {
                if ui_tx
                    .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                    .is_err()
                {};
                return;
            }
        };
        let bytes_written = match std::io::copy(&mut stream, &mut file) {
            Ok(bytes_written) => bytes_written,
            Err(_) => {
                if ui_tx
                    .send(Answer::SendError("ERR_NO_FILE_SENT".to_string()))
                    .is_err()
                {};
                return;
            }
        };

        //==
        if let Ok(bytes_parsed) = total_bytes.parse::<u64>() {
            if ui_tx
                .send(rfc::client_parser::Answer::FileInfo(FileInfoAnswer {
                    file_name: filename,
                    bytes_transfered: pos + bytes_written,
                    bytes_total: bytes_parsed,
                }))
                .is_err()
            {};
        }
    });

    Ok(())
}

fn get_socket_addr(input: &str) -> Result<std::net::SocketAddr, ()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() != 6 || parts[0] != "DCC" || parts[1] != "SEND" {
        return Err(());
    }
    let address = parts[3];
    if let Ok(port) = u16::from_str(parts[4]) {
        if let Some(socket_addr) = (address, port).to_socket_addrs().map_err(|_| ())?.next() {
            return Ok(socket_addr);
        }
    }
    Err(())
}

//PRIVMSG nick :DCC SEND path_completo nombre_archivo ip tamaño_archivo
fn get_file_info(input: &str) -> (String, String, String) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let parts_without_spaces: Vec<String> = parts.iter().map(|s| s.replace('*', " ")).collect();
    // DCC SEND PATH_ABSOLUTO FILENAME IP TAM

    let mut file_path = parts_without_spaces[2].to_owned();
    file_path.pop();
    file_path.remove(0);
    // saco las comillas
    let file_size = parts_without_spaces[5].to_owned();
    let mut file_name = parts_without_spaces[3].to_owned();
    file_name.pop();
    file_name.remove(0);

    (file_path, file_size, file_name)
}

fn parse_dcc_field(s: &str, field: &str) -> Option<String> {
    let parts: Vec<&str> = s.split_whitespace().collect();

    if parts[0] != "DCC" || (parts[1] != "ACCEPT" && parts[1] != "RESUME") {
        // The string doesn't start with "DCC ACCEPT" or "DCC RESUME"
        return None;
    }

    let field_index = match field {
        "filename" => 3,
        "port" => 4,
        "position" => 5,
        _ => return None, // Unknown field name
    };

    // Get the nth part of the string, where n is the index of the field we want
    Some(parts[field_index - 1].to_string())
}
