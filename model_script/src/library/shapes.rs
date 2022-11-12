use crate::runtime::RuntimeError;
use crate::syntax::{Accessible, Value};

use std::cell::RefCell;
use std::collections::HashMap;

use std::rc::Rc;

use opencascade::{Axis, Edge, Point, Shape};

use super::*;

/// syntax[3D]: `cube(x=number,y=number,z=number)`  Create a 3D cube
pub fn cube(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let x = number!(args, "x");
    let y = number!(args, "y");
    let z = number!(args, "z");

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::cube(
        x, y, z,
    )))))
}

/// syntax[3D]: `cylinder(radius=number,height=number)`  Create a 3D cylinder
pub fn cylinder(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let radius = number!(args, "radius");
    let height = number!(args, "height");

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::cylinder(
        radius, height,
    )))))
}

/// syntax[2D]: `union(left=line,right=line)`  Combine two 2D lines
/// syntax[3D]: `union(left=shape,right=shape)`  Combine two 3D shapes
pub fn union(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    if let Ok(left) = shape!(args, "left") {
        let mut left = left.borrow_mut();
        let right = shape!(args, "right")?;
        let mut right = right.borrow_mut();

        return Ok(Value::Shape(Rc::new(RefCell::new(Shape::fuse(
            &mut left, &mut right,
        )))));
    }

    let left = edge!(args, "left")?;
    let right = edge!(args, "right")?;

    let mut left = left.borrow_mut();
    let mut right = right.borrow_mut();

    let mut edge = Edge::new();
    edge.join(&mut left, &mut right);
    Ok(Value::Line(Rc::new(RefCell::new(edge))))
}

/// syntax[3D]: `difference(left=shape,right=shape)`  Remove one 3D shape from another
pub fn difference(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let left = shape!(args, "left")?;
    let right = shape!(args, "right")?;

    let mut left = left.borrow_mut();
    let mut right = right.borrow_mut();

    Ok(Value::Shape(Rc::new(RefCell::new(Shape::cut(
        &mut left, &mut right,
    )))))
}

/// syntax[3D]: `chamfer(shape=shape,radius=number)`  Chamfer all edges of a 3D shape
pub fn chamfer(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let shape = shape!(args, "shape")?;
    let radius = number!(args, "radius");

    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(Shape::chamfer(
        &mut shape, radius,
    )))))
}

/// syntax[3D]: `fillet(shape=shape,radius=number)`  Fillet all edges of a 3D shape
pub fn fillet(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let shape = shape!(args, "shape")?;
    let radius = number!(args, "radius");

    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(Shape::fillet(
        &mut shape, radius,
    )))))
}

/// syntax[3D]: `translate(shape=shape,x=number,y=number,z=number)`  Translate a 3D shape
pub fn translate(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let shape = shape!(args, "shape")?;
    let x = number!(args, "x", 0.);
    let y = number!(args, "y", 0.);
    let z = number!(args, "z", 0.);

    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(Shape::translate(
        &mut shape,
        &Point::new(x, y, z),
    )))))
}

/// syntax[3D]: `rotate(shape=shape,x=number,y=number,z=number)`  Rotate a 3D shape
pub fn rotate(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let shape = shape!(args, "shape")?;
    let x = number!(args, "x", 0.);
    let y = number!(args, "y", 0.);
    let z = number!(args, "z", 0.);

    let mut shape = shape.borrow_mut();
    let mut shape = Shape::rotate(&mut shape, Axis::X, x);
    let mut shape = Shape::rotate(&mut shape, Axis::Y, y);
    let shape = Shape::rotate(&mut shape, Axis::Z, z);

    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}

/// syntax[3D]: `scale(shape=shape,size=number)` Scale a 3D shape
pub fn scale(args: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let shape = shape!(args, "shape")?;
    let size = number!(args, "size", 0.);

    let mut shape = shape.borrow_mut();
    Ok(Value::Shape(Rc::new(RefCell::new(Shape::scale(
        &mut shape, size,
    )))))
}

impl Accessible for Shape {
    fn get(&self, _: &str) -> Option<&Value> {
        None
    }
}
