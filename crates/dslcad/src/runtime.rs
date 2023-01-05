mod access;
mod output;
mod runtime_error;
mod scope;
mod script_instance;
mod types;
mod value;

use crate::library::{CallSignature, Library};
use crate::parser::Document;
use crate::parser::{Expression, Literal, Statement};
use crate::runtime::scope::Scope;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

pub use access::Access;
pub use output::Output;
pub use types::Type;
pub use value::Value;

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
                None => {
                    let f = ctx
                        .library
                        .find(CallSignature::new(path, &argument_types))?;
                    Ok(f(&argument_values)?)
                }
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
        Expression::Literal(literal) => Ok(match literal {
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
        }),
        Expression::Map {
            identifier,
            action,
            range,
        } => {
            let range_value = eval_expression(instance, range, ctx)?;
            let range_value = range_value
                .to_list()
                .ok_or_else(|| RuntimeError::UnexpectedType(range_value.get_type()))?;

            let mut loop_scope = instance.clone();
            let mut results = Vec::new();
            for v in range_value {
                loop_scope.set(identifier.clone(), v);
                results.push(eval_expression(&loop_scope, action, ctx)?);
            }

            Ok(Value::List(results))
        }
        Expression::Reduce {
            left,
            right,
            action,
            range,
            root,
        } => {
            let range_value = eval_expression(instance, range, ctx)?;
            let mut range_value = range_value
                .to_list()
                .ok_or_else(|| RuntimeError::UnexpectedType(range_value.get_type()))?;
            range_value.reverse();

            let mut loop_scope = instance.clone();
            let mut result = if let Some(expr) = root {
                eval_expression(instance, expr, ctx)?
            } else {
                range_value.pop().ok_or(RuntimeError::EmptyReduce())?
            };
            for v in range_value.into_iter().rev() {
                loop_scope.set(left.clone(), result.clone());
                loop_scope.set(right.clone(), v);
                result = eval_expression(&loop_scope, action, ctx)?;
            }

            Ok(result)
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
