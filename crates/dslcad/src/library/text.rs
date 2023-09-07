use crate::runtime::{RuntimeError, Value};
use std::collections::HashMap;

pub fn add(left: String, right: String) -> Result<String, RuntimeError> {
    Ok(left + &right)
}

pub fn string(item: Value) -> Result<String, RuntimeError> {
    match item {
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(n) => Ok(n.to_string()),
        Value::Text(n) => Ok(n),
        _ => Err(RuntimeError::UnexpectedType(item.get_type())),
    }
}

pub fn format(arguments: &HashMap<&str, Value>) -> Result<String, RuntimeError> {
    let message = arguments
        .get("message")
        .ok_or_else(|| RuntimeError::UnsetParameter("message".into()))?;
    let mut message = message.to_text()?;

    for (key, value) in arguments {
        message = message.replace(&format!("{{{}}}", key), &string(value.clone())?);
    }

    Ok(message)
}

pub fn formatln(arguments: &HashMap<&str, Value>) -> Result<String, RuntimeError> {
    Ok(format(arguments)? + "\n")
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn it_formats_strings() {
        assert_eq!(
            "hi".to_owned(),
            format(&HashMap::from([("message", Value::Text("hi".into()))])).unwrap()
        );

        assert_eq!(
            "hi dom".to_owned(),
            format(&HashMap::from([
                ("message", Value::Text("hi {name}".into())),
                ("name", Value::Text("dom".into()))
            ]))
            .unwrap()
        );
    }
}
