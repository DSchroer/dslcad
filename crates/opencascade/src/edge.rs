use crate::command::{Builder, Command};
use crate::Point;
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    new_HandleGeomCurve_from_HandleGeom_TrimmedCurve, BRepBuilderAPI_MakeEdge,
    BRepBuilderAPI_MakeEdge_HandleGeomCurve, GC_MakeArcOfCircle_Value,
    GC_MakeArcOfCircle_point_point_point, GC_MakeSegment_Value, GC_MakeSegment_point_point,
    Message_ProgressRange, TopoDS_Edge,
};

pub struct Edge(pub(crate) UniquePtr<BRepBuilderAPI_MakeEdge>);

impl Edge {
    pub fn new_line(a: &Point, b: &Point) -> Self {
        let segment = GC_MakeSegment_point_point(&a.point, &b.point);
        let edge_1 = BRepBuilderAPI_MakeEdge_HandleGeomCurve(
            &new_HandleGeomCurve_from_HandleGeom_TrimmedCurve(&GC_MakeSegment_Value(&segment)),
        );
        Edge(edge_1)
    }

    pub fn new_arc(a: &Point, b: &Point, c: &Point) -> Self {
        let segment = GC_MakeArcOfCircle_point_point_point(&a.point, &b.point, &c.point);
        let edge_1 = BRepBuilderAPI_MakeEdge_HandleGeomCurve(
            &new_HandleGeomCurve_from_HandleGeom_TrimmedCurve(&GC_MakeArcOfCircle_Value(&segment)),
        );
        Edge(edge_1)
    }
}

impl Command for Edge {
    fn is_done(&self) -> bool {
        self.0.IsDone()
    }

    fn build(&mut self, progress: &Message_ProgressRange) {
        self.0.pin_mut().Build(progress)
    }
}

impl Builder<TopoDS_Edge> for Edge {
    unsafe fn value(&mut self) -> &TopoDS_Edge {
        self.0.pin_mut().Edge()
    }
}
