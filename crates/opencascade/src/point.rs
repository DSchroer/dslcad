use cxx::UniquePtr;
use opencascade_sys::ffi::{gp_Pnt, new_point};
use std::fmt::{Debug, Formatter};

pub struct Point {
    pub(crate) point: UniquePtr<gp_Pnt>,
}

impl Point {
    pub fn new_2d(x: f64, y: f64) -> Self {
        Point::new(x, y, 0.0)
    }

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Point {
            point: new_point(x, y, z),
        }
    }

    pub fn x(&self) -> f64 {
        self.point.X()
    }

    pub fn y(&self) -> f64 {
        self.point.Y()
    }

    pub fn z(&self) -> f64 {
        self.point.Z()
    }
}

impl AsRef<gp_Pnt> for Point {
    fn as_ref(&self) -> &gp_Pnt {
        &self.point
    }
}

impl From<UniquePtr<gp_Pnt>> for Point {
    fn from(point: UniquePtr<gp_Pnt>) -> Self {
        Point { point }
    }
}

impl From<Point> for [f64; 3] {
    fn from(value: Point) -> Self {
        [value.x(), value.y(), value.z()]
    }
}

impl Default for Point {
    fn default() -> Self {
        Point::new(0., 0., 0.)
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.point.X())
            .field("y", &self.point.Y())
            .field("z", &self.point.Z())
            .finish()
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.point.X() == other.point.X()
            && self.point.Y() == other.point.Y()
            && self.point.Z() == other.point.Z()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_create() {
        let point = Point::new(0., 0., 0.);
        dbg!(point);
    }
}
