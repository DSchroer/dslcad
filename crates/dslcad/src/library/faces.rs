use crate::runtime::{RuntimeError, Value};
use opencascade::{Axis, DsShape, Edge, Point, Shape, Wire, WireFactory};
use std::rc::Rc;

pub fn point(x: Option<f64>, y: Option<f64>, z: Option<f64>) -> Result<Value, RuntimeError> {
    Ok(Value::Point(Rc::new(Point::new(
        x.unwrap_or(0.0),
        y.unwrap_or(0.0),
        z.unwrap_or(0.0),
    ))))
}

pub fn line(start: &Point, end: &Point) -> Result<Value, RuntimeError> {
    let mut edge = WireFactory::new();
    edge.add_edge(&Edge::new_line(start, end)?);
    Ok(Value::Line(Rc::new(edge.build()?)))
}

pub fn arc(start: &Point, center: &Point, end: &Point) -> Result<Value, RuntimeError> {
    if start == center || center == end {
        return Err(RuntimeError::ArcWithIdenticalPoints());
    }

    let mut edge = WireFactory::new();
    edge.add_edge(&Edge::new_arc(start, center, end)?);
    Ok(Value::Line(Rc::new(edge.build()?)))
}

pub fn square(x: Option<f64>, y: Option<f64>) -> Result<Value, RuntimeError> {
    let mut edge = WireFactory::new();

    let x0 = 0.0;
    let y0 = 0.0;

    let a = Point::new_2d(x0, y0);
    let b = Point::new_2d(x0, y0 + y.unwrap_or(1.0));
    let c = Point::new_2d(x0 + x.unwrap_or(1.0), y0 + y.unwrap_or(1.0));
    let d = Point::new_2d(x0 + x.unwrap_or(1.0), y0);

    edge.add_edge(&Edge::new_line(&a, &b)?);
    edge.add_edge(&Edge::new_line(&b, &c)?);
    edge.add_edge(&Edge::new_line(&c, &d)?);
    edge.add_edge(&Edge::new_line(&d, &a)?);

    Ok(Value::Line(Rc::new(edge.build()?)))
}

pub fn circle(radius: Option<f64>) -> Result<Value, RuntimeError> {
    let mut edge = WireFactory::new();

    let r = radius.unwrap_or(0.5);

    let x0 = 0.0;
    let y0 = 0.0;

    let a = Point::new_2d(x0, y0 + r);
    let b = Point::new_2d(x0 + r, y0 + (r * 2.0));
    let c = Point::new_2d(x0 + (r * 2.0), y0 + r);
    let d = Point::new_2d(x0 + r, y0);

    edge.add_edge(&Edge::new_arc(&a, &b, &c)?);
    edge.add_edge(&Edge::new_arc(&c, &d, &a)?);

    Ok(Value::Line(Rc::new(edge.build()?)))
}

pub fn extrude(
    shape: &Wire,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    Ok(Value::Shape(Rc::new(Shape::extrude(
        shape,
        x.unwrap_or(0.0),
        y.unwrap_or(0.0),
        z.unwrap_or(0.0),
    )?)))
}

pub fn revolve(
    shape: &Wire,
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

    Ok(Value::Shape(Rc::new(Shape::extrude_rotate(
        shape, axis, angle,
    )?)))
}

pub fn union_edge(left: &Wire, right: &Wire) -> Result<Value, RuntimeError> {
    let mut edge = WireFactory::new();
    edge.add_wire(left);
    edge.add_wire(right);
    Ok(Value::Line(Rc::new(edge.build()?)))
}

pub fn face(parts: &[Value]) -> Result<Value, RuntimeError> {
    if parts.is_empty() {
        return point(None, None, None);
    }
    let parts_len = parts.len();

    let start = start_point(&parts[0])?;
    let end = end_point(&parts[parts_len - 1])?;

    if parts_len == 1 && start == end {
        return Ok(parts[0].clone());
    }

    let mut edge = WireFactory::new();
    for i in 0..parts_len {
        let last = if i == 0 { parts_len - 1 } else { i - 1 };
        let last_end = end_point(&parts[last])?;
        let current_start = start_point(&parts[i])?;
        let point = &parts[i];

        if last_end != current_start {
            edge.add_edge(&Edge::new_line(&last_end, &current_start)?);
        }

        if let Ok(line) = point.to_line() {
            edge.add_wire(&line);
        }
    }

    Ok(Value::Line(Rc::new(edge.build()?)))
}

fn start_point(value: &Value) -> Result<Rc<Point>, RuntimeError> {
    if let Ok(point) = value.to_point() {
        Ok(point.clone())
    } else if let Ok(edge) = value.to_line() {
        Ok(Rc::new(edge.start()?.unwrap()))
    } else {
        Err(RuntimeError::UnexpectedType())
    }
}

fn end_point(value: &Value) -> Result<Rc<Point>, RuntimeError> {
    if let Ok(point) = value.to_point() {
        Ok(point.clone())
    } else if let Ok(edge) = value.to_line() {
        Ok(Rc::new(edge.end()?.unwrap()))
    } else {
        Err(RuntimeError::UnexpectedType())
    }
}

pub fn translate(
    shape: &Wire,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    Ok(Value::Line(Rc::new(Wire::translate(
        shape,
        &Point::new(x.unwrap_or(0.0), y.unwrap_or(0.0), z.unwrap_or(0.0)),
    )?)))
}

pub fn rotate(shape: &Wire, angle: Option<f64>) -> Result<Value, RuntimeError> {
    let shape = Wire::rotate(shape, Axis::Z, angle.unwrap_or(0.0))?;

    Ok(Value::Line(Rc::new(shape)))
}

pub fn rotate_3d(
    shape: &Wire,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let shape = Wire::rotate(shape, Axis::X, x.unwrap_or(0.0))?;
    let shape = Wire::rotate(&shape, Axis::Y, y.unwrap_or(0.0))?;
    let shape = Wire::rotate(&shape, Axis::Z, z.unwrap_or(0.0))?;

    Ok(Value::Line(Rc::new(shape)))
}

pub fn scale(shape: &Wire, size: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Line(Rc::new(Wire::scale(shape, size)?)))
}

pub fn center(
    shape: &Wire,
    x: Option<bool>,
    y: Option<bool>,
    z: Option<bool>,
) -> Result<Value, RuntimeError> {
    let center = shape.center_of_mass();
    let x = if x.unwrap_or(true) { -center.x() } else { 0.0 };
    let y = if y.unwrap_or(true) { -center.y() } else { 0.0 };
    let z = if z.unwrap_or(true) { -center.z() } else { 0.0 };
    translate(shape, Some(x), Some(y), Some(z))
}

pub fn offset(shape: &Wire, distance: f64) -> Result<Value, RuntimeError> {
    Ok(shape.offset(distance)?.into())
}
