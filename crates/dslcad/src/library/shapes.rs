use super::from_cascade;
use crate::runtime::{RuntimeError, Value};

use std::cell::RefCell;
use std::rc::Rc;

use opencascade::{Axis, Point, Shape};

pub fn cube(x: Option<f64>, y: Option<f64>, z: Option<f64>) -> Result<Value, RuntimeError> {
    Ok(Value::Shape(Rc::new(RefCell::new(from_cascade!(
        Shape::cube(x.unwrap_or(1.0), y.unwrap_or(1.0), z.unwrap_or(1.0),)
    )?))))
}

pub fn sphere(radius: Option<f64>) -> Result<Value, RuntimeError> {
    let r = radius.unwrap_or(0.5);
    let mut base = from_cascade!(Shape::sphere(r))?;
    let moved = from_cascade!(Shape::translate(&mut base, &Point::new(r, r, r)))?;
    Ok(Value::Shape(Rc::new(RefCell::new(moved))))
}

pub fn cylinder(radius: Option<f64>, height: Option<f64>) -> Result<Value, RuntimeError> {
    Ok(Value::Shape(Rc::new(RefCell::new(from_cascade!(
        Shape::cylinder(radius.unwrap_or(0.5), height.unwrap_or(1.0),)
    )?))))
}

pub fn union_shape(
    left: Rc<RefCell<Shape>>,
    right: Rc<RefCell<Shape>>,
) -> Result<Value, RuntimeError> {
    let mut left = left.borrow_mut();
    let mut right = right.borrow_mut();

    Ok(Value::Shape(Rc::new(RefCell::new(from_cascade!(
        Shape::fuse(&mut left, &mut right,)
    )?))))
}

pub fn difference(
    left: Rc<RefCell<Shape>>,
    right: Rc<RefCell<Shape>>,
) -> Result<Value, RuntimeError> {
    let mut left = left.borrow_mut();
    let mut right = right.borrow_mut();

    Ok(Value::Shape(Rc::new(RefCell::new(from_cascade!(
        Shape::cut(&mut left, &mut right,)
    )?))))
}

pub fn intersect(
    left: Rc<RefCell<Shape>>,
    right: Rc<RefCell<Shape>>,
) -> Result<Value, RuntimeError> {
    let mut left = left.borrow_mut();
    let mut right = right.borrow_mut();

    Ok(Value::Shape(Rc::new(RefCell::new(from_cascade!(
        Shape::intersect(&mut left, &mut right,)
    )?))))
}

pub fn chamfer(shape: Rc<RefCell<Shape>>, radius: f64) -> Result<Value, RuntimeError> {
    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(from_cascade!(
        Shape::chamfer(&mut shape, radius,)
    )?))))
}

pub fn fillet(shape: Rc<RefCell<Shape>>, radius: f64) -> Result<Value, RuntimeError> {
    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(from_cascade!(
        Shape::fillet(&mut shape, radius,)
    )?))))
}

pub fn translate(
    shape: Rc<RefCell<Shape>>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(from_cascade!(
        Shape::translate(
            &mut shape,
            &Point::new(x.unwrap_or(0.0), y.unwrap_or(0.0), z.unwrap_or(0.0)),
        )
    )?))))
}

pub fn rotate(
    shape: Rc<RefCell<Shape>>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<Value, RuntimeError> {
    let mut shape = shape.borrow_mut();
    let mut shape = from_cascade!(Shape::rotate(&mut shape, Axis::X, x.unwrap_or(0.0)))?;
    let mut shape = from_cascade!(Shape::rotate(&mut shape, Axis::Y, y.unwrap_or(0.0)))?;
    let shape = from_cascade!(Shape::rotate(&mut shape, Axis::Z, z.unwrap_or(0.0)))?;

    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}

pub fn scale(shape: Rc<RefCell<Shape>>, size: f64) -> Result<Value, RuntimeError> {
    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(from_cascade!(
        Shape::scale(&mut shape, size,)
    )?))))
}
