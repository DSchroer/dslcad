use crate::command::{Builder, Command};
use crate::edge::Edge;
use crate::explorer::Explorer;
use crate::{DsShape, Error, Point};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    cast_wire_to_shape, BRepBuilderAPI_MakeWire, BRepBuilderAPI_MakeWire_ctor,
    BRepGProp_LinearProperties, BRepOffsetAPI_MakeOffset, BRepOffsetAPI_MakeOffset_wire_ctor,
    BRep_Builder_ctor, BRep_Builder_upcast_to_topods_builder, BRep_Tool_Curve,
    GProp_GProps_CentreOfMass, GProp_GProps_ctor, GeomAbs_JoinType, HandleGeomCurve,
    HandleGeomCurve_Value, TopAbs_ShapeEnum, TopExp_Explorer_ctor, TopoDS_Compound_as_shape,
    TopoDS_Compound_ctor, TopoDS_Edge, TopoDS_Shape, TopoDS_Shape_to_owned, TopoDS_Wire,
    TopoDS_cast_to_edge, TopoDS_cast_to_wire,
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
        if wire.is_compound() {
            for contour in wire.contours() {
                self.make_wire.pin_mut().add_wire(contour.wire());
            }
        } else {
            self.make_wire.pin_mut().add_wire(wire.wire());
        }
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

impl AsRef<TopoDS_Shape> for Wire {
    fn as_ref(&self) -> &TopoDS_Shape {
        &self.0
    }
}

impl Clone for Wire {
    fn clone(&self) -> Self {
        Wire(TopoDS_Shape_to_owned(&self.0))
    }
}

impl Wire {
    pub(crate) fn wire(&self) -> &TopoDS_Wire {
        TopoDS_cast_to_wire(&self.0)
    }

    fn as_wire(&self) -> Result<&TopoDS_Wire, Error> {
        if self.is_compound() {
            Err("wire contains multiple contours".into())
        } else {
            Ok(self.wire())
        }
    }

    pub fn compound(contours: &[Wire]) -> Result<Self, Error> {
        let mut compound = TopoDS_Compound_ctor();
        let builder = BRep_Builder_ctor();
        let topods_builder = BRep_Builder_upcast_to_topods_builder(&builder);

        topods_builder.MakeCompound(compound.pin_mut());

        let mut shape = TopoDS_Compound_as_shape(compound);
        for contour in contours {
            topods_builder.Add(shape.pin_mut(), contour.shape());
        }

        Ok(Wire(TopoDS_Shape_to_owned(&shape)))
    }

    pub fn is_compound(&self) -> bool {
        matches!(self.0.ShapeType(), TopAbs_ShapeEnum::TopAbs_COMPOUND)
    }

    pub fn contours(&self) -> Vec<Wire> {
        let mut explorer: Explorer<TopoDS_Wire> = Explorer::new(self);
        let mut contours = Vec::new();
        while let Some(contour) = explorer.next() {
            contours.push(Wire::from(cast_wire_to_shape(contour)));
        }
        contours
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
        wire_builder.pin_mut().add_wire(self.as_wire()?);
        wire_builder.pin_mut().add_edge(&left.0);
        Ok(Wire(TopoDS_Shape_to_owned(Builder::try_build(
            &mut wire_builder,
        )?)))
    }

    pub fn join(&mut self, wire: &Wire) -> Result<Self, Error> {
        let mut wire_builder = BRepBuilderAPI_MakeWire_ctor();
        wire_builder.pin_mut().add_wire(self.as_wire()?);
        wire_builder.pin_mut().add_wire(wire.as_wire()?);
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
            BRepOffsetAPI_MakeOffset_wire_ctor(self.as_wire()?, GeomAbs_JoinType::GeomAbs_Arc);
        offset.pin_mut().Perform(distance, 0.0);
        Ok(Builder::try_build(&mut offset)?.into())
    }

    pub fn points(&self, deflection: f64) -> Result<Vec<Vec<[f64; 3]>>, Error> {
        let mut lines = Vec::new();

        let mut edge_explorer = TopExp_Explorer_ctor(&self.0, TopAbs_ShapeEnum::TopAbs_EDGE);
        while edge_explorer.More() {
            let edge = TopoDS_cast_to_edge(edge_explorer.Current());

            lines.push(Self::extract_line(edge, deflection).unwrap());
            edge_explorer.pin_mut().Next();
        }

        Ok(lines)
    }

    pub fn extract_line(edge: &TopoDS_Edge, deflection: f64) -> Option<Vec<[f64; 3]>> {
        let mut first = 0.;
        let mut last = 0.;
        let curve = BRep_Tool_Curve(edge, &mut first, &mut last);
        if curve.IsNull() {
            return None;
        }

        let points = Self::points_on_curve(&curve, first, last, deflection);
        Some(points)
    }

    /// calculate points along a curve under a maximum `linear deflection`
    fn points_on_curve(
        curve: &HandleGeomCurve,
        start: f64,
        end: f64,
        deflection: f64,
    ) -> Vec<[f64; 3]> {
        let mid = (start + end) / 2.0;

        let a: Point = HandleGeomCurve_Value(curve, start).into();
        let b: Point = HandleGeomCurve_Value(curve, mid).into();
        let c: Point = HandleGeomCurve_Value(curve, end).into();

        let m = (a.clone() + c.clone()) / 2.0;

        if m.distance(&b) > deflection {
            [
                Self::points_on_curve(curve, start, mid, deflection),
                Self::points_on_curve(curve, mid, end, deflection),
            ]
            .concat()
        } else {
            vec![a.into(), c.into()]
        }
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

    fn name() -> &'static str {
        stringify!(BRepBuilderAPI_MakeWire)
    }
}

impl Command for BRepOffsetAPI_MakeOffset {
    fn is_done(&self) -> bool {
        self.IsDone()
    }

    fn build(self: Pin<&mut Self>, progress: &opencascade_sys::ffi::Message_ProgressRange) {
        self.Build(progress)
    }

    fn name() -> &'static str {
        stringify!(BRepOffsetAPI_MakeOffset)
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

    fn line(from: Point, to: Point) -> Wire {
        let mut wire = WireFactory::new();
        wire.add_edge(&Edge::new_line(&from, &to).unwrap());
        wire.build().unwrap()
    }

    #[test]
    fn it_can_find_points() {
        let mut wire = WireFactory::new();
        wire.add_edge(&Edge::new_line(&Point::new(0., 0., 0.), &Point::new(0., 10., 0.)).unwrap());
        let wire = wire.build().unwrap();

        assert!(!wire.points(0.1).unwrap().is_empty());
    }

    #[test]
    fn it_can_build_compounds() {
        let compound = Wire::compound(&[
            line(Point::new(0., 0., 0.), Point::new(0., 10., 0.)),
            line(Point::new(10., 0., 0.), Point::new(10., 10., 0.)),
        ])
        .unwrap();

        assert!(compound.is_compound());
        assert_eq!(2, compound.contours().len());
        assert_eq!(2, compound.points(0.1).unwrap().len());
    }

    #[test]
    fn it_rejects_operations_on_compounds() {
        let compound = Wire::compound(&[
            line(Point::new(0., 0., 0.), Point::new(0., 10., 0.)),
            line(Point::new(10., 0., 0.), Point::new(10., 10., 0.)),
        ])
        .unwrap();

        assert!(compound.offset(1.).is_err());
        assert!(compound
            .add_edge(&Edge::new_line(&Point::new(0., 0., 0.), &Point::new(1., 1., 0.)).unwrap())
            .is_err());
    }
}
