use crate::syntax::Value;

pub trait Accessible {
    fn get(&self, identifier: &str) -> Option<&Value>;
}
