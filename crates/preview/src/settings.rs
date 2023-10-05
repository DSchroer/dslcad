use bevy::prelude::Resource;
use bevy::utils::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;

pub trait Store: Resource + Default {
    fn load<'a>(&'a self, key: &'static str) -> Option<&'a str>;
    fn store(&mut self, key: &'static str, value: &str);
}

pub type Settings = FileStore;

#[derive(Default, Resource)]
pub struct MemStore {
    values: HashMap<&'static str, String>,
}

impl Store for MemStore {
    fn load<'a>(&'a self, key: &'static str) -> Option<&'a str> {
        self.values.get(key).map(|f| f.as_str())
    }

    fn store(&mut self, key: &'static str, value: &str) {
        self.values.insert(key, value.to_string());
    }
}

#[derive(Resource)]
pub struct FileStore {
    values: HashMap<String, String>,
}

impl Default for FileStore {
    fn default() -> Self {
        let mut store = Self {
            values: HashMap::new(),
        };
        store.load_values();
        store
    }
}

impl FileStore {
    fn load_values(&mut self) {
        let config_file = Path::new(env!("HOME"))
            .join(".config")
            .join("dslcad")
            .join("config.ini");
        if let Ok(config) = fs::read_to_string(config_file) {
            for mut line in config.lines().map(|l| l.split('=')) {
                if let (Some(key), Some(value)) = (line.next(), line.next()) {
                    self.values.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    fn save_values(&self) {
        let mut output = String::new();
        for (key, value) in &self.values {
            writeln!(output, "{key}={value}").expect("failed to write value");
        }

        let config_dir = Path::new(env!("HOME")).join(".config").join("dslcad");

        fs::create_dir_all(&config_dir).expect("failed to create config dir");
        fs::write(config_dir.join("config.ini"), output).expect("failed to save config");
    }
}

impl Store for FileStore {
    fn load<'a>(&'a self, key: &'static str) -> Option<&'a str> {
        self.values.get(key).map(|f| f.as_str())
    }

    fn store(&mut self, key: &'static str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
        self.save_values();
    }
}
