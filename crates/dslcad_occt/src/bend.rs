use crate::{Axis, Error, Shape};
use cxx::UniquePtr;
use opencascade_sys::ffi::TopoDS_Shape;
use std::os::raw::c_void;

extern "C" {
    fn dslcad_bend_shape(shape: *const c_void, axis: i32, degrees: f64) -> *mut c_void;
}

impl Shape {
    /// Bend a shape around an axis. The shape's larger extent perpendicular to
    /// the axis is wrapped into a circular arc of `degrees`, the remaining
    /// perpendicular axis is the direction the shape bends towards.
    pub fn bend(shape: &Shape, axis: Axis, degrees: f64) -> Result<Self, Error> {
        if degrees == 0.0 {
            return Ok(Shape::from(shape.as_ref()));
        }

        let raw = unsafe {
            dslcad_bend_shape(
                shape.as_ref() as *const TopoDS_Shape as *const c_void,
                axis as i32,
                degrees,
            )
        };

        if raw.is_null() {
            return Err("could not bend shape".into());
        }

        Ok(Shape {
            shape: unsafe { UniquePtr::from_raw(raw as *mut TopoDS_Shape) },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_bends_a_cube() {
        let cube = Shape::cube(10., 1., 1.).unwrap();
        let bent = Shape::bend(&cube, Axis::Y, 90.).unwrap();

        bent.mesh(0.1).unwrap();

        let volume = bent.volume();
        assert!((volume - 10.0).abs() < 0.1, "unexpected volume {volume}");
    }

    #[test]
    fn it_bends_in_both_directions() {
        let cube = Shape::cube(10., 1., 1.).unwrap();

        let up = Shape::bend(&cube, Axis::Y, 90.).unwrap();
        let down = Shape::bend(&cube, Axis::Y, -90.).unwrap();

        assert!(up.center_of_mass().z() > 0.5);
        assert!(down.center_of_mass().z() < 0.5);
    }

    #[test]
    fn it_keeps_the_original_shape_untouched() {
        let cube = Shape::cube(10., 1., 1.).unwrap();
        Shape::bend(&cube, Axis::Y, 90.).unwrap();

        assert!((cube.volume() - 10.0).abs() < 1e-9);
    }
}
