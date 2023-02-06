use crate::command::{PinBuilder, PinCommand};
use crate::{Error, Point};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    new_HandleGeomCurve_from_HandleGeom_TrimmedCurve, BRepBuilderAPI_MakeEdge,
    BRepBuilderAPI_MakeEdge_HandleGeomCurve, GC_MakeArcOfCircle_Value,
    GC_MakeArcOfCircle_point_point_point, GC_MakeSegment_Value, GC_MakeSegment_point_point,
    TopoDS_Edge, TopoDS_Edge_to_owned,
};
use std::pin::Pin;

pub struct Edge(pub(crate) UniquePtr<TopoDS_Edge>);

impl Edge {
    pub fn new_line(a: &Point, b: &Point) -> Result<Self, Error> {
        let segment = GC_MakeSegment_point_point(&a.point, &b.point);
        let mut edge_1 = BRepBuilderAPI_MakeEdge_HandleGeomCurve(
            &new_HandleGeomCurve_from_HandleGeom_TrimmedCurve(&GC_MakeSegment_Value(&segment)),
        );
        Ok(Edge(TopoDS_Edge_to_owned(PinBuilder::try_build(
            &mut edge_1,
        )?)))
    }

    pub fn new_arc(a: &Point, b: &Point, c: &Point) -> Result<Self, Error> {
        let segment = GC_MakeArcOfCircle_point_point_point(&a.point, &b.point, &c.point);
        let mut edge_1 = BRepBuilderAPI_MakeEdge_HandleGeomCurve(
            &new_HandleGeomCurve_from_HandleGeom_TrimmedCurve(&GC_MakeArcOfCircle_Value(&segment)),
        );
        Ok(Edge(TopoDS_Edge_to_owned(PinBuilder::try_build(
            &mut edge_1,
        )?)))
    }
}

impl PinCommand for BRepBuilderAPI_MakeEdge {
    fn is_done(&self) -> bool {
        self.IsDone()
    }

    fn build(self: Pin<&mut Self>, progress: &opencascade_sys::ffi::Message_ProgressRange) {
        self.Build(progress)
    }
}

impl PinBuilder<TopoDS_Edge> for BRepBuilderAPI_MakeEdge {
    unsafe fn value(self: Pin<&mut Self>) -> &TopoDS_Edge {
        self.Edge()
    }
}
