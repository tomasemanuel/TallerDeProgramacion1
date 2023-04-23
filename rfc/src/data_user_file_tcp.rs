use std::net::TcpStream;

use crate::datauser::DataUserFile;

#[derive(Debug)]
pub struct DataUserFileTcpStream {
    pub data_file: Option<DataUserFile>,
    pub stream: Option<TcpStream>,
    pub prefix: Option<String>,
}
