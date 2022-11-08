use crate::runtime::RuntimeError;
use crate::syntax::{Instance, Value};

use std::cell::RefCell;
use std::collections::HashMap;

use std::io::Error;

use std::rc::Rc;

use opencascade::{Axis, Point, Shape};

use super::*;

pub fn cube(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let width = number!(args, "width");
    let height = number!(args, "height");
    let length = number!(args, "length");

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::cube(
        length, width, height,
    )))))
}

pub fn cylinder(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let radius = number!(args, "radius");
    let height = number!(args, "height");

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::cylinder(
        radius, height,
    )))))
}

pub fn union(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut left = shape!(args, "left").borrow_mut();
    let mut right = shape!(args, "right").borrow_mut();

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::fuse(
        &mut left, &mut right,
    )))))
}

pub fn difference(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut left = shape!(args, "left").borrow_mut();
    let mut right = shape!(args, "right").borrow_mut();

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::cut(
        &mut left, &mut right,
    )))))
}

pub fn chamfer(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut shape = shape!(args, "shape").borrow_mut();
    let radius = number!(args, "radius");

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::chamfer(
        &mut shape, radius,
    )))))
}

pub fn fillet(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut shape = shape!(args, "shape").borrow_mut();
    let radius = number!(args, "radius");

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::fillet(
        &mut shape, radius,
    )))))
}

pub fn translate(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut shape = shape!(args, "shape").borrow_mut();
    let x = number!(args, "x", 0.);
    let y = number!(args, "y", 0.);
    let z = number!(args, "z", 0.);

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::translate(
        &mut shape,
        &Point::new(x, y, z),
    )))))
}

pub fn rotate(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut shape = shape!(args, "shape").borrow_mut();
    let x = number!(args, "x", 0.);
    let y = number!(args, "y", 0.);
    let z = number!(args, "z", 0.);

    let mut shape = Shape::rotate(&mut shape, Axis::X, x);
    let mut shape = Shape::rotate(&mut shape, Axis::Y, y);
    let shape = Shape::rotate(&mut shape, Axis::Z, z);

    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}

pub fn scale(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut shape = shape!(args, "shape").borrow_mut();
    let size = number!(args, "size", 0.);

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::scale(
        &mut shape, size,
    )))))
}

impl Instance for Shape {
    fn get(&self, _: &str) -> Option<&Box<Value>> {
        None
    }

    fn value(&self) -> &Value {
        todo!("Implement value")
    }

    fn write_to_file(&mut self, path: &str) -> Result<(), Error> {
        self.write_stl(path)
    }
}
