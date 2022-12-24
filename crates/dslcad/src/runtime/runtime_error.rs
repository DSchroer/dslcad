use super::Type;
use opencascade::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Argument does not exist {1} in {0}")]
    ArgumentDoesNotExist(String, String),
    #[error("Unknown identifier {0}")]
    UnknownIdentifier(String),
    #[error("Unset parameter {0}")]
    UnsetParameter(String),
    #[error("Could not find property {0}")]
    MissingProperty(String),
    #[error("Mismatched types between {0}")]
    UnexpectedType(Type),
    #[error("Script did not return a value")]
    NoReturnValue(),
    #[error("Cant Write")]
    CantWrite(),
    #[error("{0}")]
    Opencascade(opencascade::Error),
}

impl From<opencascade::Error> for RuntimeError {
    fn from(value: Error) -> Self {
        RuntimeError::Opencascade(value)
    }
}
