// use std::net::TcpStream;

static POS_NICKNAME_HASH: usize = 0;
static POS_NICKNAME_ACTUALIZADO_HASH: usize = 1;
static POS_PASSWORD_HASH: usize = 2;
static POS_USERNAME_HASH: usize = 3;
static POS_HOSTNAME_HASH: usize = 4;
static POS_SERVERNAME_HASH: usize = 5;
static POS_REALNAME_HASH: usize = 6;

#[derive(Debug, Clone)]
pub struct DataUserFile {
    pub password: String,
    pub nickname: String,
    pub nickname_actualizado: String,
    pub username: String,
    pub hostname: String,
    pub servername: String,
    pub realname: String,
    pub away: Option<String>,
    // pub stream: Option<TcpStream>,
}

impl DataUserFile {
    pub fn new(vector: Vec<&str>) -> DataUserFile {
        DataUserFile {
            password: vector[POS_PASSWORD_HASH].to_string(),
            nickname: vector[POS_NICKNAME_HASH].to_string(),
            nickname_actualizado: vector[POS_NICKNAME_ACTUALIZADO_HASH].to_string(),
            username: vector[POS_USERNAME_HASH].to_string(),
            hostname: vector[POS_HOSTNAME_HASH].to_string(),
            servername: vector[POS_SERVERNAME_HASH].to_string(),
            realname: vector[POS_REALNAME_HASH].to_string(),
            away: None,
            // stream: None,
        }
    }
    pub fn default_for_clients() -> DataUserFile {
        DataUserFile {
            password: String::from("-1"),
            nickname: String::from("-1"),
            nickname_actualizado: String::from("-1"),
            username: String::from("-1"),
            hostname: String::from("-1"),
            servername: String::from("-1"),
            realname: String::from("-1"),
            away: None,
            // stream: stream,
        }
    }

    pub fn default_for_servers() -> DataUserFile {
        DataUserFile {
            password: String::from("1"),
            nickname: String::from("1"),
            nickname_actualizado: String::from("1"),
            username: String::from("1"),
            hostname: String::from("1"),
            servername: String::from("1"),
            realname: String::from("1"),
            away: None,
            // stream: stream,
        }
    }

    pub fn clone_data_user(&mut self, data_user: &DataUserFile) {
        self.password = data_user.password.clone();
        self.nickname = data_user.nickname.clone();
        self.nickname_actualizado = data_user.nickname_actualizado.clone();
        self.username = data_user.username.clone();
        self.hostname = data_user.hostname.clone();
        self.servername = data_user.servername.clone();
        self.realname = data_user.realname.clone();
        self.away = data_user.away.clone();
    }

    pub fn contains_all_values(&self) -> bool {
        let menos_uno = String::from("-1");
        self.password != menos_uno
            && self.nickname != menos_uno
            && self.nickname_actualizado != menos_uno
            && self.username != menos_uno
            && self.hostname != menos_uno
            && self.servername != menos_uno
            && self.realname != menos_uno
    }
}
