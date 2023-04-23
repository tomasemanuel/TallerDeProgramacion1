use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug)]
pub struct NameIpMap {
    map: HashMap<String, IpAddr>,
    file_ip: HashMap<String, String>,
}

impl Default for NameIpMap {
    fn default() -> Self {
        Self::new()
    }
}

impl NameIpMap {
    pub fn new() -> NameIpMap {
        NameIpMap {
            map: HashMap::new(),
            file_ip: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, name: String, ip: IpAddr) {
        self.map.insert(name, ip);
    }
    pub fn add_entry_file(&mut self, name: String, ip: String) {
        self.file_ip.insert(name, ip);
    }

    pub fn get_ip(&self, name: &str) -> Option<&IpAddr> {
        self.map.get(name)
    }
    pub fn get_file_ip(&self, name: &str) -> Option<&String> {
        self.file_ip.get(name)
    }
}
