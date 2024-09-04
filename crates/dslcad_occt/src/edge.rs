use crate::command::{Builder, Command};
use crate::{Error, Point};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    new_HandleGeomCurve_from_HandleGeom_TrimmedCurve, BRepBuilderAPI_MakeEdge,
    BRepBuilderAPI_MakeEdge_HandleGeomCurve, BRep_Tool_Curve, GC_MakeArcOfCircle_Value,
    GC_MakeArcOfCircle_point_point_point, GC_MakeSegment_Value, GC_MakeSegment_point_point,
    HandleGeomCurve_Value, TopoDS_Edge, TopoDS_Edge_to_owned,
};
use std::fmt::{Debug, Formatter};
use std::pin::Pin;

pub struct Edge(pub(crate) UniquePtr<TopoDS_Edge>);

impl Debug for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (start, end) = self.start_end();

        f.debug_struct("Edge")
            .field("start", &start)
            .field("end", &end)
            .finish()
    }
}

impl Edge {
    pub fn new_line(a: &Point, b: &Point) -> Result<Self, Error> {
        let segment = GC_MakeSegment_point_point(&a.point, &b.point);
        let mut edge_1 = BRepBuilderAPI_MakeEdge_HandleGeomCurve(
            &new_HandleGeomCurve_from_HandleGeom_TrimmedCurve(&GC_MakeSegment_Value(&segment)),
        );
        Ok(Edge(TopoDS_Edge_to_owned(Builder::try_build(&mut edge_1)?)))
    }

    pub fn new_arc(a: &Point, b: &Point, c: &Point) -> Result<Self, Error> {
        let segment = GC_MakeArcOfCircle_point_point_point(&a.point, &b.point, &c.point);
        let mut edge_1 = BRepBuilderAPI_MakeEdge_HandleGeomCurve(
            &new_HandleGeomCurve_from_HandleGeom_TrimmedCurve(&GC_MakeArcOfCircle_Value(&segment)),
        );
        Ok(Edge(TopoDS_Edge_to_owned(Builder::try_build(&mut edge_1)?)))
    }

    pub fn start_end(&self) -> (Point, Point) {
        let mut first = 0.;
        let mut last = 0.;
        let curve = BRep_Tool_Curve(&self.0, &mut first, &mut last);

        let start = HandleGeomCurve_Value(&curve, first).into();
        let end = HandleGeomCurve_Value(&curve, last).into();

        (start, end)
    }
}

impl From<UniquePtr<TopoDS_Edge>> for Edge {
    fn from(value: UniquePtr<TopoDS_Edge>) -> Self {
        Edge(value)
    }
}

impl Command for BRepBuilderAPI_MakeEdge {
    fn name() -> &'static str {
        stringify!(BRepBuilderAPI_MakeEdge)
    }

    fn is_done(&self) -> bool {
        self.IsDone()
    }

    fn build(self: Pin<&mut Self>, progress: &opencascade_sys::ffi::Message_ProgressRange) {
        self.Build(progress)
    }
}

impl Builder<TopoDS_Edge> for BRepBuilderAPI_MakeEdge {
    unsafe fn value(self: Pin<&mut Self>) -> &TopoDS_Edge {
        self.Edge()
    }
}
