use super::from_cascade;
use crate::runtime::{RuntimeError, Type, Value};
use opencascade::{Axis, Edge, Point, Shape};
use std::cell::RefCell;
use std::rc::Rc;

pub fn point(x: Option<f64>, y: Option<f64>) -> Result<Value, RuntimeError> {
    Ok(Value::Point(Rc::new(RefCell::new(Point::new(
        x.unwrap_or(0.0),
        y.unwrap_or(0.0),
        0.0,
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

pub fn square(x: Option<f64>, y: Option<f64>) -> Result<Value, RuntimeError> {
    let mut edge = Edge::new();

    let a = Point::default();
    let b = Point::new(0.0, y.unwrap_or(1.0), 0.0);
    let c = Point::new(x.unwrap_or(1.0), y.unwrap_or(1.0), 0.0);
    let d = Point::new(x.unwrap_or(1.0), 0.0, 0.0);

    edge.add_line(&a, &b);
    edge.add_line(&b, &c);
    edge.add_line(&c, &d);
    edge.add_line(&d, &a);

    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}

pub fn circle(radius: Option<f64>) -> Result<Value, RuntimeError> {
    let mut edge = Edge::new();

    let r = radius.unwrap_or(0.5);

    let a = Point::new_2d(0.0, r);
    let b = Point::new_2d(r, r * 2.0);
    let c = Point::new_2d(r * 2.0, r);
    let d = Point::new_2d(r, 0.0);

    edge.add_arc(&a, &b, &c);
    edge.add_arc(&c, &d, &a);

    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}

pub fn extrude(
    shape: Rc<RefCell<Edge>>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(from_cascade!(
        Shape::extrude(
            &mut shape,
            x.unwrap_or(0.0),
            y.unwrap_or(0.0),
            z.unwrap_or(0.0),
        )
    )?))))
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
    Ok(Value::Shape(Rc::new(RefCell::new(from_cascade!(
        Shape::extrude_rotate(&mut shape, axis, angle,)
    )?))))
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

pub fn face(mut parts: Vec<Value>) -> Result<Value, RuntimeError> {
    if parts.is_empty() {
        return point(None, None);
    }
    let parts_len = parts.len();

    let start = start_point(&mut parts[0])?;
    let end = end_point(&mut parts[parts_len - 1])?;

    if parts_len == 1 && start == end {
        return Ok(parts[0].clone());
    }

    let mut edge = Edge::new();
    for i in 0..parts_len {
        let last = if i == 0 { parts_len - 1 } else { i - 1 };
        let last_end = end_point(&mut parts[last])?;
        let current_start = start_point(&mut parts[i])?;
        let point = &parts[i];

        if last_end != current_start {
            edge.add_line(&last_end.borrow(), &current_start.borrow());
        }

        if point.get_type() == Type::Edge {
            edge.add_edge(&mut point.to_line().unwrap().borrow_mut())
        }
    }

    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}

fn start_point(value: &mut Value) -> Result<Rc<RefCell<Point>>, RuntimeError> {
    match value.get_type() {
        Type::Point => Ok(value.to_point().unwrap()),
        Type::Edge => Ok(Rc::new(RefCell::new(
            value.to_line().unwrap().borrow_mut().start().unwrap(),
        ))),
        other => Err(RuntimeError::UnexpectedType(other)),
    }
}

fn end_point(value: &mut Value) -> Result<Rc<RefCell<Point>>, RuntimeError> {
    match value.get_type() {
        Type::Point => Ok(value.to_point().unwrap()),
        Type::Edge => Ok(Rc::new(RefCell::new(
            value.to_line().unwrap().borrow_mut().end().unwrap(),
        ))),
        other => Err(RuntimeError::UnexpectedType(other)),
    }
}
