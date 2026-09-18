use crate::command::{Builder, Command};
use crate::{Error, Point};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    new_HandleGeomCurve_from_HandleGeom_BezierCurve,
    new_HandleGeomCurve_from_HandleGeom_TrimmedCurve, BRepBuilderAPI_MakeEdge,
    BRepBuilderAPI_MakeEdge_HandleGeomCurve, BRep_Tool_Curve, GC_MakeArcOfCircle_Value,
    GC_MakeArcOfCircle_point_point_point, GC_MakeSegment_Value, GC_MakeSegment_point_point,
    Geom_BezierCurve_ctor_points, Geom_BezierCurve_to_handle, HandleGeomCurve_Value,
    TColgp_HArray1OfPnt_ctor, TopoDS_Edge, TopoDS_Edge_to_owned,
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

    pub fn new_bezier(points: &[Point]) -> Result<Self, Error> {
        let mut poles = TColgp_HArray1OfPnt_ctor(1, points.len() as i32);
        for (index, point) in points.iter().enumerate() {
            poles.pin_mut().SetValue(index as i32 + 1, &point.point);
        }

        let curve = new_HandleGeomCurve_from_HandleGeom_BezierCurve(&Geom_BezierCurve_to_handle(
            Geom_BezierCurve_ctor_points(&poles),
        ));

        let mut edge = BRepBuilderAPI_MakeEdge_HandleGeomCurve(&curve);
        Ok(Edge(TopoDS_Edge_to_owned(Builder::try_build(&mut edge)?)))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_create_bezier_edges() {
        let edge = Edge::new_bezier(&[
            Point::new(0., 0., 0.),
            Point::new(1., 2., 0.),
            Point::new(3., 2., 0.),
            Point::new(4., 0., 0.),
        ])
        .unwrap();

        let (start, end) = edge.start_end();
        assert_eq!(0., start.x());
        assert_eq!(4., end.x());
    }
}
