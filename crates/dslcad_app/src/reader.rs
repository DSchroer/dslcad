use dslcad_parser::Reader;
use std::io::{Error, ErrorKind, Read};
use std::path::{Path, PathBuf};
use vfs::FileSystem;

pub struct ProjectReader<'a> {
    fs: &'a dyn FileSystem,
}

impl<'a> ProjectReader<'a> {
    pub fn new(fs: &'a dyn FileSystem) -> Self {
        Self { fs }
    }
}

impl<'a> Reader for ProjectReader<'a> {
    fn read(&self, path: &Path) -> Result<String, Error> {
        let mut buffer = String::new();
        self.fs
            .open_file(path.to_str().unwrap())
            .map_err(|e| Error::new(ErrorKind::Other, e))?
            .read_to_string(&mut buffer)
            .map_err(|e| Error::new(ErrorKind::Other, e))?;
        Ok(buffer)
    }

    fn normalize(&self, _path: &Path) -> PathBuf {
        todo!()
    }
}
