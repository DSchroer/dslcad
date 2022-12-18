use std::path::{Path, PathBuf};

pub trait Reader {
    fn read(&self, path: &Path) -> Result<String, std::io::Error>;
    fn normalize(&self, path: &Path) -> PathBuf;
}
