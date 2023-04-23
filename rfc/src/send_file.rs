use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::messages::send::*;

#[derive(Debug, Clone)]
pub struct FilePacket {
    pub name_size: u8,
    pub name: String,
    pub content_size: u8,
    pub content: String,
}

impl FilePacket {
    pub fn new(path: String, name: String) -> Result<FilePacket, String> {
        let name_size = name.len() as u8;

        let mut content = String::from("");
        if let Ok(file) = File::open(path) {
            if let Ok(_metadata) = file.metadata() {
                let reader = BufReader::new(file);
                for line in reader.lines().flatten() {
                    content.push_str(line.as_str());
                    content.push('\n');
                }
                content.pop();
            }
        }
        let content_size = content.len() as u8;

        Ok(FilePacket {
            name_size,
            name,
            content_size,
            content,
        })
    }

    /// SEND  3                                4 CONTENT
    pub fn to_bytes(&self) -> Vec<u8> {
        let command_bytes = "SEND ".as_bytes().to_vec();
        let name_size_bytes = self.name_size.to_be_bytes().to_vec();
        let name_bytes = self.name.as_bytes().to_vec();
        let content_size_bytes = self.content_size.to_be_bytes().to_vec();
        let content_bytes = self.content.as_bytes().to_vec();
        let crln = "\r\n".as_bytes().to_vec();

        [
            command_bytes,
            name_size_bytes,
            name_bytes,
            content_size_bytes,
            content_bytes,
            crln,
        ]
        .concat()
    }
} // SEND
pub fn from_bytes(bytes: Vec<u8>) -> SendInfo {
    let mut i = 0; // Contador
    let name_size = bytes[5] as usize;
    i += 5 + name_size + 1;

    if let Ok(name) = String::from_utf8(bytes[6..i].to_vec()) {
        let start_content = 5 + name_size + 2;
        let size = bytes.len();
        if let Ok(content) = String::from_utf8(bytes[start_content..(size - 2)].to_vec()) {
            return SendInfo::new(name, content);
        } else {
            SendInfo::new("".to_string(), "".to_string())
        };
    } else {
        return SendInfo::new("".to_string(), "".to_string());
    }
    SendInfo::new("".to_string(), "".to_string())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create_file_packet() {
        if let Ok(file_pak) =
            FilePacket::new("./src/data_file".to_string(), "data_file".to_string())
        {
            println!("{:?}", file_pak);
        }
    }

    #[test]
    fn file_packet_to_bytes() {
        if let Ok(file_pak) =
            FilePacket::new("./src/data_file".to_string(), "data_file".to_string())
        {
            println!("{:?}", file_pak.to_bytes());
        }
    }
}
