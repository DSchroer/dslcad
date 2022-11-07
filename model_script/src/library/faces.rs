use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use opencascade::{Edge, Point};
use crate::library::number;
use crate::library::point;
use crate::runtime::RuntimeError;
use crate::syntax::Value;

pub fn point(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let x = number!(args, "x");
    let y = number!(args, "y");

    Ok(Value::Point(Rc::new(RefCell::new(Point::new(
        x, y, 0.,
    )))))
}

pub fn line(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let start = point!(args, "start").borrow();
    let end = point!(args, "end").borrow();

    let mut edge = Edge::new();
    edge.add_line(&start, &end);
    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}

pub fn arc(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let start = point!(args, "start").borrow();
    let center = point!(args, "center").borrow();
    let end = point!(args, "end").borrow();

    let mut edge = Edge::new();
    edge.add_arc(&start, &center, &end);
    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}