use crate::Point;
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    new_HandleGeomCurve_from_HandleGeom_TrimmedCurve, BRepBuilderAPI_MakeEdge_HandleGeomCurve,
    BRepBuilderAPI_MakeWire, BRepBuilderAPI_MakeWire_ctor, GC_MakeArcOfCircle_Value,
    GC_MakeArcOfCircle_point_point_point, GC_MakeSegment_Value, GC_MakeSegment_point_point,
};

pub struct Edge(pub(crate) UniquePtr<BRepBuilderAPI_MakeWire>);

impl Edge {
    pub fn new() -> Self {
        let make_wire = BRepBuilderAPI_MakeWire_ctor();
        Edge(make_wire)
    }

    pub fn add_line(&mut self, a: &Point, b: &Point) {
        let segment = GC_MakeSegment_point_point(&a.point, &b.point);
        let mut edge_1 = BRepBuilderAPI_MakeEdge_HandleGeomCurve(
            &new_HandleGeomCurve_from_HandleGeom_TrimmedCurve(&GC_MakeSegment_Value(&segment)),
        );
        self.0.pin_mut().add_edge(edge_1.pin_mut().Edge());
    }

    pub fn add_arc(&mut self, a: &Point, b: &Point, c: &Point) {
        let segment = GC_MakeArcOfCircle_point_point_point(&a.point, &b.point, &c.point);
        let mut edge_1 = BRepBuilderAPI_MakeEdge_HandleGeomCurve(
            &new_HandleGeomCurve_from_HandleGeom_TrimmedCurve(&GC_MakeArcOfCircle_Value(&segment)),
        );
        self.0.pin_mut().add_edge(edge_1.pin_mut().Edge());
    }

    pub fn join(&mut self, left: &mut Edge, right: &mut Edge) {
        self.0.pin_mut().add_wire(left.0.pin_mut().Wire());
        self.0.pin_mut().add_wire(right.0.pin_mut().Wire());
    }
}

impl Default for Edge {
    fn default() -> Self {
        Self::new()
    }
}
