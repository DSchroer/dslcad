use crate::library::edge;
use crate::library::number;
use crate::library::point;
use crate::runtime::RuntimeError;
use crate::syntax::Value;
use opencascade::{Axis, Edge, Point, Shape};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn point(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let x = number!(args, "x", 0.);
    let y = number!(args, "y", 0.);
    let z = number!(args, "z", 0.);

    Ok(Value::Point(Rc::new(RefCell::new(Point::new(x, y, z)))))
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

pub fn join(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut left = edge!(args, "left").borrow_mut();
    let mut right = edge!(args, "right").borrow_mut();

    let mut edge = Edge::new();
    edge.join(&mut left, &mut right);
    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}

pub fn extrude(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut shape = edge!(args, "shape").borrow_mut();
    let height = number!(args, "height");

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::extrude(
        &mut shape, height,
    )))))
}

pub fn revolve(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut shape = edge!(args, "shape").borrow_mut();
    let x = number!(args, "x", 0.);
    let y = number!(args, "y", 0.);
    let z = number!(args, "z", 0.);

    let (axis, angle) = if x != 0.0 {
        (Axis::X, x)
    } else if y != 0.0 {
        (Axis::Y, y)
    } else {
        (Axis::Z, z)
    };

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::extrude_rotate(
        &mut shape, axis, angle,
    )))))
}
