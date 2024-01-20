use cxx::UniquePtr;
use opencascade_sys::ffi::{gp_Pnt, new_point};
use std::fmt::{Debug, Formatter};
use std::ops::{Add, Div, Sub};

pub struct Point {
    pub(crate) point: UniquePtr<gp_Pnt>,
}

unsafe impl Send for Point {}

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

    pub fn distance(&self, target: &Point) -> f64 {
        f64::sqrt(
            f64::powi(target.x() - self.x(), 2)
                + f64::powi(target.y() - self.y(), 2)
                + f64::powi(target.z() - self.z(), 2),
        )
    }

    pub fn length(&self) -> f64 {
        f64::sqrt(f64::powi(self.x(), 2) + f64::powi(self.y(), 2) + f64::powi(self.z(), 2))
    }

    pub fn dot(&self, target: &Point) -> f64 {
        (self.x() * target.x()) + (self.y() * target.y()) + (self.z() * target.z())
    }

    pub fn normalized(self) -> Self {
        let len = self.length();
        Self::new(self.x() / len, self.y() / len, self.z() / len)
    }
}

impl Clone for Point {
    fn clone(&self) -> Self {
        Point::new(self.x(), self.y(), self.z())
    }
}

impl Div<f64> for Point {
    type Output = Point;

    fn div(self, rhs: f64) -> Self::Output {
        Point::new(self.x() / rhs, self.y() / rhs, self.z() / rhs)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z())
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Point::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
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
