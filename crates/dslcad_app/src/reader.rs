use dslcad_parser::Reader;
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};

pub struct FileReader;
impl Reader for FileReader {
    fn read(&self, path: &Path) -> Result<String, Error> {
        fs::read_to_string(path)
    }

    fn normalize(&self, _path: &Path) -> PathBuf {
        todo!()
    }
}
