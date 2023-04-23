use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Clone)]
pub struct ConfigFile {
    pub server_type: String,
    pub main_port: String,
    pub data_file_path: String,
    pub joined_channels_path: String,
    pub secondary_port: String,
    pub server_name: String,
    pub data_channels_path: String,
    pub users_connected_path: String,
    pub server_password: String,
}

impl ConfigFile {
    /// Se crea una nueva config file usando el path que se le provee en el config path, a partir de esta estructura se
    /// sabe el server_type, main_port,data_file_path,joined_channels_path,
    ///,secondary_port,server_name,data_channels_path y users_connected_path
    pub fn new(config_path: String) -> Result<ConfigFile, String> {
        let file: File = match File::open(config_path) {
            Ok(data_file) => data_file,
            Err(_) => return Err(String::from("Config file not found")),
        };

        let mut main_port = "".to_string();
        let mut server_type = "".to_string();
        let mut data_file_path = "".to_string();
        let mut joined_channels_path = "".to_string();
        let mut secondary_port = "".to_string();
        let mut server_name = "".to_string();
        let mut data_channels_path = "".to_string();
        let mut users_connected_path = "".to_string();
        let mut server_password = "".to_string();

        let reader = BufReader::new(file);

        for line in reader.lines().flatten() {
            let split_line = line.split(','); // en nuestro database lo separamos como nick,contraseña,user,etc    el nick va a ser la clave del usuario
            let vector_split = split_line.collect::<Vec<&str>>();
            match vector_split[0] {
                "MAIN_PORT" => main_port = vector_split[1].to_string(),
                "DATA_FILE_PATH" => data_file_path = vector_split[1].to_string(),
                "JOINED_CHANNELS_PATH" => joined_channels_path = vector_split[1].to_string(),
                "SERVER" => server_type = vector_split[1].to_string(),
                "SECONDARY_PORT" => secondary_port = vector_split[1].to_string(),
                "SERVER_NAME" => server_name = vector_split[1].to_string(),
                "DATA_CHANNELS_PATH" => data_channels_path = vector_split[1].to_string(),
                "USERS_CONNECTED" => users_connected_path = vector_split[1].to_string(),
                "SERVER_PASSWORD" => server_password = vector_split[1].to_string(),
                _ => continue,
            }
        }
        Ok(ConfigFile {
            server_type,
            main_port,
            data_file_path,
            joined_channels_path,
            secondary_port,
            server_name,
            data_channels_path,
            users_connected_path,
            server_password,
        })
    }
}
