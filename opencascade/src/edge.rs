use crate::Point;
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    new_HandleGeomCurve_from_HandleGeom_TrimmedCurve, BRepBuilderAPI_MakeEdge_HandleGeomCurve,
    BRepBuilderAPI_MakeWire, BRepBuilderAPI_MakeWire_ctor, BRep_Tool_Curve,
    GC_MakeArcOfCircle_Value, GC_MakeArcOfCircle_point_point_point, GC_MakeSegment_Value,
    GC_MakeSegment_point_point, HandleGeomCurve_Value, TopAbs_ShapeEnum, TopExp_Explorer_ctor,
    TopoDS_cast_to_edge,
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

    pub fn points(&mut self) {
        let mut edge_explorer =
            TopExp_Explorer_ctor(self.0.pin_mut().Shape(), TopAbs_ShapeEnum::TopAbs_EDGE);
        while edge_explorer.More() {
            let edge = TopoDS_cast_to_edge(edge_explorer.Current());
            let mut first = 0.;
            let mut last = 0.;
            let curve = BRep_Tool_Curve(edge, &mut first, &mut last);

            let pnt: Point = HandleGeomCurve_Value(&curve, 6.).into();
            println!(". {:?}", pnt);
            edge_explorer.pin_mut().Next();
        }
    }
}

impl Default for Edge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_find_points() {
        let mut edge = Edge::new();
        edge.add_line(&Point::new(0., 0., 0.), &Point::new(0., 10., 0.));

        edge.points();
    }
}
