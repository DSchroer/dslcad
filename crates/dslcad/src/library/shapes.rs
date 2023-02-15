use crate::runtime::{RuntimeError, Value};

use std::cell::RefCell;
use std::rc::Rc;

use opencascade::{Axis, DsShape, Point, Shape};

pub fn cube(
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    center: bool,
) -> Result<Value, RuntimeError> {
    let x = x.unwrap_or(1.0);
    let y = y.unwrap_or(1.0);
    let z = z.unwrap_or(1.0);

    let cube = Shape::cube(x, y, z)?;
    let aligned = if center {
        cube.translate(&Point::new(-x / 2., -y / 2., -z / 2.))?
    } else {
        cube
    };

    Ok(aligned.into())
}

pub fn sphere(radius: Option<f64>, center: bool) -> Result<Value, RuntimeError> {
    let r = radius.unwrap_or(0.5);

    let base = Shape::sphere(r)?;
    let aligned = if center {
        base
    } else {
        base.translate(&Point::new(r, r, r))?
    };

    Ok(aligned.into())
}

pub fn cylinder(
    radius: Option<f64>,
    height: Option<f64>,
    center: bool,
) -> Result<Value, RuntimeError> {
    let radius = radius.unwrap_or(0.5);
    let height = height.unwrap_or(1.0);

    let base = Shape::cylinder(radius, height)?;
    let aligned = if center {
        base.translate(&Point::new(-radius, -radius, -height / 2.))?
    } else {
        base
    };

    Ok(aligned.into())
}

pub fn union_shape(
    left: Rc<RefCell<Shape>>,
    right: Rc<RefCell<Shape>>,
) -> Result<Value, RuntimeError> {
    let left = left.borrow();
    let right = right.borrow();

    Ok(Shape::fuse(&left, &right)?.into())
}

pub fn difference(
    left: Rc<RefCell<Shape>>,
    right: Rc<RefCell<Shape>>,
) -> Result<Value, RuntimeError> {
    let left = left.borrow();
    let right = right.borrow();

    Ok(Shape::cut(&left, &right)?.into())
}

pub fn intersect(
    left: Rc<RefCell<Shape>>,
    right: Rc<RefCell<Shape>>,
) -> Result<Value, RuntimeError> {
    let left = left.borrow();
    let right = right.borrow();

    Ok(Shape::intersect(&left, &right)?.into())
}

pub fn chamfer(shape: Rc<RefCell<Shape>>, radius: f64) -> Result<Value, RuntimeError> {
    let shape = shape.borrow();
    Ok(Shape::chamfer(&shape, radius)?.into())
}

pub fn fillet(shape: Rc<RefCell<Shape>>, radius: f64) -> Result<Value, RuntimeError> {
    let shape = shape.borrow();
    Ok(Shape::fillet(&shape, radius)?.into())
}

pub fn translate(
    shape: Rc<RefCell<Shape>>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let shape = shape.borrow();
    Ok(Shape::translate(
        &shape,
        &Point::new(x.unwrap_or(0.0), y.unwrap_or(0.0), z.unwrap_or(0.0)),
    )?
    .into())
}

pub fn rotate(
    shape: Rc<RefCell<Shape>>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let shape = shape.borrow();
    let shape = Shape::rotate(&shape, Axis::X, x.unwrap_or(0.0))?;
    let shape = Shape::rotate(&shape, Axis::Y, y.unwrap_or(0.0))?;
    let shape = Shape::rotate(&shape, Axis::Z, z.unwrap_or(0.0))?;

    Ok(shape.into())
}

pub fn scale(shape: Rc<RefCell<Shape>>, size: f64) -> Result<Value, RuntimeError> {
    let shape = shape.borrow();
    Ok(Shape::scale(&shape, size)?.into())
}
