use cxx::UniquePtr;
use opencascade_sys::ffi::{gp_Pnt, new_point};
use std::fmt::{Debug, Formatter};

pub struct Point {
    pub(crate) point: UniquePtr<gp_Pnt>,
}

impl Point {
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

impl From<UniquePtr<gp_Pnt>> for Point {
    fn from(point: UniquePtr<gp_Pnt>) -> Self {
        Point { point }
    }
}

impl Into<[f64;3]> for Point {
    fn into(self) -> [f64; 3] {
        [self.x(), self.y(), self.z()]
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
