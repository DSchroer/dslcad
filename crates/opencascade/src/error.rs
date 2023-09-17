use std::fmt::{Debug, Display, Formatter};

#[derive(PartialEq)]
pub struct Error(String);

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Error(value.to_owned())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error(value)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error").field("msg", &self.0).finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}
