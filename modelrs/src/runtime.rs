use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::ops::{Add, Deref, Div, Mul, Sub};
use std::rc::Rc;
use thiserror::Error;
use crate::library::Library;
use crate::parser::Document;
use crate::syntax::Instance;
use crate::syntax::{*};

pub struct EvalContext {
    pub library: Library,
    pub documents: HashMap<String, Document>
}

#[derive(Debug, Clone)]
pub struct ScriptInstance {
    arguments: HashMap<String, Box<Value>>,
    variables: HashMap<String, Box<Value>>,
    value: Box<Value>
}

impl ScriptInstance {
    pub fn new(arguments: HashMap<String, Value>) -> Self {
        ScriptInstance {
            arguments: arguments.into_iter().map(|(k,v)|(k, Box::new(v))).collect(),
            variables: HashMap::new(),
            value: Box::new(Value::Empty)
        }
    }

    pub fn set(&mut self, identifier: String, value: Value) {
        self.variables.insert(identifier, Box::new(value));
    }

    pub fn set_value(&mut self, value: Value) {
        self.value = Box::new(value);
    }
}

impl Instance for ScriptInstance {
    fn get(&self, identifier: &str) -> Option<&Box<Value>> {
        self.arguments.get(identifier).or_else(||self.variables.get(identifier))
    }

    fn value(&self) -> &Box<Value> {
        &self.value
    }

    fn write_to_file(&mut self, path: &str) -> Result<(), Error> {
        let instance = self.value.to_shape().ok_or(std::io::Error::from(ErrorKind::Other))?;
        unsafe {
            instance.borrow_mut().write_to_file(path)
        }

    }
}

pub fn eval(doc: &Document, arguments: HashMap<String, Value>, ctx: &EvalContext) -> Result<ScriptInstance, RuntimeError> {
    let mut instance = ScriptInstance::new(arguments);

    for statement in doc.statements() {
        match statement {
            Statement::Variable{name, value} => {
                match value {
                    Some(value) => {
                        let value = eval_expression(&instance, value, ctx)?;
                        instance.set(name.clone(), value);
                    },
                    None => {
                        if instance.get(name).is_none() {
                            return Err(RuntimeError::UnsetParameter(name.to_string()))
                        }
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

fn eval_expression(instance: &dyn Instance, expression: &Expression, ctx: &EvalContext) -> Result<Value, RuntimeError> {
    match expression {
        Expression::Literal(v) => Ok(v.clone()),
        Expression::Invocation{path, arguments} => {
            let arguments = arguments.clone().into_iter()
                .map(|(k,v)|(k, eval_expression(instance, v.deref(), ctx).unwrap()))
                .collect();

            let doc = ctx.documents.get(path);
            match doc {
                None => {
                    match ctx.library.find(path) {
                        None => Err(RuntimeError::UnknownIdentifier(path.to_string())),
                        Some(f) => Ok(f(&arguments)?)
                    }
                },
                Some(doc) => {
                    let v = eval(doc, arguments, ctx)?;
                    return Ok(Value::Script(Rc::new(RefCell::new(v))))
                }
            }
        },
        Expression::Reference(n) => {
            if let Some(value) = instance.get(&n) {
                Ok(*value.clone())
            } else {
                Err(RuntimeError::UnknownIdentifier(n.to_string()))
            }
        },
        Expression::Access(l, name) => access(instance, ctx, l, name),
        Expression::Add(l, r) => numeric(instance, ctx, l, r, f64::add),
        Expression::Subtract(l, r) => numeric(instance, ctx, l, r, f64::sub),
        Expression::Multiply(l, r) => numeric(instance, ctx, l, r, f64::mul),
        Expression::Divide(l, r) => numeric(instance, ctx, l, r, f64::div),
    }
}

fn access(instance: &dyn Instance, ctx: &EvalContext, l: &Box<Expression>, name: &String) -> Result<Value, RuntimeError>  {
    let l = eval_expression(instance, l.deref(), ctx)?;

    let lv = l.to_script();

    if lv.is_some() {
        match lv.unwrap().borrow().get(name) {
            None => Err(RuntimeError::MissingProperty(name.clone())),
            Some(v) => Ok(*v.clone())
        }
    } else {
        Err(RuntimeError::UnexpectedType(l))
    }
}

fn numeric(instance: &dyn Instance, ctx: &EvalContext, l: &Box<Expression>, r: &Box<Expression>, op: impl FnOnce(f64, f64) -> f64) -> Result<Value, RuntimeError>  {
    let l = eval_expression(instance, l.deref(), ctx)?;
    let r = eval_expression(instance, r.deref(), ctx)?;

    let lv = l.to_number();
    let rv = r.to_number();

    if lv.is_some() && rv.is_some() {
        Ok(Value::Number(op(lv.unwrap(), rv.unwrap())))
    } else {
        Err(RuntimeError::MismatchedTypes(l, r))
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
    MismatchedTypes(Value, Value)
}
