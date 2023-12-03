use std::path::{Path, PathBuf};

pub trait Reader {
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, std::io::Error>;
    fn read(&self, path: &Path) -> Result<String, std::io::Error>;
    fn normalize(&self, path: &Path) -> PathBuf;
}
