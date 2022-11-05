pub trait Reader {
    fn read(&self, name: &str) -> String;
}