use crate::runtime::{RuntimeError, Value};

use dslcad_occt::{Axis, DsShape, Point, Shape, Wire};

pub fn cube(x: Option<f64>, y: Option<f64>, z: Option<f64>) -> Result<Value, RuntimeError> {
    let x = x.unwrap_or(1.0);
    let y = y.unwrap_or(1.0);
    let z = z.unwrap_or(1.0);

    let cube = Shape::cube(x, y, z)?;
    Ok(cube.into())
}

pub fn sphere(radius: Option<f64>) -> Result<Value, RuntimeError> {
    let r = radius.unwrap_or(0.5);

    let base = Shape::sphere(r)?;
    let aligned = base.translate(&Point::new(r, r, r))?;
    Ok(aligned.into())
}

pub fn cylinder(radius: Option<f64>, height: Option<f64>) -> Result<Value, RuntimeError> {
    let radius = radius.unwrap_or(0.5);
    let height = height.unwrap_or(1.0);

    let base = Shape::cylinder(radius, height)?;
    Ok(base.into())
}

pub fn union_shape(left: &Shape, right: &Shape) -> Result<Value, RuntimeError> {
    Ok(Shape::fuse(left, right)?.into())
}

pub fn difference(left: &Shape, right: &Shape) -> Result<Value, RuntimeError> {
    Ok(Shape::cut(left, right)?.into())
}

pub fn intersect(left: &Shape, right: &Shape) -> Result<Value, RuntimeError> {
    Ok(Shape::intersect(left, right)?.into())
}

pub fn chamfer(shape: &Shape, radius: f64) -> Result<Value, RuntimeError> {
    Ok(Shape::chamfer(shape, radius)?.into())
}

pub fn fillet(shape: &Shape, radius: f64) -> Result<Value, RuntimeError> {
    Ok(Shape::fillet(shape, radius)?.into())
}

pub fn translate(
    shape: &Shape,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    Ok(Shape::translate(
        shape,
        &Point::new(x.unwrap_or(0.0), y.unwrap_or(0.0), z.unwrap_or(0.0)),
    )?
    .into())
}

pub fn rotate(
    shape: &Shape,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let shape = Shape::rotate(shape, Axis::X, x.unwrap_or(0.0))?;
    let shape = Shape::rotate(&shape, Axis::Y, y.unwrap_or(0.0))?;
    let shape = Shape::rotate(&shape, Axis::Z, z.unwrap_or(0.0))?;

    Ok(shape.into())
}

pub fn scale(shape: &Shape, size: f64) -> Result<Value, RuntimeError> {
    Ok(Shape::scale(shape, size)?.into())
}

pub fn scale_xyz(
    shape: &Shape,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let transform_matrix = [
        x.unwrap_or(1.0),
        0.0,
        0.0,
        0.0,
        0.0,
        y.unwrap_or(1.0),
        0.0,
        0.0,
        0.0,
        0.0,
        z.unwrap_or(1.0),
        0.0,
    ];

    Ok(Shape::transform(shape, &transform_matrix)?.into())
}

pub fn center(
    shape: &Shape,
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

pub fn slice_2d(left: &Shape, right: &Wire) -> Result<Value, RuntimeError> {
    Ok(left.section_2d(right)?.into())
}

pub fn slice(left: &Shape, right: &Shape) -> Result<Value, RuntimeError> {
    Ok(left.section(right)?.into())
}
