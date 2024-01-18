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

unsafe impl Send for Value {}

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

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::List(value)
    }
}

impl From<Rc<Point>> for Value {
    fn from(value: Rc<Point>) -> Self {
        Value::Point(value)
    }
}

impl From<Rc<Wire>> for Value {
    fn from(value: Rc<Wire>) -> Self {
        Value::Line(value)
    }
}

impl From<Rc<Shape>> for Value {
    fn from(value: Rc<Shape>) -> Self {
        Value::Shape(value)
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
            Value::List(i) => f.debug_list().entries(i).finish(),
            Value::Script(_) => f.debug_tuple("Script").finish(),
            Value::Shape(_) => f.debug_tuple("Shape").finish(),
            Value::Point(p) => f
                .debug_struct("Point")
                .field("x", &p.x())
                .field("y", &p.x())
                .field("z", &p.x())
                .finish(),
            Value::Line(_) => f.debug_tuple("Line").finish(),
        }
    }
}

impl Value {
    pub fn flatten(&self) -> Vec<&Value> {
        match self {
            Value::Number(_) => vec![self],
            Value::Bool(_) => vec![self],
            Value::Text(_) => vec![self],
            Value::Point(_) => vec![self],
            Value::Line(_) => vec![self],
            Value::Shape(_) => vec![self],
            Value::List(list) => list.iter().flat_map(|l| l.flatten()).collect(),
            Value::Script(s) => s.value().flatten(),
        }
    }

    pub fn to_output(&self, deflection: f64) -> Result<Part> {
        match self {
            Value::Number(_) | Value::Bool(_) | Value::Text(_) => Ok(Part::Empty),
            Value::Point(p) => Ok(p.into_part(deflection)?),
            Value::Line(l) => Ok(l.into_part(deflection)?),
            Value::Shape(s) => Ok(s.into_part(deflection)?),
            Value::List(_) | Value::Script(_) => {
                panic!("can not be turned into Part directly, use `flatten` first")
            }
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
            Value::List(l) => Ok(l
                .iter()
                .filter_map(|i| i.to_text().ok())
                .reduce(|a, b| format!("{a}\n{b}"))
                .ok_or(RuntimeError::UnexpectedType())?),
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
            Value::List(values) => {
                let lines: Vec<_> = values.iter().filter_map(|v| v.to_line().ok()).collect();
                Self::fuse_list(&lines)
            }
            _ => Err(RuntimeError::UnexpectedType()),
        }
    }

    pub fn to_shape(&self) -> Result<Rc<Shape>> {
        match self {
            Value::Shape(s) => Ok(s.clone()),
            Value::Script(i) => i.value().to_shape(),
            Value::List(values) => {
                let shapes: Vec<_> = values.iter().filter_map(|v| v.to_shape().ok()).collect();
                Self::fuse_list(&shapes)
            }
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

    pub fn to_type(&self, target: Type) -> Result<Value> {
        match target {
            Type::Number => Ok(self.to_number()?.into()),
            Type::Bool => Ok(self.to_bool()?.into()),
            Type::Text => Ok(self.to_text()?.into()),
            Type::List => Ok(self.to_list()?.into()),
            Type::Point => Ok(self.to_point()?.into()),
            Type::Edge => Ok(self.to_line()?.into()),
            Type::Shape => Ok(self.to_shape()?.into()),
        }
    }

    fn fuse_list<T: DsShape>(lines: &[Rc<T>]) -> Result<Rc<T>> {
        match lines.len() {
            0 => Err(RuntimeError::UnexpectedType()),
            1 => Ok(lines[0].clone()),
            _ => {
                let mut acc = lines[0].fuse(lines[1].as_ref())?;
                for value in &lines[2..] {
                    acc = acc.fuse(value.as_ref())?
                }
                Ok(Rc::new(acc))
            }
        }
    }
}
