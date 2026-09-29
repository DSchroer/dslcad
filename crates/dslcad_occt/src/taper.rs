use crate::{Axis, Error, Shape};
use cxx::UniquePtr;
use opencascade_sys::ffi::TopoDS_Shape;
use std::os::raw::c_void;

extern "C" {
    fn dslcad_taper_shape(
        shape: *const c_void,
        up_axis: i32,
        directions: i32,
        degrees: f64,
    ) -> *mut c_void;
}

impl Shape {
    /// Taper the walls of a shape that run along the `up` axis. Every wall
    /// facing a direction in `directions` leans inward, translating its top
    /// edge by `height * tan(degrees)` while the base of the shape stays in
    /// place, so the cross-section is offset instead of scaled.
    pub fn taper(
        shape: &Shape,
        up: Axis,
        directions: &[Axis],
        degrees: f64,
    ) -> Result<Self, Error> {
        if degrees == 0.0 || directions.is_empty() {
            return Ok(Shape::from(shape.as_ref()));
        }

        let mut mask = 0;
        for direction in directions {
            mask |= 1 << (*direction as i32);
        }

        let raw = unsafe {
            dslcad_taper_shape(
                shape.as_ref() as *const TopoDS_Shape as *const c_void,
                up as i32,
                mask,
                degrees,
            )
        };

        if raw.is_null() {
            return Err("could not taper shape".into());
        }

        Ok(Shape {
            shape: unsafe { UniquePtr::from_raw(raw as *mut TopoDS_Shape) },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DsShape;

    #[test]
    fn it_tapers_the_walls_of_a_cube() {
        let cube = Shape::cube(10., 10., 1.).unwrap();
        let tapered = Shape::taper(&cube, Axis::Z, &[Axis::Y], 45.).unwrap();

        tapered.mesh(0.1).unwrap();

        // The walls move in by 1 * tan(45) = 1 on each side, so the top is
        // (10 + 8) / 2 * 1 * 10 = 90.
        let volume = tapered.volume();
        assert!((volume - 90.0).abs() < 0.1, "unexpected volume {volume}");
    }

    #[test]
    fn it_tapers_every_given_direction() {
        let cube = Shape::cube(10., 10., 1.).unwrap();
        let tapered = Shape::taper(&cube, Axis::Z, &[Axis::X, Axis::Y], 45.).unwrap();

        // Both directions move in by z * tan(45) = z, so the cross-section at
        // height z is (10 - 2z) squared: the integral over z is 81.33.
        let volume = tapered.volume();
        assert!((volume - 81.33).abs() < 0.1, "unexpected volume {volume}");
    }

    #[test]
    fn it_keeps_the_base_of_the_shape_in_place() {
        let cube = Shape::cube(10., 10., 1.).unwrap();
        let tapered = Shape::taper(&cube, Axis::Z, &[Axis::Y], 45.).unwrap();

        let (minimum, maximum) = tapered.bounds().unwrap();
        assert!((minimum.x() - 0.0).abs() < 1e-6);
        assert!((minimum.y() - 0.0).abs() < 1e-6);
        assert!((maximum.x() - 10.0).abs() < 1e-6);
        assert!((maximum.y() - 10.0).abs() < 1e-6);
        assert!((maximum.z() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn it_only_tapers_walls_facing_a_direction() {
        let cube = Shape::cube(10., 10., 1.).unwrap();
        let tapered = Shape::taper(&cube, Axis::Z, &[Axis::X], 45.).unwrap();

        let (minimum, maximum) = tapered.bounds().unwrap();
        assert!((minimum.x() - 0.0).abs() < 1e-6);
        assert!((minimum.y() - 0.0).abs() < 1e-6);
        assert!((maximum.x() - 10.0).abs() < 1e-6);
        assert!((maximum.y() - 10.0).abs() < 1e-6);

        let volume = tapered.volume();
        assert!((volume - 90.0).abs() < 0.1, "unexpected volume {volume}");
    }

    #[test]
    fn it_gives_every_wall_the_same_angle() {
        use crate::{Edge, Point, WireFactory};

        // A sheared box whose y facing walls are not aligned with the axes, so
        // a draft would lean them along their normals and each would move by a
        // different amount along x. The ends are aligned with x and stay put.
        let corners = [
            Point::new(0., 0., 0.),
            Point::new(10., 1., 0.),
            Point::new(10., 3., 0.),
            Point::new(0., 2., 0.),
        ];

        let mut wire = WireFactory::new();
        for index in 0..4 {
            wire.add_edge(&Edge::new_line(&corners[index], &corners[(index + 1) % 4]).unwrap());
        }

        let face = wire.build().unwrap();
        let shape = Shape::extrude(&face, 0., 0., 1.).unwrap();
        let tapered = Shape::taper(&shape, Axis::Z, &[Axis::Y], 20.).unwrap();

        let amount = (20f64).to_radians().tan();

        let top: Vec<[f64; 3]> = tapered
            .points()
            .unwrap()
            .into_iter()
            .filter(|point| (point[2] - 1.0).abs() < 1e-6)
            .collect();

        // The ends are still at x 0 and 10, and each slanted wall leaned inward
        // by height * tan(angle).
        let has = |x: f64, y: f64| {
            top.iter()
                .any(|point| (point[0] - x).abs() < 1e-6 && (point[1] - y).abs() < 1e-6)
        };

        assert!(has(0., amount), "lower wall did not lean by {amount}");
        assert!(has(10., 1. + amount), "lower wall did not lean by {amount}");
        assert!(has(0., 2. - amount), "upper wall did not lean by {amount}");
        assert!(has(10., 3. - amount), "upper wall did not lean by {amount}");
    }

    #[test]
    fn it_keeps_unselected_axes_in_place() {
        // A cube rotated around z has walls that are not aligned with the
        // axes. A taper along y must not move them along x, however they are
        // oriented, and the top stays as wide as the base.
        let cube = Shape::cube(10., 10., 1.).unwrap();
        let rotated = cube.rotate(Axis::Z, 45.).unwrap();
        let tapered = Shape::taper(&rotated, Axis::Z, &[Axis::Y], 45.).unwrap();

        let (minimum, maximum) = rotated.bounds().unwrap();
        let extent = maximum.x() - minimum.x();

        let mut minimum_x = f64::MAX;
        let mut maximum_x = f64::MIN;
        for point in tapered.points().unwrap() {
            if (point[2] - 1.0).abs() < 1e-6 {
                minimum_x = minimum_x.min(point[0]);
                maximum_x = maximum_x.max(point[0]);
            }
        }

        assert!(
            ((maximum_x - minimum_x) - extent).abs() < 1e-6,
            "top x extent {} does not match the base extent {extent}",
            maximum_x - minimum_x
        );
    }

    #[test]
    fn it_keeps_the_original_shape_untouched() {
        let cube = Shape::cube(10., 10., 1.).unwrap();
        Shape::taper(&cube, Axis::Z, &[Axis::Y], 45.).unwrap();

        assert!((cube.volume() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn it_does_nothing_without_an_angle() {
        let cube = Shape::cube(10., 10., 1.).unwrap();
        let tapered = Shape::taper(&cube, Axis::Z, &[Axis::Y], 0.).unwrap();

        assert!((tapered.volume() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn it_does_nothing_without_a_direction() {
        let cube = Shape::cube(10., 10., 1.).unwrap();
        let tapered = Shape::taper(&cube, Axis::Z, &[], 45.).unwrap();

        assert!((tapered.volume() - 100.0).abs() < 1e-9);
    }
}
