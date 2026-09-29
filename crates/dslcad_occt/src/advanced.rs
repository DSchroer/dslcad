use crate::command::Builder;
use crate::{Axis, Error, Point, Shape, Wire};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    BRepBuilderAPI_MakeFace_wire, TopAbs_ShapeEnum, TopExp_Explorer_ctor, TopoDS_Shape,
};
use std::os::raw::c_void;

extern "C" {
    fn dslcad_distance(left: *const c_void, right: *const c_void, distance: *mut f64) -> bool;
    fn dslcad_contains(shape: *const c_void, x: f64, y: f64, z: f64) -> i32;
    fn dslcad_split(shape: *const c_void, tool: *const c_void) -> *mut c_void;
    fn dslcad_defeature(shape: *const c_void, radius: f64) -> *mut c_void;
    fn dslcad_hole(
        shape: *const c_void,
        x: f64,
        y: f64,
        z: f64,
        dx: f64,
        dy: f64,
        dz: f64,
        radius: f64,
        depth: f64,
    ) -> *mut c_void;
}

fn shape_ptr(shape: &Shape) -> *const c_void {
    shape.as_ref() as *const TopoDS_Shape as *const c_void
}

impl Shape {
    /// Minimum distance between two shapes.
    pub fn distance(left: &Shape, right: &Shape) -> Result<f64, Error> {
        let mut distance = 0.0;

        let ok = unsafe { dslcad_distance(shape_ptr(left), shape_ptr(right), &mut distance) };
        if !ok {
            return Err("could not measure the distance".into());
        }

        Ok(distance)
    }

    /// Whether a point is inside the shape.
    pub fn contains(&self, point: &Point) -> bool {
        unsafe { dslcad_contains(shape_ptr(self), point.x(), point.y(), point.z()) != 0 }
    }

    /// Split the shape with a planar tool, returning the resulting solids.
    pub fn split(&self, plane: &Wire) -> Result<Vec<Self>, Error> {
        if plane.is_compound() {
            return Err("split tools must be a single contour".into());
        }

        let mut face_builder = BRepBuilderAPI_MakeFace_wire(plane.wire(), false);
        let face = Builder::try_build(&mut face_builder)?;

        let raw = unsafe {
            dslcad_split(
                shape_ptr(self),
                face as *const TopoDS_Shape as *const c_void,
            )
        };
        if raw.is_null() {
            return Err("could not split the shape".into());
        }

        let result = Shape {
            shape: unsafe { UniquePtr::from_raw(raw as *mut TopoDS_Shape) },
        };

        let mut pieces = Vec::new();
        let mut explorer = TopExp_Explorer_ctor(&result.shape, TopAbs_ShapeEnum::TopAbs_SOLID);
        while explorer.More() {
            pieces.push(Shape::from(explorer.Current()));
            explorer.pin_mut().Next();
        }

        if pieces.is_empty() {
            return Err("splitting produced no solids".into());
        }

        Ok(pieces)
    }

    /// Remove every cylindrical face with the given radius, for example to get
    /// rid of drilled holes.
    pub fn defeature(&self, radius: f64) -> Result<Self, Error> {
        let raw = unsafe { dslcad_defeature(shape_ptr(self), radius) };
        if raw.is_null() {
            return Err("could not remove the features".into());
        }

        Ok(Shape {
            shape: unsafe { UniquePtr::from_raw(raw as *mut TopoDS_Shape) },
        })
    }

    /// Drill a cylindrical hole along an axis at a point. A missing depth
    /// drills through the whole shape.
    pub fn hole(
        &self,
        at: &Point,
        axis: Axis,
        radius: f64,
        depth: Option<f64>,
    ) -> Result<Self, Error> {
        let (dx, dy, dz) = match axis {
            Axis::X => (1., 0., 0.),
            Axis::Y => (0., 1., 0.),
            Axis::Z => (0., 0., 1.),
        };

        let raw = unsafe {
            dslcad_hole(
                shape_ptr(self),
                at.x(),
                at.y(),
                at.z(),
                dx,
                dy,
                dz,
                radius,
                depth.unwrap_or(-1.0),
            )
        };
        if raw.is_null() {
            return Err("could not drill the hole".into());
        }

        Ok(Shape {
            shape: unsafe { UniquePtr::from_raw(raw as *mut TopoDS_Shape) },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::DsShape;
    use crate::{Edge, WireFactory};
    use std::f64::consts::PI;

    fn square(x: f64, y: f64, size: f64) -> Wire {
        let mut wire = WireFactory::new();

        wire.add_edge(
            &Edge::new_line(&Point::new(x, y, 0.), &Point::new(x + size, y, 0.)).unwrap(),
        );
        wire.add_edge(
            &Edge::new_line(
                &Point::new(x + size, y, 0.),
                &Point::new(x + size, y + size, 0.),
            )
            .unwrap(),
        );
        wire.add_edge(
            &Edge::new_line(
                &Point::new(x + size, y + size, 0.),
                &Point::new(x, y + size, 0.),
            )
            .unwrap(),
        );
        wire.add_edge(
            &Edge::new_line(&Point::new(x, y + size, 0.), &Point::new(x, y, 0.)).unwrap(),
        );

        wire.build().unwrap()
    }

    #[test]
    fn it_measures_the_distance_between_shapes() {
        let left = Shape::cube(10., 10., 10.).unwrap();
        let right = Shape::cube(10., 10., 10.)
            .unwrap()
            .translate(&Point::new(25., 0., 0.))
            .unwrap();

        assert!((Shape::distance(&left, &right).unwrap() - 15.).abs() < 1e-6);
    }

    #[test]
    fn it_tests_whether_a_point_is_inside() {
        let cube = Shape::cube(10., 10., 10.).unwrap();

        assert!(cube.contains(&Point::new(5., 5., 5.)));
        assert!(!cube.contains(&Point::new(15., 5., 5.)));
    }

    #[test]
    fn it_splits_a_shape_with_a_plane() {
        let cube = Shape::cube(10., 10., 10.).unwrap();
        let plane = square(-5., -5., 20.)
            .translate(&Point::new(0., 0., 5.))
            .unwrap();

        let pieces = cube.split(&plane).unwrap();
        assert_eq!(pieces.len(), 2);
        for piece in pieces {
            assert!((piece.volume() - 500.).abs() < 0.1);
        }
    }

    #[test]
    fn it_can_drill_and_remove_a_hole() {
        let cube = Shape::cube(10., 10., 10.).unwrap();

        let drilled = cube
            .hole(&Point::new(5., 5., 0.), Axis::Z, 2., None)
            .unwrap();
        assert!((drilled.volume() - (1000. - PI * 4. * 10.)).abs() < 0.1);

        let defeatured = drilled.defeature(2.).unwrap();
        assert!((defeatured.volume() - 1000.).abs() < 0.1);
    }
}
