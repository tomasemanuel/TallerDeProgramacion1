use crate::channel_list::ChannelList;
use crate::channels::Channels;
use crate::datauser::DataUserFile;
use crate::messages::away::away;
use crate::messages::mode::spread_mode_neighbors;
use crate::messages::nick::set_nick_name;
use crate::messages::oper::set_operator;
use crate::messages::pass::set_password;
use crate::messages::private_message::send_private_message;
use crate::messages::quit::{quit_message, spread_quit_neighbors};
use crate::messages::send::*;
use crate::messages::server_msg::new_server_request;
use crate::messages::server_quit::squit_request;
use crate::messages::server_quit_request::quit_server;
use crate::messages::shut::shut_connection_from_server;
use crate::messages::user::set_user;
use crate::mylines::MyLines;
use crate::parser::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self};

use crate::data_user_file_tcp::*;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::Arc;

use std::collections::HashMap;
use std::io::Write;
use std::sync::RwLock;

use crate::data_base::DataBase;

//static DATA_CHANNELS_PATH: &str = "./src/data_channels";
//static USERS_CONNECTED_PATH: &str = "./src/users_connected";

#[derive(Debug)]
pub struct Server {
    pub name: String,
    pub neighbours: HashMap<String, TcpStream>,
    pub password: String,
    pub data_base: Option<Arc<DataBase>>,
    pub data_file_path: String,
    pub joined_channels_path: String,
    pub data_channels_path: String,
    pub users_coneccted_path: String,
}

impl Server {
    /// agrega un nuevo vecino (server) a la base de datos de los vecinos de un servidor
    pub fn add_neighbour(&mut self, stream: TcpStream, server_name: String) -> Result<(), String> {
        if self.neighbours.contains_key(&server_name) {
            return Err(String::from("SERVER_NAME_ALREADY_REGISTERED"));
        }
        self.neighbours.entry(server_name).or_insert(stream);
        Ok(())
    }
}

/// funcion principal del servidor, se conecta a traves del puerto que se le pasa por parametro
pub fn manage_server(server: Arc<RwLock<Server>>, port: String) -> Result<(), String> {
    match TcpListener::bind(port) {
        Err(_) => return Err(String::from("Error con el bind")),
        Ok(listener) => obtain_connections(listener, server)?,
    }
    Ok(())
}

/// Escucha a sus vecinos cuando se esparce un mensaje (con el formato de IRC, es por ello que se utiliza el parser) con el prefijo,
///  en el caso de Nick y user no se tiene que usar el prefijo ya que no existe en la base de datos.
fn listen_neighbours(
    server_lock: Arc<RwLock<Server>>,
    data_base: Arc<DataBase>,
    tx: &Sender<DataUserFileTcpStream>,
    channels: Arc<Channels>,
) {
    if let Ok(server) = server_lock.read() {
        for neighbour in &server.neighbours {
            if let Ok(stream) = neighbour.1.try_clone() {
                let server_clone = server_lock.clone();
                let tx1 = tx.clone();
                let db1 = data_base.clone();
                let channels1 = channels.clone();
                let mut data_user = DataUserFile::default_for_clients();
                let _join_handler: thread::JoinHandle<_> = thread::spawn(move || {
                    let reader = BufReader::new(&stream); // Aca se pasa una referencia del stream para leer las linesa
                    let iter = MyLines::new(reader);
                    for line in iter.flatten() {
                        // println!("llega el mensaje:{}", line);
                        if let Ok(cmd) = parser(line) {
                            if let Ok(stream_c) = stream.try_clone() {
                                if let Some(nickname) = cmd.prefix.clone() {
                                    match cmd.cmd.clone() {
                                        Message::Nick(_nickinfo) => println!(),
                                        Message::Password(_pass_info) => {
                                            data_user = DataUserFile::default_for_clients()
                                        }
                                        Message::User(_user_info) => println!(),
                                        _ => {
                                            if let Ok(users_registered) = db1.data_registered.read()
                                            {
                                                if let Some(data_user_prefix) =
                                                    users_registered.get(&nickname)
                                                {
                                                    data_user = data_user_prefix.clone();
                                                }
                                            }
                                        }
                                    };
                                    match match_message(
                                        cmd.clone(),
                                        &mut data_user,
                                        &tx1.clone(),
                                        stream_c,
                                        channels1.clone(),
                                        db1.clone(),
                                        server_clone.clone(),
                                    ) {
                                        Ok(_string) => {
                                            if let Message::Nick(_nickinfo) = cmd.cmd.clone() {
                                                data_user = DataUserFile::default_for_clients();
                                            }
                                        }
                                        Err(e) => println!("ERR0R:{e:?}"),
                                    }
                                }
                            }
                        }
                        println!("hay error con el parser")
                    }
                });
            }
        }
    }

    let _join_handler: thread::JoinHandle<_> = thread::spawn(move || {});
}

/// A partir de un nuevo registro o un logeo de un usuario caido, se esparce el mensaje de registro a todos los servidores conectados
pub fn spread_register_neighbors(
    data_user: DataUserFile,
    flag_register: bool,
    server: Arc<RwLock<Server>>,
    server_sender: &TcpStream,
    prefix: &Option<String>,
) -> Result<(), String> {
    if let Ok(server) = server.write() {
        for mut server in &server.neighbours {
            if let Ok(socket_address_receiver) = server.1.peer_addr() {
                if let Ok(socket_address_sender) = server_sender.peer_addr() {
                    if socket_address_receiver == socket_address_sender {
                        continue;
                    }
                    let mut message = String::from(":");
                    match prefix {
                        Some(nickname) => message.push_str(nickname),
                        None => message.push_str(&data_user.nickname),
                    }
                    message.push_str(" PASS ");
                    message.push_str(&data_user.password);
                    message.push_str("\r\n");
                    if server.1.write(message.as_bytes()).is_err() {
                        return Err(String::from("Neighbour write failed"));
                    }

                    if flag_register {
                        let mut message = String::from(":");
                        match prefix {
                            Some(nickname) => message.push_str(nickname),
                            None => message.push_str(&data_user.nickname),
                        }
                        message.push_str(" USER ");
                        message.push_str(&data_user.username);
                        message.push(' ');
                        message.push_str(&data_user.hostname);
                        message.push_str("\r\n");

                        if server.1.write(message.as_bytes()).is_err() {
                            return Err(String::from("Neighbour write failed"));
                        }
                    }
                    let mut message = String::from(":");
                    match prefix {
                        Some(nickname) => message.push_str(nickname),
                        None => message.push_str(&data_user.nickname),
                    }
                    message.push_str(" NICK ");
                    message.push_str(&data_user.nickname);
                    message.push_str("\r\n");
                    if server.1.write(message.as_bytes()).is_err() {
                        return Err(String::from("Neighbour write failed"));
                    }

                    continue;
                }
                return Err(String::from("Error la socket_address"));
            }

            return Err(String::from("Error no se pudo sacar la socket_address"));
        }
        return Ok(());
    }
    Err(String::from("Error con el write"))
    // }
    // Ok(())
}

/// Inicializa la base de datos (USuarios registrados) del hash, esta base de datos contendra un hashmap
/// con el nickname de contraseña con clave el datauser file asociado
fn initialize_data_base_hash(
    server: Arc<RwLock<Server>>,
) -> Result<HashMap<String, DataUserFile>, String> {
    if let Ok(server) = server.read() {
        let file: File = match File::open(server.data_file_path.clone()) {
            Ok(data_file) => data_file,
            Err(_) => return Err(String::from("no encontro el archivo de la base de datos")),
        };

        let reader = BufReader::new(file);
        let mut data_hash = HashMap::new();

        for line in reader.lines() {
            if let Ok(linea) = line {
                let split_line = linea.split(','); // en nuestro database lo separamos como nick,contraseña,user,etc    el nick va a ser la clave del usuario
                let vector_split = split_line.collect::<Vec<&str>>();
                let clave_nick = vector_split[0].to_string();
                let data_user = DataUserFile::new(vector_split); // tiene que ser mutable para despues
                data_hash.entry(clave_nick).or_insert(data_user);
            } else {
                return Err(String::from(
                    "Hubo un error con la linea del archivo para la base de datos",
                ));
            }
        }
        return Ok(data_hash);
    }
    Err(String::from("DATA FILE READ FAILED"))
}

/// A partir del path propuesto en el config file, se escribe en dicho archivo los usuarios conectados en todos los servidores
fn write_connected_on_all_servers(
    data_base: &Arc<DataBase>,
    server: Arc<RwLock<Server>>,
    flag_first_time: bool,
) -> Result<(), String> {
    if let Ok(server) = server.read() {
        let mut f_users_connected = match flag_first_time {
            true => File::open(server.users_coneccted_path.clone()).expect("Unable to create file"),

            false => {
                File::create(server.users_coneccted_path.clone()).expect("Unable to create file")
            }
        };
        match data_base.data_connected_all_servers.write() {
            Ok(users_on_all_servers) => {
                let mut string_a_mandar = String::from("");
                for user in users_on_all_servers.clone() {
                    string_a_mandar.push_str(&user);
                    string_a_mandar.push(',');
                }
                if !string_a_mandar.is_empty() {
                    string_a_mandar.pop();
                }
                f_users_connected
                    .write_all(string_a_mandar.as_bytes())
                    .expect("Unable to write data");
                return Ok(());
            }
            Err(_) => {
                return Err(String::from(
                    "hubo un error en el write de la base de datos",
                ))
            }
        }
    }
    Err(String::from("SERVER READ FAILED"))
}
/// A partir de todos los paths de las bases de datos, se escriben en dichos archivos las bases de datos
/// para poder tener una persistencia de datos
fn write_database_on_file(
    data_base: &Arc<DataBase>,
    channels: &Arc<Channels>,
    server_l: Arc<RwLock<Server>>,
) -> Result<(), String> {
    if let Ok(server) = server_l.read() {
        let f_data_file =
            File::create(server.data_file_path.clone()).expect("Unable to create file");
        let f_joined_channels =
            File::create(server.joined_channels_path.clone()).expect("Unable to create file");
        let f_channels_data_base =
            File::create(server.data_channels_path.clone()).expect("Unable to create file");
        match data_base.data_registered.read() {
            Ok(hash) => {
                let hash_read = hash.clone();
                // let hash_read = hash_clone(hash)?;
                write_on_data_file(hash_read, f_data_file)?;
                match channels.joined_channels.read() {
                    Ok(joined_channels) => {
                        write_on_joined_channels(joined_channels.clone(), f_joined_channels)?
                    }
                    Err(_) => {
                        return Err(String::from(
                            "Hubo un error leyendo el hash_de la base de datos de joined channels",
                        ))
                    }
                }
                match channels.data_base.read() {
                    Ok(channels_data_base) => write_on_channels_data_base(
                        channels_data_base.clone(),
                        f_channels_data_base,
                    )?,
                    Err(_) => {
                        return Err(String::from(
                            "Hubo un error leyendo el hash_de la base de datos de joined channels",
                        ))
                    }
                }
                write_connected_on_all_servers(data_base, server_l.clone(), false)?;
                return Ok(());
            }
            Err(_) => {
                return Err(String::from(
                    "Hubo un error leyendo el hash_de la base de datos",
                ))
            }
        }
    }
    Err(String::from("SERVER READ ERROR"))
}

/// A partir del path de channels_data_base, se escribe en el path la base de datos con un formato que se pueda leer facilmente por el server
fn write_on_channels_data_base(
    channels_data_base: HashMap<String, ChannelList>,
    mut f_channels_data_base: File,
) -> Result<(), String> {
    let comma: String = String::from(",");
    let jump = String::from("\n");
    for (nickname, channel_list) in channels_data_base.into_iter() {
        f_channels_data_base
            .write_all(nickname.as_bytes())
            .expect("Unable to write data");
        f_channels_data_base
            .write_all(comma.as_bytes())
            .expect("Unable to write data");
        let mut invite_string = String::from("");
        match channel_list.invited_list {
            Some(invite_list) => {
                for invite in invite_list {
                    invite_string.push_str(invite.as_str());
                    invite_string.push(';');
                }
                if !invite_string.is_empty() {
                    invite_string.pop();
                }
            }
            None => invite_string = String::from("None"),
        }
        let joined_string = change_vec_to_string(channel_list.joined_list);

        let operator_string = change_vec_to_string(channel_list.operators);

        let topic_string = match channel_list.topic {
            Some(topic) => topic,
            None => String::from("None"),
        };
        let ban_string = match channel_list.ban_mask {
            Some(ban_mask) => change_vec_to_string(ban_mask),
            None => String::from("None"),
        };
        let secret_string = change_bool_to_string(channel_list.secret);
        let private_string = change_bool_to_string(channel_list.private);

        let vector: Vec<String> = vec![
            invite_string,
            joined_string,
            operator_string,
            topic_string,
            ban_string,
            secret_string,
            private_string,
        ];
        for atribute in vector.iter() {
            f_channels_data_base
                .write_all(atribute.as_bytes())
                .expect("Unable to write data");
            f_channels_data_base
                .write_all(comma.as_bytes())
                .expect("Unable to write data");
        }
        f_channels_data_base
            .write_all(jump.as_bytes())
            .expect("Unable to write data");
    }
    Ok(())
}

fn change_bool_to_string(bool: bool) -> String {
    if bool {
        return String::from("true");
    }
    String::from("false")
}

pub fn change_vec_to_string(vec: Vec<String>) -> String {
    let mut new_string = String::from("");
    for string in vec {
        new_string.push_str(string.as_str());
        new_string.push(';');
    }
    if !new_string.is_empty() {
        new_string.pop();
    }
    new_string
}
/// A partir del path de joined_channels, se escribe en el path la base de datos con un formato que se pueda leer facilmente por el server
fn write_on_joined_channels(
    joined_channels: HashMap<String, Vec<String>>,
    mut f_joined_channels: File,
) -> Result<(), String> {
    let comma: String = String::from(",");
    let jump = String::from("\n");
    for (nickname, joined_channels) in joined_channels.into_iter() {
        f_joined_channels
            .write_all(nickname.as_bytes())
            .expect("Unable to write data");
        f_joined_channels
            .write_all(comma.as_bytes())
            .expect("Unable to write data");
        let mut string_canales = String::from("");
        for channel in joined_channels {
            string_canales.push_str(&channel);
            string_canales.push(';');
        }
        if !string_canales.is_empty() {
            string_canales.pop();
        }
        f_joined_channels
            .write_all(string_canales.as_bytes())
            .expect("Unable to write data");
        f_joined_channels
            .write_all(jump.as_bytes())
            .expect("Unable to write data");
    }
    Ok(())
}
/// A partir del path de data_file, se escribe en el path la base de datos con un formato que se pueda leer facilmente por el server
fn write_on_data_file(
    hash_read: HashMap<String, DataUserFile>,
    mut f_data_file: File,
) -> Result<(), String> {
    for (_nickname, data_user_file) in hash_read.into_iter() {
        let vector: Vec<String> = vec![
            data_user_file.nickname,
            data_user_file.nickname_actualizado,
            data_user_file.password,
            data_user_file.username,
            data_user_file.hostname,
            data_user_file.servername,
            data_user_file.realname,
        ];
        let comma: String = String::from(",");
        for atribute in vector.iter() {
            f_data_file
                .write_all(atribute.as_bytes())
                .expect("Unable to write data");
            f_data_file
                .write_all(comma.as_bytes())
                .expect("Unable to write data");
        }
        let jump = String::from("\n");
        f_data_file
            .write_all(jump.as_bytes())
            .expect("Unable to write data");
    }
    Ok(())
}
/// A partir del path de los usuarios conectados, actualizamos en la base de datos con el contenido de ese archivo
pub fn update_users_connected_all_servers(
    data_base: Arc<DataBase>,
    server: Arc<RwLock<Server>>,
) -> Result<(), String> {
    if let Ok(mut users_connected_all_servers) = data_base.data_connected_all_servers.write() {
        users_connected_all_servers.clear();
        if let Ok(server) = server.read() {
            let file: File = match File::open(server.users_coneccted_path.clone()) {
                Ok(users_connected_file) => users_connected_file,
                Err(_) => return Ok(()),
            };
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if line == *"" {
                    return Ok(());
                }
                let split_line = line.split(',');
                let vector_split = split_line.collect::<Vec<&str>>();
                for user in vector_split {
                    users_connected_all_servers.push(user.to_string());
                }
            }
        }
        return Ok(());
    }
    Err(String::from("No se puede escribir el database"))
}

/// A partir del path de los joined_channels, actualizamos en la base de datos con el contenido de ese archivo
pub fn update_channels_joined_channels_data_base(
    channels: Arc<Channels>,
    server: Arc<RwLock<Server>>,
) -> Result<(), String> {
    if let Ok(server) = server.read() {
        if let Ok(mut joined_channels) = channels.joined_channels.write() {
            joined_channels.drain();

            let file: File = match File::open(server.joined_channels_path.clone()) {
                Ok(joined_channel_file) => joined_channel_file,
                Err(_) => return Ok(()),
            };

            let reader = BufReader::new(file);

            for line in reader.lines() {
                if let Ok(linea) = line {
                    if linea == *"" {
                        return Ok(());
                    }
                    let split_line = linea.split(',');
                    let vector_split = split_line.collect::<Vec<&str>>();
                    if vector_split.is_empty() {
                        return Ok(());
                    }
                    let clave_nick = vector_split[0].to_string();

                    let joined_channels_vec: Vec<String> = vector_split[1]
                        .split(';')
                        .map(|channel| channel.to_string())
                        .collect();

                    joined_channels
                        .entry(clave_nick)
                        .or_insert(joined_channels_vec);
                } else {
                    return Err(String::from(
                        "Hubo un error con la linea del archivo para la base de datos",
                    ));
                }
            }
            return Ok(());
        }
    }
    Err(String::from("No deja ver el channel"))
}

/// A partir del path de los channels_data_base, actualizamos en la base de datos con el contenido de ese archivo
pub fn update_channels_data_base(
    channels: Arc<Channels>,
    server: Arc<RwLock<Server>>,
) -> Result<(), String> {
    if let Ok(mut data_base) = channels.data_base.write() {
        data_base.drain();

        if let Ok(server) = server.read() {
            let file: File = match File::open(server.data_channels_path.clone()) {
                Ok(data_channels_file) => data_channels_file,
                Err(_) => return Err(String::from("no encontro el archivo del data_channels")),
            };

            let reader = BufReader::new(file);

            for line in reader.lines() {
                if let Ok(linea) = line {
                    if linea == *"" {
                        return Ok(());
                    }
                    let split_line = linea.split(',');
                    let vector_split = split_line.collect::<Vec<&str>>();
                    if vector_split.is_empty() {
                        return Ok(());
                    }
                    let clave_canal = vector_split[0].to_string();

                    let channel_list = ChannelList::new_with(vector_split);

                    data_base.entry(clave_canal).or_insert(channel_list);
                } else {
                    return Err(String::from(
                        "Hubo un error con la linea del archivo para la base de datos",
                    ));
                }
            }
            return Ok(());
        }
    }
    Err(String::from("No deja writear el database del channel"))
}

/// Se inicializa el thread del main, aca se va a recibir la data que se quiera actualizar, despues de eso se escribe en los archivos.
fn launch_main_thread(
    data_base: &mut Arc<DataBase>,
    rx: Receiver<DataUserFileTcpStream>,
    mut channels: Arc<Channels>,
    server: Arc<RwLock<Server>>,
) -> Result<(), String> {
    let mut data_base_clone = data_base.clone();
    let _join_handler: thread::JoinHandle<_> = thread::spawn(move || loop {
        match rx.recv() {
            Ok(data) => {
                if let Some(data_user_file) = data.data_file {
                    if let Some(stream) = data.stream {
                        if update_data_base(
                            data_user_file,
                            stream,
                            &mut channels,
                            &mut data_base_clone,
                            data.prefix,
                        )
                        .is_err()
                        {
                            println!("DATABASE WRITE ERROR");
                        }
                    }
                }

                if write_database_on_file(&data_base_clone, &channels, server.clone()).is_err() {
                    println!("DATABASE WRITE ERROR");
                }
            }
            Err(_) => println!("Error "),
        }
    });
    Ok(())
}

// Se inicializa el thread del cliente,  y se llama a la funcion para manejar a cada cliente.
fn launch_client_handler_threads(
    listener: TcpListener,
    tx: &Sender<DataUserFileTcpStream>,
    channels: Arc<Channels>,
    data_base: Arc<DataBase>,
    server: Arc<RwLock<Server>>,
) -> Result<(), String> {
    for cliente in listener.incoming() {
        let channel_clone = channels.clone();
        let data_base_clone = data_base.clone();
        let tx1 = mpsc::Sender::clone(tx);

        let server_ref = server.clone();
        let _join_handle: thread::JoinHandle<_> = thread::spawn(move || {
            match cliente {
                // mover el match para arriba de todo
                Err(_) => return Err(String::from("Error con el cliente")),
                Ok(cliente_tcp) => {
                    handle_client(cliente_tcp, tx1, channel_clone, data_base_clone, server_ref)?
                }
            };
            Ok(())
        });
    }

    Ok(())
}

/// Se obtienen las conexiones de los clientes, una vez que las tenemos, llamamos a las funciones de main_thread y client_handler_thread
fn obtain_connections(listener: TcpListener, server: Arc<RwLock<Server>>) -> Result<(), String> {
    //println!("Hizo el bind?");
    let db_lock = Arc::new(RwLock::new(initialize_data_base_hash(server.clone())?));

    let data_connected: Arc<RwLock<HashMap<String, TcpStream>>> =
        Arc::new(RwLock::new(HashMap::new())); // nick: TcpStream
    let server_operators: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));

    let channels = Channels::default(Some(server.clone()))?;
    let channels_clone = channels.clone();

    let data_connected_all_servers: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
    let mut data_base = Arc::new(DataBase {
        data_registered: db_lock,
        data_connected,
        data_connected_all_servers,
        operators: server_operators,
    });
    if write_connected_on_all_servers(&data_base, server.clone(), true).is_err() {
        return Err("WRITE TO ALL SERVERS FAILED".to_string());
    }
    if let Ok(mut server) = server.write() {
        server.data_base = Some(data_base.clone());
    }

    let (tx, rx) = mpsc::channel();

    listen_neighbours(server.clone(), data_base.clone(), &tx, channels.clone());

    launch_main_thread(&mut data_base, rx, channels_clone, server.clone())?;
    launch_client_handler_threads(listener, &tx, channels, data_base, server)
}

/// Funcion que actualiza la base de datos, hay diferentes casos en los cuales se tienen que actualizar, todo debidamente comentado.
fn update_data_base(
    data_user_file: DataUserFile,
    stream: TcpStream,
    channels: &mut Arc<Channels>,
    data_base: &mut Arc<DataBase>,
    prefix: Option<String>,
) -> Result<(), String> {
    match data_base.data_registered.write() {
        Ok(mut data_registered) => {
            match data_base.data_connected.write() {
                Ok(mut data_connected) => {
                    match data_base.data_connected_all_servers.write() {
                        Ok(mut data_connected_all_servers) => {
                            // INICIO ya registrado o CAMbio de datos
                            if data_user_file.nickname_actualizado == "same"
                                && data_registered.contains_key(&data_user_file.nickname)
                            {
                                // caso de cambio de cosas, pisar los datos actuales!!
                                let mut data_user_file_new = DataUserFile::default_for_clients();
                                data_user_file_new.clone_data_user(&data_user_file);
                                let nickname_registrado = data_user_file.nickname.clone();
                                data_registered.insert(data_user_file.nickname, data_user_file_new);

                                if prefix.is_none() {
                                    data_connected
                                        .entry(nickname_registrado.clone())
                                        .or_insert(stream);
                                }
                                data_connected_all_servers.push(nickname_registrado);
                                return Ok(());
                            }

                            let mut data_user_file_new = DataUserFile::default_for_clients();
                            data_user_file_new.clone_data_user(&data_user_file);

                            // CAMBIO DE NICKNAME
                            if data_user_file.nickname_actualizado != "same" {
                                data_registered.remove(&data_user_file.nickname);
                                if prefix.is_none() {
                                    data_connected.remove(&data_user_file.nickname);
                                }
                                if data_connected_all_servers.contains(&data_user_file.nickname) {
                                    data_connected_all_servers
                                        .retain(|users| users != &data_user_file.nickname);
                                    data_connected_all_servers
                                        .push(data_user_file.nickname_actualizado.clone());
                                }
                                match data_base.operators.write() {
                                    Ok(mut server_operators) => {
                                        if server_operators.contains(&data_user_file.nickname) {
                                            server_operators.retain(|operators| {
                                                operators != &data_user_file.nickname
                                            });
                                            server_operators
                                                .push(data_user_file.nickname_actualizado.clone());
                                        }
                                    }
                                    Err(_) => {
                                        return Err(String::from(
                                            "no pudo escribir en server operators",
                                        ))
                                    }
                                }
                                match channels.joined_channels.write() {
                                    Ok(mut joined_channels) => {
                                        if let Some(channels_name) =
                                            joined_channels.remove(&data_user_file.nickname)
                                        {
                                            joined_channels
                                                .entry(data_user_file.nickname_actualizado.clone())
                                                .or_insert(channels_name);
                                        }

                                        match channels.data_base.write() {
                                            Ok(mut hash_channels) => {
                                                let hash_clone = hash_channels.clone();
                                                for (channel, channel_list) in
                                                    hash_clone.into_iter()
                                                {
                                                    let mut joined_list_clone =
                                                        channel_list.joined_list.clone();
                                                    if channel_list
                                                        .joined_list
                                                        .contains(&data_user_file.nickname)
                                                    {
                                                        joined_list_clone.retain(|names| {
                                                            names != &data_user_file.nickname
                                                        });
                                                        joined_list_clone.push(
                                                            data_user_file
                                                                .nickname_actualizado
                                                                .clone(),
                                                        );
                                                    }

                                                    let channel_list_clone = match channel_list
                                                        .invited_list
                                                    {
                                                        Some(invited_list) => {
                                                            let mut invited_list_clone =
                                                                invited_list.clone();
                                                            if invited_list
                                                                .contains(&data_user_file.nickname)
                                                            {
                                                                invited_list_clone.retain(
                                                                    |names| {
                                                                        names
                                                                            != &data_user_file
                                                                                .nickname
                                                                    },
                                                                );
                                                                invited_list_clone.push(
                                                                    data_user_file
                                                                        .nickname_actualizado
                                                                        .clone(),
                                                                );
                                                            }
                                                            ChannelList {
                                                                joined_list: joined_list_clone,
                                                                invited_list: Some(
                                                                    invited_list_clone,
                                                                ),
                                                                operators: channel_list
                                                                    .operators
                                                                    .clone(),
                                                                topic: channel_list.topic.clone(),
                                                                ban_mask: channel_list
                                                                    .ban_mask
                                                                    .clone(),
                                                                secret: channel_list.secret,
                                                                private: channel_list.private,
                                                            }
                                                        }
                                                        None => ChannelList {
                                                            joined_list: joined_list_clone,
                                                            invited_list: None,
                                                            operators: channel_list
                                                                .operators
                                                                .clone(),
                                                            topic: channel_list.topic.clone(),
                                                            ban_mask: channel_list.ban_mask.clone(),
                                                            secret: channel_list.secret,
                                                            private: channel_list.private,
                                                        },
                                                    };
                                                    hash_channels.remove(&channel);
                                                    hash_channels
                                                        .entry(channel)
                                                        .or_insert(channel_list_clone);
                                                }
                                            }
                                            Err(_) => {
                                                return Err(String::from(
                                                    "no pudo escribir en server operators",
                                                ))
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        return Err(String::from(
                                            "no pudo escribir en joined channels",
                                        ))
                                    }
                                }

                                data_user_file_new.nickname =
                                    data_user_file.nickname_actualizado.clone();
                                data_user_file_new.nickname_actualizado = String::from("same");

                                let nickname_nuevo = data_user_file_new.nickname.clone();

                                data_registered
                                    .entry(data_user_file_new.nickname.clone())
                                    .or_insert(data_user_file_new);
                                if prefix.is_none() {
                                    data_connected.entry(nickname_nuevo).or_insert(stream);
                                }
                            }
                            // NO ESTA REGISTRADO
                            else {
                                let nickname_hash = data_user_file.nickname.clone();
                                if prefix.is_none() {
                                    data_connected
                                        .entry(data_user_file.nickname)
                                        .or_insert(stream);
                                }
                                data_registered
                                    .entry(nickname_hash.clone())
                                    .or_insert(data_user_file_new);
                                data_connected_all_servers.push(nickname_hash);
                            }
                            Ok(())
                        }
                        Err(_) => Err(String::from("fallo el read del data connected all servers")),
                    }
                }
                Err(_) => Err(String::from("Fallo read receptor")),
            }
        }
        Err(_) => Err(String::from("Fallo read receptor")),
    }
}

/// Una data_user_file se puede enviar cuando tenga todos sus atributos completos
fn ready_to_send(
    data_user: &DataUserFile,
    data_hash: &Arc<RwLock<HashMap<String, DataUserFile>>>,
    data_nickname: &Arc<RwLock<HashMap<String, TcpStream>>>,
) -> bool {
    match data_hash.read() {
        Ok(_hash_map) => match data_nickname.read() {
            Ok(_hash_nickname) => {
                if data_user.contains_all_values() {
                    return true;
                }
                false
            }
            Err(_) => false,
        },
        Err(_) => false,
    }
}
/// A a partir de un data_userfile y un stream, se envia al main thread una estructura con el data_user_file y el tcpstream
fn send_update_message(
    data_user: DataUserFile,
    data_hash: &Arc<RwLock<HashMap<String, DataUserFile>>>,
    tx1: &Sender<DataUserFileTcpStream>,
    stream: TcpStream,
    data_nickname: &Arc<RwLock<HashMap<String, TcpStream>>>,
    prefix: Option<String>,
) -> Result<(), String> {
    if ready_to_send(&data_user, data_hash, data_nickname) {
        let mut new_data_user: DataUserFile = DataUserFile::default_for_clients();
        new_data_user.clone_data_user(&data_user);
        let data_user_file_tcp_stream = DataUserFileTcpStream {
            data_file: Some(new_data_user),
            stream: Some(stream),
            prefix,
        };
        match tx1.send(data_user_file_tcp_stream) {
            Err(_) => return Err(String::from("Error con el send del thread")),
            Ok(_) => return Ok(()),
        }
    }
    Ok(())
}

/// Cuando no tenemos conocimiento de un usuario en un mismo server, esparcimos dicho mensaje a otros servidores para que actuen de manera acorde.
pub fn spread_command_neighbors(
    nickname: String,
    server_lock: Arc<RwLock<Server>>,
    server_sender: &TcpStream,
    string_command: &str,
) -> Result<(), String> {
    // println!("entra para spredear el mensaje con {}", string_command);
    if let Ok(server) = server_lock.write() {
        for mut server in &server.neighbours {
            if let Ok(socket_address_receiver) = server.1.peer_addr() {
                if let Ok(socket_address_sender) = server_sender.peer_addr() {
                    if socket_address_receiver == socket_address_sender {
                        continue;
                    }
                    let mut message = ":".to_string();
                    message.push_str(&nickname);
                    message.push_str(string_command);
                    message.push('\r');
                    message.push('\n');
                    if server.1.write(message.as_bytes()).is_err() {
                        return Err(String::from("Neighbour write failed"));
                    }
                    continue;
                }
                return Err(String::from("NO se pudo sacar el address"));
            }
            return Err(String::from("NO se pudo sacar el address"));
        }
        return Ok(());
    }
    Err(String::from("Err no server write"))
}

/// Funcion que dependiendo del Message command del parser llama a distintas funciones para poder hacer las cosas que se quieren,
///  ya sea loggear un usuario, unir a un canal, entre muchas otras cosas
fn match_message(
    message: MessageCommand,
    data_user: &mut DataUserFile,
    tx1: &Sender<DataUserFileTcpStream>,
    stream: TcpStream,
    channels: Arc<Channels>,
    data_base: Arc<DataBase>,
    server: Arc<RwLock<Server>>,
) -> Result<String, String> {
    match message.cmd {
        Message::Password(pass_info) => set_password(pass_info, data_user)?,
        Message::User(user_info) => set_user(user_info, data_user)?,
        Message::Nick(nick_info) => {
            let respuesta: String = set_nick_name(
                nick_info.clone(),
                data_user,
                &data_base.data_registered,
                &data_base.data_connected_all_servers,
            )?;
            if respuesta == format!("{}{}", "Welcome! ", nick_info.nick) {
                spread_register_neighbors(
                    data_user.clone(),
                    false,
                    server.clone(),
                    &stream,
                    &message.prefix,
                )?;
            }
            if respuesta == format!("{}{}", "Register! ", nick_info.nick) {
                spread_register_neighbors(
                    data_user.clone(),
                    true,
                    server,
                    &stream,
                    &message.prefix,
                )?;
            }

            send_update_message(
                data_user.clone(),
                &data_base.data_registered,
                tx1,
                stream,
                &data_base.data_connected,
                message.prefix.clone(),
            )?;
            return Ok(respuesta);
        }

        Message::Oper(oper_info) => {
            if data_user.contains_all_values() {
                let respuesta =
                    set_operator(data_user.clone(), oper_info.clone(), &data_base.operators)?;
                let mut string_command = String::from(" OPER ");
                string_command.push_str(&oper_info.nick);
                string_command.push(' ');
                string_command.push_str(&oper_info.pass);
                spread_command_neighbors(
                    data_user.nickname.clone(),
                    server,
                    &stream,
                    &string_command,
                )?;
                return Ok(respuesta);
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::Quit(quit_info) => {
            if data_user.contains_all_values() {
                let socket = match stream.peer_addr() {
                    Ok(socket) => socket,
                    Err(_) => return Err(String::from("no se pudo leer el socket address")),
                };
                let respuesta = quit_message(
                    &data_user.nickname,
                    quit_info.clone(),
                    stream,
                    data_base.clone(),
                    message.prefix,
                )?;
                write_connected_on_all_servers(&data_base, server.clone(), false)?;
                spread_quit_neighbors(&data_user.nickname, quit_info, server, socket, true)?;
                return Ok(respuesta);
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::PrivateMessage(private_info) => {
            if data_user.contains_all_values() {
                let data_user_priv = data_user.clone();
                send_private_message(
                    data_base,
                    private_info,
                    data_user_priv,
                    &channels,
                    server,
                    &stream,
                )?;
                return Ok(String::from(""));
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::Join(join_info) => {
            if data_user.contains_all_values() {
                let respuesta = channels.join_channel(
                    join_info.clone(),
                    data_user.nickname.clone(),
                    data_base.operators.clone(),
                )?;
                write_database_on_file(&data_base, &channels, server.clone())?;

                let mut string_command = String::from(" JOIN ");
                let string_channels_to_join: String =
                    change_vec_to_string(join_info.channel_list.clone());
                string_command.push_str(&string_channels_to_join);
                if let Some(channel_key) = join_info.channel_key {
                    let string_channels_key: String = change_vec_to_string(channel_key);
                    string_command.push(' ');
                    string_command.push_str(&string_channels_key);
                }
                spread_command_neighbors(
                    data_user.nickname.clone(),
                    server,
                    &stream,
                    &string_command,
                )?;
                return Ok(respuesta);
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::Part(part_info) => {
            if data_user.contains_all_values() {
                let respuesta =
                    channels.leave_channel(part_info.clone(), data_user.nickname.clone());
                write_database_on_file(&data_base, &channels, server.clone())?;
                let mut string_command = String::from(" PART ");
                let string_channels_to_part: String = change_vec_to_string(part_info.channel_list);
                string_command.push_str(&string_channels_to_part);
                spread_command_neighbors(
                    data_user.nickname.clone(),
                    server,
                    &stream,
                    &string_command,
                )?;
                return respuesta;
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::Names(names_info) => {
            if data_user.contains_all_values() {
                return channels.names(names_info, data_base);
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::Who(who_info) => {
            if data_user.contains_all_values() {
                return channels.who(
                    who_info,
                    &data_base.data_registered,
                    data_user.nickname.clone(),
                );
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::WhoIs(who_is_info) => {
            if data_user.contains_all_values() {
                return channels.who_info_of_user(who_is_info, data_base);
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::List(list_info) => {
            if data_user.contains_all_values() {
                return channels.list(list_info, &data_user.nickname);
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::Invite(invite_info) => {
            if data_user.contains_all_values() {
                let respuesta = channels.invite_to_channel(
                    invite_info.clone(),
                    &data_base.data_registered,
                    &data_user.nickname,
                )?;
                write_database_on_file(&data_base, &channels, server.clone())?;

                let mut string_command = String::from(" INVITE ");
                string_command.push_str(&invite_info.nick);
                string_command.push(' ');
                string_command.push_str(&invite_info.channel);
                spread_command_neighbors(
                    data_user.nickname.clone(),
                    server,
                    &stream,
                    &string_command,
                )?;
                return Ok(respuesta);
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::Kick(kick_info) => {
            if data_user.contains_all_values() {
                let respuesta = channels.kick_user_from_channel(
                    kick_info.clone(),
                    data_user.nickname.clone(),
                    &data_base.data_connected,
                );
                write_database_on_file(&data_base, &channels, server.clone())?;
                let mut string_command = String::from(" KICK ");
                string_command.push_str(&kick_info.channel);
                string_command.push(' ');
                string_command.push_str(&kick_info.nick);
                spread_command_neighbors(
                    data_user.nickname.clone(),
                    server,
                    &stream,
                    &string_command,
                )?;
                return respuesta;
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::Away(away_info) => {
            if data_user.contains_all_values() {
                let data_user = away(away_info.clone(), &mut data_user.clone());
                let mut string_command = String::from(" AWAY");
                if let Some(away_message) = away_info.away {
                    string_command.push_str(" :");
                    string_command.push_str(&away_message);
                }
                spread_command_neighbors(
                    data_user.nickname.clone(),
                    server,
                    &stream,
                    &string_command,
                )?;
                send_update_message(
                    data_user,
                    &data_base.data_registered,
                    tx1,
                    stream,
                    &data_base.data_connected,
                    message.prefix,
                )?;
                return Ok(String::from("RPL_AWAY"));
            } else {
                return Ok(String::from("NO ESTA REGISTRADO"));
            };
        }

        Message::Topic(topic_info) => {
            if data_user.contains_all_values() {
                let respuesta =
                    channels.give_or_receive_topic(topic_info.clone(), &data_user.nickname);
                write_database_on_file(&data_base, &channels, server.clone())?;
                if let Some(nueva_topic) = topic_info.topic {
                    let mut string_command = String::from(" TOPIC ");
                    string_command.push_str(&topic_info.channel);
                    string_command.push(' ');
                    string_command.push_str(&nueva_topic);
                    spread_command_neighbors(
                        data_user.nickname.clone(),
                        server,
                        &stream,
                        &string_command,
                    )?;
                }

                return respuesta;
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::Mode(mode_info) => {
            if data_user.contains_all_values() {
                match mode_info.channel.clone() {
                    Some(channel) => {
                        let respuesta = channels.mode(mode_info.clone(), &data_user.nickname);
                        write_database_on_file(&data_base, &channels, server.clone())?;
                        spread_mode_neighbors(
                            data_user.nickname.clone(),
                            mode_info.clone(),
                            server,
                            &stream,
                        )?;
                        match respuesta {
                            Ok(_) => match mode_info.set {
                                true => {
                                    return Ok::<String, String>(format!(
                                        "MODE +{} {} {}",
                                        mode_info.flag,
                                        channel,
                                        mode_info.nick.unwrap_or_else(|| "no_nick".to_owned())
                                    ))
                                }
                                false => {
                                    return Ok(format!(
                                        "MODE -{} {} {}",
                                        mode_info.flag,
                                        channel,
                                        mode_info.nick.unwrap_or_else(|| "no_nick".to_owned())
                                    ))
                                }
                            },
                            Err(error) => return Err(error),
                        }
                    }
                    None => return Err(String::from("todavia no implementado con nicks")),
                };
            }
            return Ok(String::from("NO ESTA REGISTRADO"));
        }
        Message::Connected => return channels.return_connected_channels(&data_user.nickname),
        Message::Server(server_info) => return new_server_request(server_info, stream, server), // enviar todas las bases de datos despues de esto!!!!
        Message::Send(send_info) => {
            return load_file(send_info, data_base.clone(), channels, server)
        }
        Message::ServerQuit(squit_info) => {
            return squit_request(
                squit_info,
                data_user.nickname.clone(),
                &data_base.operators,
                server,
                &stream,
            )
        }
        Message::ServerQuitRequest(server_quit_request_info) => {
            let answer = quit_server(
                server_quit_request_info,
                server.clone(),
                data_base.data_connected.clone(),
                &stream,
            );
            if let Ok(ok_answer) = answer.clone() {
                if ok_answer == *"OK" {
                    if let Ok(data_connected) = data_base.data_connected.write() {
                        for tcp_stream in data_connected.values() {
                            if let Err(e) = tcp_stream.shutdown(Shutdown::Both) {
                                return Err(e.to_string());
                            }
                        }
                    }

                    // println!("CONNECTION DROPPED");
                    let mut message_shut_server = String::from("SHUT ");
                    if let Ok(server) = server.read() {
                        message_shut_server.push_str(&server.name);
                    }
                    let nick_server = String::from("servidor");
                    spread_command_neighbors(nick_server, server, &stream, &message_shut_server)?;
                }
            }
            return answer;
        }
        Message::Shut(shut_info) => return shut_connection_from_server(shut_info, server),
    }

    let data_user_send = data_user.clone();
    send_update_message(
        data_user_send,
        &data_base.data_registered,
        tx1,
        stream,
        &data_base.data_connected,
        message.prefix.clone(),
    )?;
    Ok(String::from("recibido!"))
}

fn handle_client(
    cliente: TcpStream,
    tx1: Sender<DataUserFileTcpStream>,
    channels: Arc<Channels>,
    data_base: Arc<DataBase>,
    server: Arc<RwLock<Server>>,
) -> Result<(), String> {
    let reader = BufReader::new(&cliente);
    let mut writer: TcpStream = cliente
        .try_clone()
        .map_err(|_| "ERROR no se pudo clonar el stream".to_string())?;
    let iter = MyLines::new(reader);
    let mut data_user: DataUserFile = DataUserFile::default_for_clients();
    for line in iter {
        match line {
            Err(_) => return Err(String::from("Error con el line del cliente")),
            Ok(linea_no_parseada) => match parser(linea_no_parseada.to_string()) {
                Err(error) => {
                    if let Err(e) = writer.write(error.as_bytes()) {
                        return Err(e.to_string());
                    }
                }
                Ok(message) => {
                    // del if");
                    if let Some(nickname) = message.prefix.clone() {
                        match message.cmd.clone() {
                            Message::Nick(_nickinfo) => println!(),
                            Message::Password(_pass_info) => println!(),
                            Message::User(_user_info) => println!(),
                            _ => {
                                if let Ok(users_registered) = data_base.data_registered.read() {
                                    println!("ENTRA A DATA REGISTERED");
                                    if let Some(data_user_prefix) = users_registered.get(&nickname)
                                    {
                                        data_user = data_user_prefix.clone();
                                    }
                                }
                            }
                        }
                    }
                    if let Ok(cliente) = cliente.try_clone() {
                        match match_message(
                            message,
                            &mut data_user,
                            &tx1,
                            cliente,
                            channels.clone(),
                            data_base.clone(),
                            server.clone(),
                        ) {
                            Err(error) => {
                                if let Err(e) = writer.write(error.as_bytes()) {
                                    return Err(e.to_string());
                                }
                            }
                            Ok(string_a_mandar) => {
                                if string_a_mandar.as_str() == "Connection Dropped" {
                                    return Ok(());
                                }
                                if let Err(e) = writer.write(string_a_mandar.as_bytes()) {
                                    return Err(e.to_string());
                                }
                            }
                        }
                    }
                }
            },
        };
    }
    Ok(())
}

// #[cfg(test)]
// mod tests {

//     use crate::config_file::ConfigFile;

//     use super::*;
//     use std::io::Read;
//     use std::str;
//     use std::time::Duration;

//     struct SocketHandler {
//         reader: Option<TcpStream>,
//         writer: Option<TcpStream>,
//     }
//     impl SocketHandler {
//         pub fn new(port: &str) -> SocketHandler {
//             if let Ok(writer) = TcpStream::connect(port) {
//                 if let Ok(reader) = writer.try_clone() {
//                     return SocketHandler {
//                         reader: Some(reader),
//                         writer: Some(writer),
//                     };
//                 }
//             }
//             println!("se inicializo con none");

//             SocketHandler {
//                 reader: None,
//                 writer: None,
//             }
//         }

//         pub fn write(&mut self, message: &str) {
//             if let Some(writer) = &mut self.writer {
//                 if let Err(_) = writer.write(message.as_bytes()) {
//                     println!("Fallo write del socket")
//                 }
//             }
//         }

//         pub fn read(&mut self) -> String {
//             if let Some(reader) = &mut self.reader {
//                 let mut buffer = [0; 500];
//                 if let Ok(consumed) = reader.read(&mut buffer) {
//                     if let Ok(response) = str::from_utf8(&buffer[..consumed]) {
//                         return response.to_string();
//                     }
//                 }
//             }
//             String::from("Fallo el read del socket")
//         }
//     }

//     fn initialize_data_base_and_spawn_client_handlers() {
//         let _join_handler: thread::JoinHandle<_> = thread::spawn(move || {
//             //Inicializo estructuras de datos
//             if let Ok(configf) = ConfigFile::new("./src/config_file".to_string()) {
//                 let config_clone = configf.clone();
//                 let server = Arc::new(RwLock::new(Server {
//                     name: config_clone.server_name,
//                     neighbours: HashMap::new(),
//                     password: config_clone.server_password,
//                     data_base: None,
//                     data_file_path: config_clone.data_file_path,
//                     joined_channels_path: config_clone.joined_channels_path,
//                     data_channels_path: config_clone.data_channels_path,
//                     users_coneccted_path: config_clone.users_connected_path,
//                 }));

//                 if manage_server(server, "127.0.0.1:8096".to_string()).is_err() {
//                     println!("Test finished");
//                 }
//             }
//         });
//     }

//     #[test]
//     /// Mandar un mensaje con formato invalido devuelve error de formato
//     fn incorrect_format_test() -> Result<(), String> {
//         //let server = MockUpServer::new("127.0.0.1:8095".to_string());

//         //Inicializo el handler del cliente
//         initialize_data_base_and_spawn_client_handlers();

//         thread::sleep(Duration::from_millis(2)); // Si no hago este sleep el Socket Handler no llega a hacer el bind

//         let mut socket_handler = SocketHandler::new("127.0.0.1:8096");

//         socket_handler.write("Aqui va mi mensaje sin carriage return ni new line");
//         assert_eq!(
//             socket_handler.read(),
//             "error de formato: no hay cr-nl".to_string()
//         );

//         socket_handler.write("Aqui va mi mensaje sin carriage return \n");
//         assert_eq!(
//             socket_handler.read(),
//             "error de formato: no hay cr-nl".to_string()
//         );

//         socket_handler.write("Aqui va mi mensaje sin new line \r");
//         assert_eq!(
//             socket_handler.read(),
//             "error de formato: no hay cr-nl".to_string()
//         );

//         Ok(())
//     }

//     #[test]
//     /// Ingresar un comando incorrecto devuelve mensaje indicando lo sucedido
//     fn unknown_command_test() -> Result<(), String> {
//         //Inicializo datos y estructuras y client handler
//         initialize_data_base_and_spawn_client_handlers();

//         thread::sleep(Duration::from_millis(2)); // Si no hago este sleep el Socket Handler no llega a hacer el bind
//         let mut socket_handler = SocketHandler::new("127.0.0.1:8096");
//         //Testeo el caso
//         socket_handler.write("COMANDO no valido \r\n");
//         assert_eq!(
//             socket_handler.read(),
//             "No se identifico el Comando".to_string()
//         );

//         Ok(())
//     }

//     #[test]
//     fn send_message_without_register_first() -> Result<(), String> {
//         initialize_data_base_and_spawn_client_handlers();
//         thread::sleep(Duration::from_millis(2)); // Si no hago este sleep el Socket Handler no llega a hacer el bind
//         let mut juan = SocketHandler::new("127.0.0.1:8096");
//         juan.write("PRIVMSG franco :Hola franco \r\n");
//         assert_eq!(juan.read(), "NO ESTA REGISTRADO".to_string());

//         Ok(())
//     }
// }

// #[test]
// fn multiple_login_test() -> Result<(), String> {

//     //Inicializo el thread qe va a manejar el cliente

//     initialize_data_base_and_spawn_client_handlers();

//     thread::sleep(Duration::from_millis(2)); // Si no hago este sleep el Socket Handler no llega a hacer el bind

//     let mut juan = SocketHandler::new("127.0.0.1:8096");
//     let mut pedro = SocketHandler::new("127.0.0.1:8096");

//     // Inicio sesion Juan
//     juan.write("PASS contra \r\n");
//     juan.write("NICK franco \r\n");
//     assert_eq!(juan.read(), "recibido!".to_string());
//     assert_eq!(juan.read(), "Welcome! franco".to_string());

//     //Inicio sesion Pedro
//     pedro.write("PASS contra \r\n");
//     pedro.write("NICK tomasito \r\n");
//     assert_eq!(pedro.read(), "recibido!".to_string());
//     assert_eq!(pedro.read(), "Welcome! tomasito".to_string());

//     Ok(())
//         }

//     #[test]
//     fn private_message_test() -> Result<(), String> {
//         let  server = MockUpServer::new("127.0.0.1:8095".to_string());
//         let mut juan = SocketHandler::new("127.0.0.1:8095");
//         let mut pedro = SocketHandler::new("127.0.0.1:8095");

//         //Inicializo el thread qe va a manejar el cliente
//         initialize_data_base_and_spawn_client_handlers(server);

//         //Inicio sesion Pedro
//         pedro.write("PASS contra \r\n");
//         pedro.write("NICK pedro \r\n");
//         assert_eq!(pedro.read(), "recibido!".to_string());
//         assert_eq!(pedro.read(), "Welcome!".to_string());

//         // Inicio sesion Juan
//         juan.write("PASS contra \r\n");
//         juan.write("NICK juan \r\n");
//         assert_eq!(juan.read(), "recibido!".to_string());
//         assert_eq!(juan.read(), "Welcome!".to_string());

//         juan.write("PRIVMSG pedro :Como estas pedrin? \r\n");
//         assert_eq!(pedro.read(),"PRIVMSG juan :Como estas pedrin?".to_string());

//         Ok(())
//     }

// }
