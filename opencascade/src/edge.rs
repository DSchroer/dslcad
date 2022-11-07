use crate::Point;
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    GC_MakeArcOfCircle, GC_MakeArcOfCircle_point_point_point, GC_MakeSegment,
    GC_MakeSegment_point_point,
};

pub enum Edge {
    Arc(UniquePtr<GC_MakeArcOfCircle>),
    Segment(UniquePtr<GC_MakeSegment>),
}

impl Edge {
    pub fn from_points(a: &Point, b: &Point) -> Self {
        // SAFETY: cross C++ boundary
        unsafe { Edge::Segment(GC_MakeSegment_point_point(&a.point, &b.point)) }
    }

    pub fn from_arc(a: &Point, b: &Point, c: &Point) -> Self {
        // SAFETY: cross C++ boundary
        unsafe {
            Edge::Arc(GC_MakeArcOfCircle_point_point_point(
                &a.point, &b.point, &c.point,
            ))
        }
    }
}
