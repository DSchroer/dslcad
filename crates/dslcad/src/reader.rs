use crate::parser::Reader;
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};

pub struct FsReader;

impl Reader for FsReader {
    fn read(&self, path: &Path) -> Result<String, Error> {
        fs::read_to_string(path)
    }

    fn normalize(&self, path: &Path) -> PathBuf {
        path.to_path_buf().canonicalize().unwrap()
    }
}
