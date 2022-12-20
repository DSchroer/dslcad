mod runtime_error;
mod scope;
mod script_instance;
mod value;
mod output;
mod accessible;
mod types;

use crate::library::Library;
use crate::parser::Document;
use crate::runtime::scope::Scope;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use crate::parser::{Statement, Expression, Literal};

pub use output::Output;
pub use accessible::Accessible;
pub use value::Value;
pub use types::Type;

pub use runtime_error::RuntimeError;
pub use script_instance::ScriptInstance;

pub struct EvalContext<'a> {
    pub library: &'a Library,
    pub documents: &'a HashMap<String, Document>,
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
        Expression::Literal(literal) => {
            Ok(match literal {
                Literal::Number(n) => Value::Number(*n),
                Literal::Bool(b) => Value::Bool(*b),
                Literal::Text(t) => Value::Text(t.clone()),
                Literal::List(items) => {
                    let mut values = Vec::new();
                    for item in items {
                        values.push(eval_expression(instance, item, ctx)?);
                    }
                    Value::List(values)
                }
            })
        }
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
