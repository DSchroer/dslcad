use dslcad_occt::Error;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RuntimeError {
    #[error("unknown identifier {0}")]
    UnknownIdentifier(String),
    #[error("unset parameter {0}")]
    UnsetParameter(String),
    #[error("could not find property {0}")]
    MissingProperty(String),
    #[error("mismatched types")]
    UnexpectedType(),
    #[error("script did not return a value")]
    NoReturnValue(),
    #[error("reduce must have at least one value")]
    EmptyReduce(),
    #[error("stack overflow")]
    StackOverflow(),
    #[error(transparent)]
    Opencascade(dslcad_occt::Error),
    #[error("could not find function with name '{name}'")]
    CouldNotFindFunction { name: String },
    #[error("could not find default argument for function '{name}'")]
    UnknownDefaultArgument { name: String },
    #[error("multi part scripts must all use the same type")]
    MismatchedPartTypes,
    #[error("multi part scripts must only return shapes or edges")]
    InvalidMultiPartType,
    #[error("could not find function {target} did you mean one of {options:?}?")]
    CouldNotFindFunctionSignature {
        target: String,
        options: Vec<String>,
    },
    #[error("can not build arc with two identical points")]
    ArcWithIdenticalPoints(),
    #[error("{0}")]
    UserDefined(String),
}

impl From<dslcad_occt::Error> for RuntimeError {
    fn from(value: Error) -> Self {
        RuntimeError::Opencascade(value)
    }
}
