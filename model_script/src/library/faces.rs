use crate::runtime::RuntimeError;
use crate::syntax::Value;
use opencascade::{Axis, Edge, Point, Shape};
use std::cell::RefCell;
use std::rc::Rc;

pub fn point(x: Option<f64>, y: Option<f64>, z: Option<f64>) -> Result<Value, RuntimeError> {
    Ok(Value::Point(Rc::new(RefCell::new(Point::new(
        x.unwrap_or(0.0),
        y.unwrap_or(0.0),
        z.unwrap_or(0.0),
    )))))
}

pub fn line(start: Rc<RefCell<Point>>, end: Rc<RefCell<Point>>) -> Result<Value, RuntimeError> {
    let mut edge = Edge::new();
    edge.add_line(&start.borrow(), &end.borrow());
    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}

pub fn arc(
    start: Rc<RefCell<Point>>,
    center: Rc<RefCell<Point>>,
    end: Rc<RefCell<Point>>,
) -> Result<Value, RuntimeError> {
    let mut edge = Edge::new();
    edge.add_arc(&start.borrow(), &center.borrow(), &end.borrow());
    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}

pub fn extrude(
    shape: Rc<RefCell<Edge>>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(Shape::extrude(
        &mut shape,
        x.unwrap_or(0.0),
        y.unwrap_or(0.0),
        z.unwrap_or(0.0),
    )))))
}

pub fn revolve(
    shape: Rc<RefCell<Edge>>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let (axis, angle) = if let Some(x) = x {
        (Axis::X, x)
    } else if let Some(y) = y {
        (Axis::Y, y)
    } else if let Some(z) = z {
        (Axis::Z, z)
    } else {
        return Err(RuntimeError::UnsetParameter(String::from("x, y, or z")));
    };

    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(Shape::extrude_rotate(
        &mut shape, axis, angle,
    )))))
}

pub fn union_edge(
    left: Rc<RefCell<Edge>>,
    right: Rc<RefCell<Edge>>,
) -> Result<Value, RuntimeError> {
    let mut left = left.borrow_mut();
    let mut right = right.borrow_mut();

    let mut edge = Edge::new();
    edge.join(&mut left, &mut right);
    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}
