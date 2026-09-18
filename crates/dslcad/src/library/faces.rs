use crate::runtime::{RuntimeError, Value};
use dslcad_occt::{Axis, DsShape, Edge, Point, Shape, Wire, WireFactory};
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

    Ok(Value::Plane(Rc::new(edge.build()?)))
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

    Ok(Value::Plane(Rc::new(edge.build()?)))
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

pub fn union_edge(left: Value, right: Value) -> Result<Value, RuntimeError> {
    let left_wire = left.to_wire()?;
    let right_wire = right.to_wire()?;

    let mut edge = WireFactory::new();
    edge.add_wire(&left_wire);
    edge.add_wire(&right_wire);

    let result = edge.build()?;
    if left.is_plane() || right.is_plane() {
        Ok(Value::Plane(Rc::new(result)))
    } else {
        Ok(Value::Line(Rc::new(result)))
    }
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

        if let Ok(line) = point.to_wire() {
            edge.add_wire(&line);
        }
    }

    Ok(Value::Plane(Rc::new(edge.build()?)))
}

fn start_point(value: &Value) -> Result<Rc<Point>, RuntimeError> {
    if let Ok(point) = value.to_point() {
        Ok(point.clone())
    } else if let Ok(edge) = value.to_wire() {
        Ok(Rc::new(edge.start()?.unwrap()))
    } else {
        Err(RuntimeError::UnexpectedType())
    }
}

fn end_point(value: &Value) -> Result<Rc<Point>, RuntimeError> {
    if let Ok(point) = value.to_point() {
        Ok(point.clone())
    } else if let Ok(edge) = value.to_wire() {
        Ok(Rc::new(edge.end()?.unwrap()))
    } else {
        Err(RuntimeError::UnexpectedType())
    }
}

fn same_type(shape: &Value, wire: Wire) -> Value {
    if shape.is_plane() {
        Value::Plane(Rc::new(wire))
    } else {
        Value::Line(Rc::new(wire))
    }
}

pub fn translate(
    shape: Value,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let wire = shape.to_wire()?;
    let result = Wire::translate(
        &wire,
        &Point::new(x.unwrap_or(0.0), y.unwrap_or(0.0), z.unwrap_or(0.0)),
    )?;

    Ok(same_type(&shape, result))
}

pub fn rotate(shape: Value, angle: Option<f64>) -> Result<Value, RuntimeError> {
    let wire = shape.to_wire()?;
    let result = Wire::rotate(&wire, Axis::Z, angle.unwrap_or(0.0))?;

    Ok(same_type(&shape, result))
}

pub fn rotate_3d(
    shape: Value,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let wire = shape.to_wire()?;
    let result = Wire::rotate(&wire, Axis::X, x.unwrap_or(0.0))?;
    let result = Wire::rotate(&result, Axis::Y, y.unwrap_or(0.0))?;
    let result = Wire::rotate(&result, Axis::Z, z.unwrap_or(0.0))?;

    Ok(same_type(&shape, result))
}

pub fn scale(shape: Value, size: f64) -> Result<Value, RuntimeError> {
    let wire = shape.to_wire()?;
    let result = Wire::scale(&wire, size)?;

    Ok(same_type(&shape, result))
}

pub fn center(
    shape: Value,
    x: Option<bool>,
    y: Option<bool>,
    z: Option<bool>,
) -> Result<Value, RuntimeError> {
    let center = shape.to_wire()?.center_of_mass();
    let x = if x.unwrap_or(true) { -center.x() } else { 0.0 };
    let y = if y.unwrap_or(true) { -center.y() } else { 0.0 };
    let z = if z.unwrap_or(true) { -center.z() } else { 0.0 };
    translate(shape, Some(x), Some(y), Some(z))
}

pub fn offset(shape: Value, distance: f64) -> Result<Value, RuntimeError> {
    let wire = shape.to_wire()?;
    let result = wire.offset(distance)?;

    Ok(same_type(&shape, result))
}
