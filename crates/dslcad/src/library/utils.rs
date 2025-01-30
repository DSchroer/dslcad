use crate::runtime::RuntimeError;

pub fn error(text: String) -> Result<f64, RuntimeError> {
    Err(RuntimeError::UserDefined(text))
}
