use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Error;
use std::ops::Deref;
use std::rc::Rc;
use crate::runtime::RuntimeError;
use crate::syntax::{Instance, Value};

use opencascade::{Shape, Point};

use super::{*};

pub fn cube(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let width = number!(args, "width");
    let height = number!(args, "height");
    let length = number!(args, "length");

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::cube(length, width, height)))))
}

pub fn cylinder(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let radius = number!(args, "radius");
    let height = number!(args, "height");

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::cylinder(radius, height)))))
}

pub fn union(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut left = shape!(args, "left").borrow_mut();
    let mut right = shape!(args, "right").borrow_mut();

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::fuse(&mut left, &mut right)))))
}

pub fn difference(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut left = shape!(args, "left").borrow_mut();
    let mut right = shape!(args, "right").borrow_mut();

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::cut(&mut left, &mut right)))))
}

pub fn chamfer(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut shape = shape!(args, "shape").borrow_mut();
    let radius = number!(args, "radius");

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::chamfer(&mut shape, radius)))))
}

pub fn fillet(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut shape = shape!(args, "shape").borrow_mut();
    let radius = number!(args, "radius");

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::fillet(&mut shape, radius)))))
}

impl Instance for Shape {
    fn get(&self, identifier: &str) -> Option<&Box<Value>> {
        None
    }

    fn value(&self) -> &Box<Value> {
        todo!("Implement value")
    }

    fn write_to_file(&mut self, path: &str) -> Result<(), Error> {
        self.write_stl(path)
    }
}
