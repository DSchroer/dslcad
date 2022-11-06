use std::path::{PathBuf};

pub trait Reader {
    fn read(&self, name: &str) -> String;
    fn normalize(&self, path: &str) -> PathBuf;
}