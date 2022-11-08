use crate::syntax::Value;

pub trait Instance {
    fn get(&self, identifier: &str) -> Option<&Value>;
}
