use crate::command::Builder;
use crate::{Edge, Error, Wire};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    BRepBuilderAPI_MakeFace_wire, TopoDS_Edge_to_owned, TopoDS_Shape, TopoDS_cast_to_edge,
};
use std::os::raw::c_void;

extern "C" {
    fn dslcad_fillet_2d(face: *const c_void, size: f64, chamfer: bool) -> *mut c_void;
    fn dslcad_ellipse_edge(major: f64, minor: f64) -> *mut c_void;
}

impl Edge {
    /// A full ellipse centered on the origin in the xy plane.
    pub fn new_ellipse(major: f64, minor: f64) -> Result<Self, Error> {
        if major <= 0.0 || minor <= 0.0 {
            return Err("ellipse radii must be positive".into());
        }

        let raw = unsafe { dslcad_ellipse_edge(major, minor) };
        if raw.is_null() {
            return Err("could not create ellipse".into());
        }

        let shape = crate::Shape {
            shape: unsafe { UniquePtr::from_raw(raw as *mut TopoDS_Shape) },
        };

        Ok(Edge(TopoDS_Edge_to_owned(TopoDS_cast_to_edge(
            shape.as_ref(),
        ))))
    }
}

impl Wire {
    /// Round every corner of a closed 2D shape with the given radius.
    pub fn fillet_2d(&self, radius: f64) -> Result<Self, Error> {
        self.round_corners(radius, false)
    }

    /// Chamfer every corner of a closed 2D shape with the given distance.
    pub fn chamfer_2d(&self, distance: f64) -> Result<Self, Error> {
        self.round_corners(distance, true)
    }

    /// Round or chamfer every corner of a closed 2D shape. Each contour of a
    /// compound is treated on its own so holes keep their outline.
    fn round_corners(&self, size: f64, chamfer: bool) -> Result<Self, Error> {
        if size <= 0.0 {
            return Err("corner size must be positive".into());
        }

        let contours = if self.is_compound() {
            self.contours()
        } else {
            vec![self.clone()]
        };

        let mut results = Vec::new();
        for contour in &contours {
            let mut face_builder = BRepBuilderAPI_MakeFace_wire(contour.wire(), false);
            let face = Builder::try_build(&mut face_builder)?;

            let raw = unsafe {
                dslcad_fillet_2d(face as *const TopoDS_Shape as *const c_void, size, chamfer)
            };
            if raw.is_null() {
                return Err("could not round the corners of the sketch".into());
            }

            results.push(Wire(unsafe {
                UniquePtr::from_raw(raw as *mut TopoDS_Shape)
            }));
        }

        if results.len() == 1 {
            Ok(results.remove(0))
        } else {
            Wire::compound(&results)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point;
    use crate::WireFactory;
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

    fn circle(radius: f64) -> Wire {
        let mut wire = WireFactory::new();

        let a = Point::new_2d(0., radius);
        let b = Point::new_2d(radius, radius * 2.);
        let c = Point::new_2d(radius * 2., radius);
        let d = Point::new_2d(radius, 0.);

        wire.add_edge(&Edge::new_arc(&a, &b, &c).unwrap());
        wire.add_edge(&Edge::new_arc(&c, &d, &a).unwrap());

        wire.build().unwrap()
    }

    #[test]
    fn it_rounds_2d_corners() {
        let rounded = square(0., 0., 10.).fillet_2d(1.).unwrap();
        let shape = crate::Shape::extrude(&rounded, 0., 0., 1.).unwrap();

        // Each rounded corner removes a square of side 1 except for the
        // quarter circle of radius 1 that stays.
        let expected = 100. - (4. - PI);
        assert!((shape.volume() - expected).abs() < 1e-3);
    }

    #[test]
    fn it_chamfers_2d_corners() {
        let chamfered = square(0., 0., 10.).chamfer_2d(1.).unwrap();
        let shape = crate::Shape::extrude(&chamfered, 0., 0., 1.).unwrap();

        // Each chamfered corner removes a triangle of area 1/2.
        let expected = 100. - 4. * 0.5;
        assert!((shape.volume() - expected).abs() < 1e-3);
    }

    #[test]
    fn it_keeps_tangent_corners() {
        // A circle is made of two arcs that meet tangentially, so rounding it
        // should leave it unchanged instead of failing.
        let rounded = circle(5.).fillet_2d(1.).unwrap();
        let shape = crate::Shape::extrude(&rounded, 0., 0., 1.).unwrap();

        let expected = PI * 25.;
        assert!((shape.volume() - expected).abs() < 1e-2);
    }

    #[test]
    fn it_can_create_ellipse_edges() {
        let edge = Edge::new_ellipse(4., 2.).unwrap();
        let (start, end) = edge.start_end();

        assert!((start.x() - 4.).abs() < 1e-9);
        assert!((start.y()).abs() < 1e-9);
        assert!((end.x() - start.x()).abs() < 1e-9);
        assert!((end.y() - start.y()).abs() < 1e-9);
    }
}
