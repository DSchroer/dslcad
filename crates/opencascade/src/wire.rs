use crate::command::Builder;
use crate::edge::Edge;
use crate::{Error, Point};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    BRepBuilderAPI_MakeWire, BRepBuilderAPI_MakeWire_ctor, BRep_Tool_Curve, HandleGeomCurve_Value,
    TopAbs_ShapeEnum, TopExp_Explorer_ctor, TopoDS_Edge, TopoDS_cast_to_edge,
};

pub struct Wire(pub(crate) UniquePtr<BRepBuilderAPI_MakeWire>);

impl Wire {
    pub fn new() -> Self {
        let make_wire = BRepBuilderAPI_MakeWire_ctor();
        Wire(make_wire)
    }

    pub fn add_edge(&mut self, left: &mut Edge) -> Result<(), Error> {
        self.0.pin_mut().add_edge(left.try_build()?);
        Ok(())
    }

    pub fn join(&mut self, wire: &mut Wire) {
        self.0.pin_mut().add_wire(wire.0.pin_mut().Wire());
    }

    pub fn start(&mut self) -> Option<Point> {
        let edge_explorer =
            TopExp_Explorer_ctor(self.0.pin_mut().Shape(), TopAbs_ShapeEnum::TopAbs_EDGE);
        if edge_explorer.More() {
            let edge = TopoDS_cast_to_edge(edge_explorer.Current());
            let (start, _) = Self::extract_start_end(edge);
            return Some(start);
        }
        None
    }

    pub fn end(&mut self) -> Option<Point> {
        let mut edge_explorer =
            TopExp_Explorer_ctor(self.0.pin_mut().Shape(), TopAbs_ShapeEnum::TopAbs_EDGE);
        let mut last_end = None;
        while edge_explorer.More() {
            let edge = TopoDS_cast_to_edge(edge_explorer.Current());
            let (_, end) = Self::extract_start_end(edge);
            last_end = Some(end);
            edge_explorer.pin_mut().Next();
        }
        last_end
    }

    pub fn points(&mut self) -> Vec<Vec<[f64; 3]>> {
        let mut lines = Vec::new();

        let mut edge_explorer =
            TopExp_Explorer_ctor(self.0.pin_mut().Shape(), TopAbs_ShapeEnum::TopAbs_EDGE);
        while edge_explorer.More() {
            let edge = TopoDS_cast_to_edge(edge_explorer.Current());

            lines.push(Self::extract_line(edge));
            edge_explorer.pin_mut().Next();
        }

        lines
    }

    fn extract_line(edge: &TopoDS_Edge) -> Vec<[f64; 3]> {
        let mut first = 0.;
        let mut last = 0.;
        let curve = BRep_Tool_Curve(edge, &mut first, &mut last);

        let mut points = Vec::new();
        for u in 0..=10 {
            let point: Point =
                HandleGeomCurve_Value(&curve, first + (((last - first) / 10.0) * u as f64)).into();
            points.push(point.into())
        }
        points
    }

    fn extract_start_end(edge: &TopoDS_Edge) -> (Point, Point) {
        let mut first = 0.;
        let mut last = 0.;
        let curve = BRep_Tool_Curve(edge, &mut first, &mut last);

        let start = HandleGeomCurve_Value(&curve, first).into();
        let end = HandleGeomCurve_Value(&curve, last).into();

        (start, end)
    }
}

impl Default for Wire {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_find_points() {
        let mut wire = Wire::new();
        wire.add_edge(&mut Edge::new_line(
            &Point::new(0., 0., 0.),
            &Point::new(0., 10., 0.),
        ))
        .unwrap();

        assert!(!wire.points().is_empty());
    }
}
