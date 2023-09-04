mod access;
mod output;
mod runtime_error;
mod scope;
mod script_instance;
mod stack;
mod types;
mod value;

use crate::library::{CallSignature, Library};
use crate::parser::{ArgName, Ast, CallPath, DocId, Expression, Literal, Statement};
use crate::runtime::scope::Scope;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

pub use access::Access;
pub use stack::WithStack;
pub use types::Type;
pub use value::Value;

use crate::runtime::stack::{Stack, StackFrame};
pub use runtime_error::RuntimeError;
pub use script_instance::ScriptInstance;

const MAX_STACK_SIZE: usize = 255;

pub struct Engine<'a> {
    library: &'a Library,
    ast: Ast,
    stack: Stack,
}

impl<'a> Engine<'a> {
    pub fn new(library: &'a Library, ast: Ast) -> Self {
        Engine {
            library,
            ast,
            stack: Stack::new(),
        }
    }

    pub fn eval_root(
        &mut self,
        arguments: HashMap<&str, Value>,
    ) -> Result<ScriptInstance, WithStack<RuntimeError>> {
        let root = self.ast.root().clone();
        self.eval(root, arguments)
    }

    fn eval(
        &mut self,
        id: DocId,
        arguments: HashMap<&str, Value>,
    ) -> Result<ScriptInstance, WithStack<RuntimeError>> {
        let mut scope = Scope::new(arguments);

        let statements = self
            .ast
            .documents
            .get(&id)
            .ok_or_else(|| {
                WithStack::from_err(RuntimeError::UnknownIdentifier(id.to_string()), &self.stack)
            })?
            .clone();

        let mut ret = None;
        for statement in &statements {
            self.stack.push(StackFrame::from_statement(&id, statement));

            if self.stack.len() >= MAX_STACK_SIZE {
                return Err(WithStack::from_err(
                    RuntimeError::StackOverflow(),
                    &self.stack,
                ));
            }

            match statement {
                Statement::Variable { name, value, .. } => match value {
                    Some(value) => {
                        let value = self.expression(&scope, value)?;
                        scope.set(name.to_string(), value);
                    }
                    None => {
                        if scope.get(name).is_none() {
                            return Err(WithStack::from_err(
                                RuntimeError::UnsetParameter(name.to_string()),
                                &self.stack,
                            ));
                        }
                    }
                },
                Statement::Return(e, _) => {
                    let value = self.expression(&scope, e)?;
                    ret = match ret {
                        None => Some(value),
                        Some(v) => Some(match v.get_type() {
                            Type::List => {
                                let mut items = v.to_list().unwrap();
                                items.push(value);
                                Value::List(items)
                            }
                            _ => Value::List(vec![v, value]),
                        }),
                    }
                }
            }
            self.stack.pop();
        }

        match ret {
            None => Err(WithStack::from_err(
                RuntimeError::NoReturnValue(),
                &self.stack,
            )),
            Some(value) => Ok(ScriptInstance::from_scope(value, scope)),
        }
    }

    fn expression(
        &mut self,
        instance: &Scope,
        expression: &Expression,
    ) -> Result<Value, WithStack<RuntimeError>> {
        match expression {
            Expression::Invocation {
                path, arguments, ..
            } => match path {
                CallPath::String(path) => {
                    let mut argument_values = HashMap::new();
                    for (name, argument) in arguments.iter() {
                        let value = self.expression(instance, argument.deref())?;
                        match name {
                            ArgName::Named(name) => {
                                argument_values.insert(name.as_str(), value);
                            }
                            ArgName::Default => {
                                let name = self
                                    .library
                                    .default_argument_name(path, value.get_type())
                                    .map_err(|e| WithStack::from_err(e, &self.stack))?;
                                argument_values.insert(name, value);
                            }
                        }
                    }

                    let argument_types = argument_values
                        .iter()
                        .map(|(name, value)| (*name, value.get_type()))
                        .collect();

                    let f = self
                        .library
                        .find(CallSignature::new(path, &argument_types))
                        .map_err(|e| WithStack::from_err(e, &self.stack))?;
                    Ok(f(&argument_values).map_err(|e| WithStack::from_err(e, &self.stack))?)
                }
                CallPath::Document(id) => {
                    let mut argument_values = HashMap::new();
                    for (name, argument) in arguments.iter() {
                        let value = self.expression(instance, argument.deref())?;
                        match name {
                            ArgName::Named(name) => {
                                argument_values.insert(name.as_str(), value);
                            }
                            ArgName::Default => {
                                return Err(WithStack::from_err(
                                    RuntimeError::UnknownDefaultArgument {
                                        name: id.to_string(),
                                    },
                                    &self.stack,
                                ))
                            }
                        }
                    }

                    let v = self.eval(id.clone(), argument_values)?;
                    Ok(Value::Script(Rc::new(RefCell::new(v))))
                }
            },
            Expression::Reference(n, _) => {
                if let Some(value) = instance.get(n) {
                    Ok(value.clone())
                } else {
                    Err(WithStack::from_err(
                        RuntimeError::UnknownIdentifier(n.to_string()),
                        &self.stack,
                    ))
                }
            }
            Expression::Access(l, name, _) => self.access(instance, l, name),
            Expression::Literal(literal, _) => Ok(match literal {
                Literal::Number(n) => Value::Number(*n),
                Literal::Bool(b) => Value::Bool(*b),
                Literal::Text(t) => Value::Text(t.clone()),
                Literal::List(items) => {
                    let mut values = Vec::new();
                    for item in items {
                        values.push(self.expression(instance, item)?);
                    }
                    Value::List(values)
                }
            }),
            Expression::Map {
                identifier,
                action,
                range,
                ..
            } => {
                let range_value = self.expression(instance, range)?;
                let range_value = range_value
                    .to_list()
                    .ok_or_else(|| RuntimeError::UnexpectedType(range_value.get_type()))
                    .map_err(|e| WithStack::from_err(e, &self.stack))?;

                let mut loop_scope = instance.clone();
                let mut results = Vec::new();
                for v in range_value {
                    loop_scope.set(identifier.to_string(), v);
                    results.push(self.expression(&loop_scope, action)?);
                }

                Ok(Value::List(results))
            }
            Expression::Reduce {
                left,
                right,
                action,
                range,
                root,
                ..
            } => {
                let range_value = self.expression(instance, range)?;
                let mut range_value = range_value
                    .to_list()
                    .ok_or_else(|| RuntimeError::UnexpectedType(range_value.get_type()))
                    .map_err(|e| WithStack::from_err(e, &self.stack))?;
                range_value.reverse();

                let mut loop_scope = instance.clone();
                let mut result = if let Some(expr) = root {
                    self.expression(instance, expr)?
                } else {
                    range_value
                        .pop()
                        .ok_or(RuntimeError::EmptyReduce())
                        .map_err(|e| WithStack::from_err(e, &self.stack))?
                };
                for v in range_value.into_iter().rev() {
                    loop_scope.set(left.to_string(), result.clone());
                    loop_scope.set(right.to_string(), v);
                    result = self.expression(&loop_scope, action)?;
                }

                Ok(result)
            }
            Expression::If {
                condition,
                if_true,
                if_false,
                ..
            } => {
                let condition_value = self.expression(instance, condition)?;
                match condition_value.to_bool() {
                    Some(true) => Ok(self.expression(instance, if_true)?),
                    Some(false) => Ok(self.expression(instance, if_false)?),
                    None => Err(WithStack::from_err(
                        RuntimeError::UnexpectedType(condition_value.get_type()),
                        &self.stack,
                    )),
                }
            }
            Expression::Index { target, index, .. } => {
                let target_value = self.expression(instance, target)?;
                let index_value = self.expression(instance, index)?;

                let list = target_value
                    .to_list()
                    .ok_or_else(|| RuntimeError::UnexpectedType(target_value.get_type()))
                    .map_err(|e| WithStack::from_err(e, &self.stack))?;
                let index = index_value
                    .to_number()
                    .ok_or_else(|| RuntimeError::UnexpectedType(index_value.get_type()))
                    .map_err(|e| WithStack::from_err(e, &self.stack))?;
                Ok(list[index.round() as usize].clone())
            }
        }
    }

    fn access(
        &mut self,
        instance: &Scope,
        l: &Expression,
        name: &str,
    ) -> Result<Value, WithStack<RuntimeError>> {
        let l = self.expression(instance, l)?;

        let script = l.to_accessible();
        if let Some(instance) = script {
            match instance.get(name) {
                None => Err(WithStack::from_err(
                    RuntimeError::MissingProperty(name.to_owned()),
                    &self.stack,
                )),
                Some(v) => Ok(v),
            }
        } else {
            Err(WithStack::from_err(
                RuntimeError::MissingProperty(String::from(name)),
                &self.stack,
            ))
        }
    }
}
