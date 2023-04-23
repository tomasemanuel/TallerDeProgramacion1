#[derive(Debug, Clone)]
pub struct FileInfoAnswer {
    pub file_name: String,
    pub bytes_transfered: u64,
    pub bytes_total: u64,
}
