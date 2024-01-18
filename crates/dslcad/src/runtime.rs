mod access;
mod output;
mod runtime_error;
mod scope;
mod script_instance;
mod stack;
mod types;
mod value;

use crate::library::{ArgValue, CallSignature, Library};
use crate::parser::{Argument, Ast, CallPath, DocId, Expression, Literal, Statement};
use crate::runtime::scope::Scope;
use log::trace;
use logos::Span;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Instant;

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
        arguments: HashMap<&str, Literal>,
    ) -> Result<Value, WithStack<RuntimeError>> {
        let root = self.ast.root().clone();
        let arguments = arguments
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    self.expression(&Scope::default(), &Expression::Literal(v, Span::default()))
                        .unwrap(),
                )
            })
            .collect();
        Ok(self.eval(root, arguments)?.into())
    }

    fn eval(
        &mut self,
        id: DocId,
        arguments: HashMap<&str, Value>,
    ) -> Result<ScriptInstance, WithStack<RuntimeError>> {
        let statements = self
            .ast
            .documents
            .get(&id)
            .ok_or_else(|| {
                WithStack::from_err(RuntimeError::UnknownIdentifier(id.to_string()), &self.stack)
            })?
            .clone();

        let scope = Scope::default();
        self.eval_statements(id, arguments, scope, &statements)
    }

    fn eval_statements(
        &mut self,
        id: DocId,
        arguments: HashMap<&str, Value>,
        mut scope: Scope,
        statements: &[Statement],
    ) -> Result<ScriptInstance, WithStack<RuntimeError>> {
        let mut ret = Vec::new();
        for statement in statements {
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
                        let value = if let Some(value) = arguments.get(name.as_str()).cloned() {
                            value
                        } else {
                            self.expression(&scope, value)?
                        };
                        scope.set(name.to_string(), value);
                    }
                    None => {
                        let value = if let Some(value) = arguments.get(name.as_str()).cloned() {
                            value
                        } else {
                            return Err(WithStack::from_err(
                                RuntimeError::UnsetParameter(name.to_string()),
                                &self.stack,
                            ));
                        };
                        scope.set(name.to_string(), value);
                    }
                },
                Statement::CreatePart(e, _) => {
                    ret.push(self.expression(&scope, e)?);
                }
            }
            self.stack.pop();
        }

        ScriptInstance::from_scope(ret, scope).map_err(|e| WithStack::from_err(e, &self.stack))
    }

    fn expression(
        &mut self,
        instance: &Scope,
        expression: &Expression,
    ) -> Result<Value, WithStack<RuntimeError>> {
        match expression {
            Expression::Invocation {
                path, arguments, ..
            } => {
                let argument_values =
                    arguments.iter().try_fold(Vec::new(), |mut acc, argument| {
                        match argument {
                            Argument::Named(name, expr) => {
                                let value = self.expression(instance, expr.deref())?;
                                acc.push(ArgValue::Named(name, value))
                            }
                            Argument::Unnamed(expr) => {
                                let value = self.expression(instance, expr.deref())?;
                                acc.push(ArgValue::Unnamed(value));
                            }
                        }
                        Ok(acc)
                    })?;

                match path {
                    CallPath::Function(path) => {
                        let timer = Instant::now();

                        let (f, a) = self
                            .library
                            .find(CallSignature::new(path, argument_values))
                            .map_err(|e| WithStack::from_err(e, &self.stack))?;
                        let res = f(&a).map_err(|e| WithStack::from_err(e, &self.stack))?;

                        if timer.elapsed().as_millis() != 0 {
                            trace!(
                                "{:?}(..) executed in {}ms",
                                path,
                                timer.elapsed().as_millis()
                            );
                        }
                        Ok(res)
                    }
                    CallPath::Document(id) => {
                        let named_argument_values = argument_values
                            .into_iter()
                            .try_fold(HashMap::new(), |mut acc, v| {
                                match v {
                                    ArgValue::Named(name, val) => {
                                        acc.insert(name, val);
                                    }
                                    ArgValue::Unnamed(_) => {
                                        return Err(RuntimeError::UnknownDefaultArgument {
                                            name: "".to_string(),
                                        })
                                    }
                                }
                                Ok(acc)
                            })
                            .map_err(|e| WithStack::from_err(e, &self.stack))?;

                        let v = self.eval(id.clone(), named_argument_values)?;
                        Ok(Value::Script(Rc::new(v)))
                    }
                }
            }
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
                Literal::Resource(r) => r
                    .to_instance()
                    .map_err(|e| WithStack::from_err(e, &self.stack))?,
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
                    .map_err(|e| WithStack::from_err(e, &self.stack))?;

                let mut loop_scope = instance.clone();
                let mut results = Vec::new();
                for v in range_value {
                    loop_scope.set(identifier.to_string(), v.clone());
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
                    .map_err(|e| WithStack::from_err(e, &self.stack))?
                    .clone();
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
                    loop_scope.set(right.to_string(), v.clone());
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
                if condition_value
                    .to_bool()
                    .map_err(|e| WithStack::from_err(e, &self.stack))?
                {
                    Ok(self.expression(instance, if_true)?)
                } else {
                    Ok(self.expression(instance, if_false)?)
                }
            }
            Expression::Index { target, index, .. } => {
                let target_value = self.expression(instance, target)?;
                let index_value = self.expression(instance, index)?;

                let list = target_value
                    .to_list()
                    .map_err(|e| WithStack::from_err(e, &self.stack))?;
                let index = index_value
                    .to_number()
                    .map_err(|e| WithStack::from_err(e, &self.stack))?;
                Ok(list[index.round() as usize].clone())
            }
            Expression::Scope { statements, .. } => {
                let inst = self.eval_statements(
                    DocId::new("scope".to_string()),
                    HashMap::new(),
                    instance.clone(),
                    statements,
                )?;
                Ok(Value::Script(Rc::new(inst)))
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

        let script = l
            .to_accessible()
            .map_err(|e| WithStack::from_err(e, &self.stack))?;
        match script.get(name) {
            None => Err(WithStack::from_err(
                RuntimeError::MissingProperty(name.to_owned()),
                &self.stack,
            )),
            Some(v) => Ok(v),
        }
    }
}
