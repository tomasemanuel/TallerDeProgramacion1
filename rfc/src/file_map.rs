use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FileMap {
    map: HashMap<String, String>,
    total_bytes: HashMap<String, String>,
    bytes_written: HashMap<String, String>,
}

impl Default for FileMap {
    fn default() -> Self {
        Self::new()
    }
}

impl FileMap {
    pub fn new() -> Self {
        FileMap {
            map: HashMap::new(),
            total_bytes: HashMap::new(),
            bytes_written: HashMap::new(),
        }
    }

    pub fn add_path(&mut self, file_name: &str, absolute_path: &str) {
        self.map
            .insert(file_name.to_owned(), absolute_path.to_owned());
    }
    pub fn add_file(&mut self, file_name: &str, total_bytes: &str) {
        self.total_bytes
            .insert(file_name.to_owned(), total_bytes.to_owned());
    }
    pub fn add_file_bytes_written(&mut self, file_name: &str, total_bytes: &str) {
        self.bytes_written
            .insert(file_name.to_owned(), total_bytes.to_owned());
    }

    pub fn get_path(&self, file_name: &str) -> Option<&str> {
        self.map.get(file_name).map(|s| s.as_str())
    }
    pub fn get_total_bytes(&self, file_name: &str) -> Option<&str> {
        self.total_bytes.get(file_name).map(|s| s.as_str())
    }
    pub fn get_written_bytes(&self, file_name: &str) -> Option<&str> {
        self.bytes_written.get(file_name).map(|s| s.as_str())
    }

    pub fn print_map(&self) -> HashMap<String, String> {
        self.map.clone()
    }
}
