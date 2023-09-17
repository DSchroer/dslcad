use crate::runtime::output::IntoPart;
use persistence::protocol::Part;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use super::Access;
use super::Type;
use crate::runtime::{RuntimeError, ScriptInstance};
use opencascade::{DsShape, Point, Shape, Wire};

type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Text(String),

    Point(Rc<Point>),
    Line(Rc<Wire>),
    Shape(Rc<Shape>),

    List(Vec<Value>),

    Script(Rc<ScriptInstance>),
}

impl From<Point> for Value {
    fn from(value: Point) -> Self {
        Value::Point(Rc::new(value))
    }
}

impl From<Wire> for Value {
    fn from(value: Wire) -> Self {
        Value::Line(Rc::new(value))
    }
}

impl From<Shape> for Value {
    fn from(value: Shape) -> Self {
        Value::Shape(Rc::new(value))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<ScriptInstance> for Value {
    fn from(value: ScriptInstance) -> Self {
        Value::Script(Rc::new(value))
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Value::Bool(n) => f.debug_tuple("Bool").field(n).finish(),
            Value::Text(n) => f.debug_tuple("Text").field(n).finish(),
            Value::List(_) => f.debug_tuple("List").finish(),
            Value::Script(_) => f.debug_tuple("Script").finish(),
            Value::Shape(_) => f.debug_tuple("Shape").finish(),
            Value::Point(_) => f.debug_tuple("Point").finish(),
            Value::Line(_) => f.debug_tuple("Line").finish(),
        }
    }
}

impl Value {
    pub fn to_output(&self) -> Result<Vec<Part>> {
        match self {
            Value::Number(_) | Value::Bool(_) | Value::Text(_) => {
                Err(RuntimeError::UnexpectedType())
            }

            Value::List(l) => {
                let mut results = Vec::new();
                for item in l {
                    results.push(item.to_output()?)
                }
                Ok(results.concat())
            }

            Value::Script(s) => Ok(s.value().to_output()?),

            Value::Point(p) => Ok(vec![p.into_part()?]),
            Value::Line(l) => Ok(vec![l.into_part()?]),
            Value::Shape(s) => Ok(vec![s.into_part()?]),
        }
    }

    pub fn to_number(&self) -> Result<f64> {
        match self {
            Value::Number(f) => Ok(*f),
            Value::Script(i) => i.value().to_number(),
            Value::List(l) if l.len() == 1 => l[0].to_number(),
            _ => Err(RuntimeError::UnexpectedType()),
        }
    }

    pub fn to_text(&self) -> Result<String> {
        match self {
            Value::Text(f) => Ok(f.clone()),
            Value::Script(i) => i.value().to_text(),
            Value::List(l) if l.len() == 1 => l[0].to_text(),
            _ => Err(RuntimeError::UnexpectedType()),
        }
    }

    pub fn to_bool(&self) -> Result<bool> {
        match self {
            Value::Bool(f) => Ok(*f),
            Value::Script(i) => i.value().to_bool(),
            Value::List(l) if l.len() == 1 => l[0].to_bool(),
            _ => Err(RuntimeError::UnexpectedType()),
        }
    }

    pub fn to_accessible(&self) -> Result<&dyn Access> {
        match self {
            Value::Script(i) => Ok(i.as_ref()),
            Value::Line(w) => Ok(w.as_ref()),
            Value::Shape(s) => Ok(s.as_ref()),
            Value::Point(p) => Ok(p.as_ref()),
            _ => Err(RuntimeError::UnexpectedType()),
        }
    }

    pub fn to_point(&self) -> Result<Rc<Point>> {
        match self {
            Value::Point(s) => Ok(s.clone()),
            Value::Script(i) => i.value().to_point(),
            Value::List(l) if l.len() == 1 => l[0].to_point(),
            _ => Err(RuntimeError::UnexpectedType()),
        }
    }

    pub fn to_line(&self) -> Result<Rc<Wire>> {
        match self {
            Value::Line(s) => Ok(s.clone()),
            Value::Script(i) => i.value().to_line(),
            Value::List(l) if l.len() == 1 => l[0].to_line(),
            _ => Err(RuntimeError::UnexpectedType()),
        }
    }

    pub fn to_shape(&self) -> Result<Rc<Shape>> {
        match self {
            Value::Shape(s) => Ok(s.clone()),
            Value::Script(i) => i.value().to_shape(),
            Value::List(values) => match values.len() {
                0 => Err(RuntimeError::UnexpectedType()),
                1 => values[0].to_shape(),
                _ => {
                    let mut acc = values[0].to_shape()?.fuse(values[1].to_shape()?.as_ref())?;
                    for value in &values[2..] {
                        acc = acc.fuse(value.to_shape()?.as_ref())?
                    }
                    Ok(Rc::new(acc))
                }
            },
            _ => Err(RuntimeError::UnexpectedType()),
        }
    }

    pub fn to_list(&self) -> Result<Vec<Value>> {
        match self {
            Value::List(s) => Ok(s.clone()),
            Value::Script(i) => i.value().to_list(),
            _ => Err(RuntimeError::UnexpectedType()),
        }
    }

    pub fn is_type(&self, target: Type) -> bool {
        match target {
            Type::Number => self.to_number().is_ok(),
            Type::Bool => self.to_bool().is_ok(),
            Type::Text => self.to_text().is_ok(),
            Type::List => self.to_list().is_ok(),
            Type::Point => self.to_point().is_ok(),
            Type::Edge => self.to_line().is_ok(),
            Type::Shape => self.to_shape().is_ok(),
        }
    }
}
