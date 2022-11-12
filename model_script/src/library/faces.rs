use crate::library::edge;
use crate::library::number;
use crate::library::point;
use crate::runtime::RuntimeError;
use crate::syntax::Value;
use opencascade::{Axis, Edge, Point, Shape};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// syntax[2d]: `point(x=number,y=number)`  Create a new 2D point
pub fn point(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let x = number!(args, "x", 0.);
    let y = number!(args, "y", 0.);

    Ok(Value::Point(Rc::new(RefCell::new(Point::new(x, y, 0.0)))))
}

/// syntax[2d]: `line(start=point,end=point)`  Create a line between two points
pub fn line(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let start = point!(args, "start")?;
    let end = point!(args, "end")?;

    let mut edge = Edge::new();
    edge.add_line(&start.borrow(), &end.borrow());
    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}

/// syntax[2d]: `arc(start=point,center=point,end=point)`  Create an arcing line between three points
pub fn arc(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let start = point!(args, "start")?;
    let center = point!(args, "center")?;
    let end = point!(args, "end")?;

    let mut edge = Edge::new();
    edge.add_arc(&start.borrow(), &center.borrow(), &end.borrow());
    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}

/// syntax[2d]: `extrude(shape=line,height=number)`  Extrude a face into a 3d shape
pub fn extrude(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let shape = edge!(args, "shape")?;
    let height = number!(args, "height");

    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(Shape::extrude(
        &mut shape, height,
    )))))
}

/// syntax[2d]: `revolve(shape=line,x=number,y=number)`  Extrude a face into a 3d shape around x or y
pub fn revolve(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let shape = edge!(args, "shape")?;
    let x = number!(args, "x", 0.);
    let y = number!(args, "y", 0.);

    let (axis, angle) = if x != 0.0 { (Axis::X, x) } else { (Axis::Y, y) };

    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(Shape::extrude_rotate(
        &mut shape, axis, angle,
    )))))
}
