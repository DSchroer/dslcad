use crate::command::{Builder, Command};
use crate::edge::Edge;
use crate::{DsShape, Error, Point};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    BRepBuilderAPI_MakeWire, BRepBuilderAPI_MakeWire_ctor, BRepGProp_LinearProperties,
    BRepOffsetAPI_MakeOffset, BRepOffsetAPI_MakeOffset_wire_ctor, BRep_Tool_Curve,
    GProp_GProps_CentreOfMass, GProp_GProps_ctor, GeomAbs_JoinType, HandleGeomCurve_Value,
    TopAbs_ShapeEnum, TopExp_Explorer_ctor, TopoDS_Edge, TopoDS_Shape, TopoDS_Shape_to_owned,
    TopoDS_Wire, TopoDS_cast_to_edge, TopoDS_cast_to_wire,
};
use std::pin::Pin;

pub struct WireFactory {
    make_wire: UniquePtr<BRepBuilderAPI_MakeWire>,
}

impl WireFactory {
    pub fn new() -> Self {
        WireFactory {
            make_wire: BRepBuilderAPI_MakeWire_ctor(),
        }
    }

    pub fn add_edge(&mut self, edge: &Edge) {
        self.make_wire.pin_mut().add_edge(&edge.0)
    }

    pub fn add_wire(&mut self, wire: &Wire) {
        self.make_wire.pin_mut().add_wire(wire.wire())
    }

    pub fn build(mut self) -> Result<Wire, Error> {
        Ok(Wire(TopoDS_Shape_to_owned(Builder::try_build(
            &mut self.make_wire,
        )?)))
    }
}

impl Default for WireFactory {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Wire(pub(crate) UniquePtr<TopoDS_Shape>);

impl DsShape for Wire {
    fn shape(&self) -> &TopoDS_Shape {
        &self.0
    }
}

impl Wire {
    pub(crate) fn wire(&self) -> &TopoDS_Wire {
        TopoDS_cast_to_wire(&self.0)
    }

    pub fn from_edge(left: &Edge) -> Result<Self, Error> {
        let mut wire_builder = BRepBuilderAPI_MakeWire_ctor();
        wire_builder.pin_mut().add_edge(&left.0);
        Ok(Wire(TopoDS_Shape_to_owned(Builder::try_build(
            &mut wire_builder,
        )?)))
    }

    pub fn add_edge(&self, left: &Edge) -> Result<Self, Error> {
        let mut wire_builder = BRepBuilderAPI_MakeWire_ctor();
        wire_builder.pin_mut().add_wire(self.wire());
        wire_builder.pin_mut().add_edge(&left.0);
        Ok(Wire(TopoDS_Shape_to_owned(Builder::try_build(
            &mut wire_builder,
        )?)))
    }

    pub fn join(&mut self, wire: &Wire) -> Result<Self, Error> {
        let mut wire_builder = BRepBuilderAPI_MakeWire_ctor();
        wire_builder.pin_mut().add_wire(self.wire());
        wire_builder.pin_mut().add_wire(wire.wire());
        Ok(Wire(TopoDS_Shape_to_owned(Builder::try_build(
            &mut wire_builder,
        )?)))
    }

    pub fn start(&self) -> Result<Option<Point>, Error> {
        let edge_explorer = TopExp_Explorer_ctor(&self.0, TopAbs_ShapeEnum::TopAbs_EDGE);
        if edge_explorer.More() {
            let edge = TopoDS_cast_to_edge(edge_explorer.Current());
            let (start, _) = Self::extract_start_end(edge);
            return Ok(Some(start));
        }
        Ok(None)
    }

    pub fn end(&self) -> Result<Option<Point>, Error> {
        let mut edge_explorer = TopExp_Explorer_ctor(&self.0, TopAbs_ShapeEnum::TopAbs_EDGE);
        let mut last_end = None;
        while edge_explorer.More() {
            let edge = TopoDS_cast_to_edge(edge_explorer.Current());
            let (_, end) = Self::extract_start_end(edge);
            last_end = Some(end);
            edge_explorer.pin_mut().Next();
        }
        Ok(last_end)
    }

    pub fn offset(&self, distance: f64) -> Result<Self, Error> {
        let mut offset =
            BRepOffsetAPI_MakeOffset_wire_ctor(self.wire(), GeomAbs_JoinType::GeomAbs_Arc);
        offset.pin_mut().Perform(distance, 0.0);
        Ok(Builder::try_build(&mut offset)?.into())
    }

    pub fn points(&self) -> Result<Vec<Vec<[f64; 3]>>, Error> {
        let mut lines = Vec::new();

        let mut edge_explorer = TopExp_Explorer_ctor(&self.0, TopAbs_ShapeEnum::TopAbs_EDGE);
        while edge_explorer.More() {
            let edge = TopoDS_cast_to_edge(edge_explorer.Current());

            lines.push(Self::extract_line(edge));
            edge_explorer.pin_mut().Next();
        }

        Ok(lines)
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

    pub fn center_of_mass(&self) -> Point {
        let mut props = GProp_GProps_ctor();
        BRepGProp_LinearProperties(self.shape(), props.pin_mut());
        GProp_GProps_CentreOfMass(&props).into()
    }
}

impl From<&TopoDS_Shape> for Wire {
    fn from(value: &TopoDS_Shape) -> Self {
        Wire(TopoDS_Shape_to_owned(value))
    }
}

impl Command for BRepBuilderAPI_MakeWire {
    fn is_done(&self) -> bool {
        self.IsDone()
    }

    fn build(self: Pin<&mut Self>, progress: &opencascade_sys::ffi::Message_ProgressRange) {
        self.Build(progress)
    }
}

impl Command for BRepOffsetAPI_MakeOffset {
    fn is_done(&self) -> bool {
        self.IsDone()
    }

    fn build(self: Pin<&mut Self>, progress: &opencascade_sys::ffi::Message_ProgressRange) {
        self.Build(progress)
    }
}

impl Builder<TopoDS_Shape> for BRepOffsetAPI_MakeOffset {
    unsafe fn value(self: Pin<&mut Self>) -> &TopoDS_Shape {
        self.Shape()
    }
}

impl Builder<TopoDS_Shape> for BRepBuilderAPI_MakeWire {
    unsafe fn value(self: Pin<&mut Self>) -> &TopoDS_Shape {
        self.Shape()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_find_points() {
        let mut wire = WireFactory::new();
        wire.add_edge(&Edge::new_line(&Point::new(0., 0., 0.), &Point::new(0., 10., 0.)).unwrap());
        let wire = wire.build().unwrap();

        assert!(!wire.points().unwrap().is_empty());
    }
}
