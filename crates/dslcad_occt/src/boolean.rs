use crate::{Error, Shape};
use cxx::UniquePtr;
use opencascade_sys::ffi::TopoDS_Shape;
use std::os::raw::c_void;

extern "C" {
    fn dslcad_boolean(
        left: *const c_void,
        right: *const c_void,
        operation: i32,
        glue: i32,
        fuzzy: f64,
    ) -> *mut c_void;
}

#[derive(Clone, Copy)]
pub enum BooleanOperation {
    Fuse = 0,
    Cut = 1,
    Common = 2,
}

#[derive(Clone, Copy)]
pub enum BooleanGlue {
    Off = 0,
    Shift = 1,
    Full = 2,
}

impl Shape {
    /// Run a boolean operation with optional glue and fuzzy options.
    pub fn boolean(
        left: &Shape,
        right: &Shape,
        operation: BooleanOperation,
        glue: BooleanGlue,
        fuzzy: Option<f64>,
    ) -> Result<Self, Error> {
        let raw = unsafe {
            dslcad_boolean(
                left.as_ref() as *const TopoDS_Shape as *const c_void,
                right.as_ref() as *const TopoDS_Shape as *const c_void,
                operation as i32,
                glue as i32,
                fuzzy.unwrap_or(0.0),
            )
        };

        if raw.is_null() {
            return Err("boolean operation failed".into());
        }

        Ok(Shape {
            shape: unsafe { UniquePtr::from_raw(raw as *mut TopoDS_Shape) },
        })
    }
}
