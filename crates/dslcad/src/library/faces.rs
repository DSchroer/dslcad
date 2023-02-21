use crate::runtime::{RuntimeError, Type, Value};
use opencascade::{Axis, DsShape, Edge, Point, Shape, Wire, WireFactory};
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
    let mut edge = WireFactory::new();
    edge.add_edge(&Edge::new_line(&start.borrow(), &end.borrow())?);
    Ok(Value::Line(Rc::new(RefCell::new(edge.build()?))))
}

pub fn arc(
    start: Rc<RefCell<Point>>,
    center: Rc<RefCell<Point>>,
    end: Rc<RefCell<Point>>,
) -> Result<Value, RuntimeError> {
    let mut edge = WireFactory::new();
    edge.add_edge(&Edge::new_arc(
        &start.borrow(),
        &center.borrow(),
        &end.borrow(),
    )?);
    Ok(Value::Line(Rc::new(RefCell::new(edge.build()?))))
}

pub fn square(x: Option<f64>, y: Option<f64>, center: bool) -> Result<Value, RuntimeError> {
    let mut edge = WireFactory::new();

    let x0 = if center { -x.unwrap_or(1.0) / 2.0 } else { 0.0 };
    let y0 = if center { -y.unwrap_or(1.0) / 2.0 } else { 0.0 };

    let a = Point::new_2d(x0, y0);
    let b = Point::new_2d(x0, y0 + y.unwrap_or(1.0));
    let c = Point::new_2d(x0 + x.unwrap_or(1.0), y0 + y.unwrap_or(1.0));
    let d = Point::new_2d(x0 + x.unwrap_or(1.0), y0);

    edge.add_edge(&Edge::new_line(&a, &b)?);
    edge.add_edge(&Edge::new_line(&b, &c)?);
    edge.add_edge(&Edge::new_line(&c, &d)?);
    edge.add_edge(&Edge::new_line(&d, &a)?);

    Ok(Value::Line(Rc::new(RefCell::new(edge.build()?))))
}

pub fn circle(radius: Option<f64>, center: bool) -> Result<Value, RuntimeError> {
    let mut edge = WireFactory::new();

    let r = radius.unwrap_or(0.5);

    let x0 = if center { -r } else { 0.0 };
    let y0 = if center { -r } else { 0.0 };

    let a = Point::new_2d(x0, y0 + r);
    let b = Point::new_2d(x0 + r, y0 + (r * 2.0));
    let c = Point::new_2d(x0 + (r * 2.0), y0 + r);
    let d = Point::new_2d(x0 + r, y0);

    edge.add_edge(&Edge::new_arc(&a, &b, &c)?);
    edge.add_edge(&Edge::new_arc(&c, &d, &a)?);

    Ok(Value::Line(Rc::new(RefCell::new(edge.build()?))))
}

pub fn extrude(
    shape: Rc<RefCell<Wire>>,
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
    )?))))
}

pub fn revolve(
    shape: Rc<RefCell<Wire>>,
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
    )?))))
}

pub fn union_edge(
    left: Rc<RefCell<Wire>>,
    right: Rc<RefCell<Wire>>,
) -> Result<Value, RuntimeError> {
    let left = left.borrow();
    let right = right.borrow();

    let mut edge = WireFactory::new();
    edge.add_wire(&left);
    edge.add_wire(&right);
    Ok(Value::Line(Rc::new(RefCell::new(edge.build()?))))
}

pub fn face(mut parts: Vec<Value>) -> Result<Value, RuntimeError> {
    if parts.is_empty() {
        return point(None, None, None);
    }
    let parts_len = parts.len();

    let start = start_point(&mut parts[0])?;
    let end = end_point(&mut parts[parts_len - 1])?;

    if parts_len == 1 && start == end {
        return Ok(parts[0].clone());
    }

    let mut edge = WireFactory::new();
    for i in 0..parts_len {
        let last = if i == 0 { parts_len - 1 } else { i - 1 };
        let last_end = end_point(&mut parts[last])?;
        let current_start = start_point(&mut parts[i])?;
        let point = &parts[i];

        if last_end != current_start {
            edge.add_edge(&Edge::new_line(
                &last_end.borrow(),
                &current_start.borrow(),
            )?);
        }

        if point.get_type() == Type::Edge {
            edge.add_wire(&point.to_line().unwrap().borrow_mut());
        }
    }

    Ok(Value::Line(Rc::new(RefCell::new(edge.build()?))))
}

fn start_point(value: &mut Value) -> Result<Rc<RefCell<Point>>, RuntimeError> {
    match value.get_type() {
        Type::Point => Ok(value.to_point().unwrap()),
        Type::Edge => Ok(Rc::new(RefCell::new(
            value.to_line().unwrap().borrow_mut().start()?.unwrap(),
        ))),
        other => Err(RuntimeError::UnexpectedType(other)),
    }
}

fn end_point(value: &mut Value) -> Result<Rc<RefCell<Point>>, RuntimeError> {
    match value.get_type() {
        Type::Point => Ok(value.to_point().unwrap()),
        Type::Edge => Ok(Rc::new(RefCell::new(
            value.to_line().unwrap().borrow_mut().end()?.unwrap(),
        ))),
        other => Err(RuntimeError::UnexpectedType(other)),
    }
}

pub fn translate(
    shape: Rc<RefCell<Wire>>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let shape = shape.borrow();
    Ok(Value::Line(Rc::new(RefCell::new(Wire::translate(
        &shape,
        &Point::new(x.unwrap_or(0.0), y.unwrap_or(0.0), z.unwrap_or(0.0)),
    )?))))
}

pub fn rotate(shape: Rc<RefCell<Wire>>, angle: Option<f64>) -> Result<Value, RuntimeError> {
    let shape = shape.borrow();
    let shape = Wire::rotate(&shape, Axis::Z, angle.unwrap_or(0.0))?;

    Ok(Value::Line(Rc::new(RefCell::new(shape))))
}

pub fn rotate_3d(
    shape: Rc<RefCell<Wire>>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let shape = shape.borrow();
    let shape = Wire::rotate(&shape, Axis::X, x.unwrap_or(0.0))?;
    let shape = Wire::rotate(&shape, Axis::Y, y.unwrap_or(0.0))?;
    let shape = Wire::rotate(&shape, Axis::Z, z.unwrap_or(0.0))?;

    Ok(Value::Line(Rc::new(RefCell::new(shape))))
}

pub fn scale(shape: Rc<RefCell<Wire>>, size: f64) -> Result<Value, RuntimeError> {
    let shape = shape.borrow();
    Ok(Value::Line(Rc::new(RefCell::new(Wire::scale(
        &shape, size,
    )?))))
}
