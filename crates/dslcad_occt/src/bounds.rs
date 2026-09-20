use crate::{Error, Point};
use opencascade_sys::ffi::TopoDS_Shape;
use std::os::raw::c_void;

extern "C" {
    fn dslcad_shape_bounds(shape: *const c_void, minimum: *mut f64, maximum: *mut f64) -> bool;
}

/// Compute the axis aligned bounding box of a shape. The corners are returned
/// as `(minimum, maximum)`.
pub(crate) fn bounds(shape: &TopoDS_Shape) -> Result<(Point, Point), Error> {
    let mut minimum = [0.0; 3];
    let mut maximum = [0.0; 3];

    let ok = unsafe {
        dslcad_shape_bounds(
            shape as *const TopoDS_Shape as *const c_void,
            minimum.as_mut_ptr(),
            maximum.as_mut_ptr(),
        )
    };

    if !ok {
        return Err("could not compute bounding box".into());
    }

    Ok((
        Point::new(minimum[0], minimum[1], minimum[2]),
        Point::new(maximum[0], maximum[1], maximum[2]),
    ))
}
