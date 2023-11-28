use opencascade::Error;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RuntimeError {
    #[error("Unknown identifier {0}")]
    UnknownIdentifier(String),
    #[error("Unset parameter {0}")]
    UnsetParameter(String),
    #[error("Could not find property {0}")]
    MissingProperty(String),
    #[error("Mismatched types")]
    UnexpectedType(),
    #[error("Script did not return a value")]
    NoReturnValue(),
    #[error("Reduce must have at least one value")]
    EmptyReduce(),
    #[error("Stack overflow")]
    StackOverflow(),
    #[error("{0}")]
    Opencascade(opencascade::Error),
    #[error("Could not find function with name '{name}'")]
    CouldNotFindFunction { name: String },
    #[error("Could not find default argument for function '{name}'")]
    UnknownDefaultArgument { name: String },
    #[error("Multi part scripts must all use the same type")]
    MismatchedPartTypes,
    #[error("Multi part scripts must only return shapes or edges")]
    InvalidMultiPartType,
    #[error("Could not find function {target} did you mean one of {options:?}?")]
    CouldNotFindFunctionSignature {
        target: String,
        options: Vec<String>,
    },
    #[error("Can not build arc with two identical points")]
    ArcWithIdenticalPoints(),
}

impl From<opencascade::Error> for RuntimeError {
    fn from(value: Error) -> Self {
        RuntimeError::Opencascade(value)
    }
}
