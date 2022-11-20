mod scope;

use crate::library::Library;
use crate::parser::Document;
use crate::runtime::scope::Scope;
use crate::syntax::Accessible;
use crate::syntax::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
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
    pub fn from_scope(value: Value, scope: Scope) -> Self {
        ScriptInstance {
            arguments: scope.arguments,
            variables: scope.variables,
            value: Box::new(value),
        }
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl Display for ScriptInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.value
            .to_output()
            .map_err(|_| std::fmt::Error::default())?
            .fmt(f)
    }
}

impl Accessible for ScriptInstance {
    fn get(&self, identifier: &str) -> Option<Value> {
        let val = self
            .arguments
            .get(identifier)
            .or_else(|| self.variables.get(identifier))?;
        Some(*val.clone())
    }
}

pub fn eval(
    doc: &Document,
    arguments: HashMap<String, Value>,
    ctx: &EvalContext,
) -> Result<ScriptInstance, RuntimeError> {
    let mut scope = Scope::new(arguments);

    for statement in doc.statements() {
        match statement {
            Statement::Variable { name, value } => match value {
                Some(value) => {
                    let value = eval_expression(&scope, value, ctx)?;
                    scope.set(name.clone(), value);
                }
                None => {
                    if scope.get(name).is_none() {
                        return Err(RuntimeError::UnsetParameter(name.to_string()));
                    }
                }
            },
            Statement::Return(e) => {
                let value = eval_expression(&scope, e, ctx)?;
                return Ok(ScriptInstance::from_scope(value, scope));
            }
        }
    }

    Err(RuntimeError::NoReturnValue())
}

fn eval_expression(
    instance: &Scope,
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
            let argument_types = argument_values
                .iter()
                .map(|(name, value)| (name.as_str(), value.get_type()))
                .collect();

            let doc = ctx.documents.get(path);
            match doc {
                None => match ctx.library.find(path, &argument_types) {
                    None => Err(RuntimeError::UnknownIdentifier(path.to_string())),
                    Some(f) => Ok(f(&argument_values)?),
                },
                Some(doc) => {
                    for name in arguments.keys() {
                        if !doc.has_identifier(name) {
                            return Err(RuntimeError::ArgumentDoesNotExist(
                                path.to_string(),
                                name.to_string(),
                            ));
                        }
                    }

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
    instance: &Scope,
    ctx: &EvalContext,
    l: &Expression,
    name: &str,
) -> Result<Value, RuntimeError> {
    let l = eval_expression(instance, l.deref(), ctx)?;

    let script = l.to_accessible();
    if let Some(instance) = script {
        match instance.get(name) {
            None => Err(RuntimeError::MissingProperty(name.to_owned())),
            Some(v) => Ok(v),
        }
    } else {
        Err(RuntimeError::MissingProperty(String::from(name)))
    }
}

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
    UnexpectedType(Value),
    #[error("Script did not return a value")]
    NoReturnValue(),
    #[error("Cant Write")]
    CantWrite(),
    #[error("Mismatched types between {0} and {1}")]
    MismatchedTypes(Value, Value),
}
