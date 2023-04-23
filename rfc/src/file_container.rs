use std::{fs::File, io::Write};

#[derive(Debug, Clone)]
pub struct FileContainer {
    pub file_name: String,
    pub content: Vec<u8>,
    pub bytes_transfered: u64,
    pub size: u64,
}

impl FileContainer {
    pub fn write_file(&self) -> std::io::Result<()> {
        let mut file = File::create(self.file_name.clone())?;
        file.write_all(&self.content)?;
        Ok(())
    }
}
