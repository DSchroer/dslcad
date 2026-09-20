use crate::parser::Reader;
use std::fs;
use std::io::{Error, Read};
use std::path::{Path, PathBuf};

pub struct FsReader;

impl Reader for FsReader {
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, Error> {
        fs::read(path)
    }

    fn read(&self, path: &Path) -> Result<String, Error> {
        fs::read_to_string(path)
    }

    fn normalize(&self, path: &Path) -> PathBuf {
        path.to_path_buf().canonicalize().unwrap()
    }
}

pub struct StdinReader {
    source: String,
}

impl StdinReader {
    pub fn new() -> Result<Self, Error> {
        let mut source = String::new();
        std::io::stdin().read_to_string(&mut source)?;
        Ok(Self { source })
    }
}

impl Reader for StdinReader {
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, Error> {
        if path == Path::new("-") {
            Ok(self.source.as_bytes().to_vec())
        } else {
            fs::read(path)
        }
    }

    fn read(&self, path: &Path) -> Result<String, Error> {
        if path == Path::new("-") {
            Ok(self.source.clone())
        } else {
            fs::read_to_string(path)
        }
    }

    fn normalize(&self, path: &Path) -> PathBuf {
        if path == Path::new("-") {
            PathBuf::from("-")
        } else {
            path.to_path_buf().canonicalize().unwrap()
        }
    }
}
