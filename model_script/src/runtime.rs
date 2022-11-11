use crate::library::Library;
use crate::parser::Document;
use crate::syntax::Accessible;
use crate::syntax::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::ops::Deref;
use std::rc::Rc;
use thiserror::Error;

pub struct EvalContext<'a> {
    pub library: Library,
    pub documents: &'a HashMap<String, Document>,
}

#[derive(Debug, Clone)]
pub struct ScriptInstance {
    arguments: HashMap<String, Box<Value>>,
    variables: HashMap<String, Box<Value>>,
    value: Box<Value>,
}

impl ScriptInstance {
    pub fn new(arguments: HashMap<String, Value>) -> Self {
        ScriptInstance {
            arguments: arguments
                .into_iter()
                .map(|(k, v)| (k, Box::new(v)))
                .collect(),
            variables: HashMap::new(),
            value: Box::new(Value::Empty),
        }
    }

    pub fn set(&mut self, identifier: String, value: Value) {
        self.variables.insert(identifier, Box::new(value));
    }

    pub fn set_value(&mut self, value: Value) {
        self.value = Box::new(value);
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn write_to_file(&mut self, path: &str) -> Result<(), Error> {
        let instance = self
            .value
            .to_shape()
            .ok_or_else(|| std::io::Error::from(ErrorKind::Other))?;
        let mut instance = instance.borrow_mut();
        instance.write_stl(path)
    }
}

impl Accessible for ScriptInstance {
    fn get(&self, identifier: &str) -> Option<&Value> {
        let val = self
            .arguments
            .get(identifier)
            .or_else(|| self.variables.get(identifier))?;
        Some(val)
    }
}

pub fn eval(
    doc: &Document,
    arguments: HashMap<String, Value>,
    ctx: &EvalContext,
) -> Result<ScriptInstance, RuntimeError> {
    let mut instance = ScriptInstance::new(arguments);

    for statement in doc.statements() {
        match statement {
            Statement::Variable { name, value } => match value {
                Some(value) => {
                    let value = eval_expression(&instance, value, ctx)?;
                    instance.set(name.clone(), value);
                }
                None => {
                    if instance.get(name).is_none() {
                        return Err(RuntimeError::UnsetParameter(name.to_string()));
                    }
                }
            },
            Statement::Return(e) => {
                instance.set_value(eval_expression(&instance, e, ctx)?);
            }
        }
    }

    Ok(instance)
}

fn eval_expression(
    instance: &dyn Accessible,
    expression: &Expression,
    ctx: &EvalContext,
) -> Result<Value, RuntimeError> {
    match expression {
        Expression::Literal(v) => Ok(v.clone()),
        Expression::Invocation { path, arguments } => {
            let mut argument_values = HashMap::new();
            for (name, argument) in arguments.clone().into_iter() {
                let value = eval_expression(instance, argument.deref(), ctx)?;
                argument_values.insert(name, value);
            }

            let doc = ctx.documents.get(path);
            match doc {
                None => match ctx.library.find(path) {
                    None => Err(RuntimeError::UnknownIdentifier(path.to_string())),
                    Some(f) => Ok(f(&argument_values)?),
                },
                Some(doc) => {
                    let v = eval(doc, argument_values, ctx)?;
                    Ok(Value::Script(Rc::new(RefCell::new(v))))
                }
            }
        }
        Expression::Reference(n) => {
            if let Some(value) = instance.get(n) {
                Ok(value.clone())
            } else {
                Err(RuntimeError::UnknownIdentifier(n.to_string()))
            }
        }
        Expression::Access(l, name) => access(instance, ctx, l, name),
    }
}

fn access(
    instance: &dyn Accessible,
    ctx: &EvalContext,
    l: &Expression,
    name: &str,
) -> Result<Value, RuntimeError> {
    let l = eval_expression(instance, l.deref(), ctx)?;

    let lv = l.to_script();

    if let Some(instance) = lv {
        match instance.borrow().get(name) {
            None => Err(RuntimeError::MissingProperty(name.to_owned())),
            Some(v) => Ok(v.clone()),
        }
    } else {
        Err(RuntimeError::UnexpectedType(l))
    }
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Unknown identifier {0}")]
    UnknownIdentifier(String),
    #[error("Unset parameter {0}")]
    UnsetParameter(String),
    #[error("Could not find property {0}")]
    MissingProperty(String),
    #[error("Mismatched types between {0}")]
    UnexpectedType(Value),
    #[error("Cant Write")]
    CantWrite(),
    #[error("Mismatched types between {0} and {1}")]
    MismatchedTypes(Value, Value),
}
