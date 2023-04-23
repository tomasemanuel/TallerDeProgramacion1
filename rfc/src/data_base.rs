use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{Arc, RwLock},
};

use crate::datauser::DataUserFile;

#[derive(Debug, Clone)]
pub struct DataBase {
    pub data_registered: Arc<RwLock<HashMap<String, DataUserFile>>>,
    pub data_connected: Arc<RwLock<HashMap<String, TcpStream>>>,
    pub data_connected_all_servers: Arc<RwLock<Vec<String>>>,
    pub operators: Arc<RwLock<Vec<String>>>,
}
