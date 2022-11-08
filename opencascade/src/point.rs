use cxx::UniquePtr;
use opencascade_sys::ffi::{gp_Pnt, new_point};
use std::fmt::{Debug, Formatter};

pub struct Point {
    pub(crate) point: UniquePtr<gp_Pnt>,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        // SAFETY: cross C++ boundary
        unsafe {
            Point {
                point: new_point(x, y, z),
            }
        }
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
