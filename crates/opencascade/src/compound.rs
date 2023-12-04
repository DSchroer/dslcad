use crate::command::Builder;
use crate::explorer::Explorer;
use crate::{Error, Wire};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    cast_compound_to_shape, BRepBuilderAPI_MakeWire_ctor, TopoDS_Compound,
    TopoDS_Compound_to_owned, TopoDS_Edge, TopoDS_Shape, TopoDS_Shape_to_owned,
};

pub struct Compound(pub(crate) UniquePtr<TopoDS_Compound>);

impl AsRef<TopoDS_Shape> for Compound {
    fn as_ref(&self) -> &TopoDS_Shape {
        cast_compound_to_shape(&self.0)
    }
}

impl From<&TopoDS_Compound> for Compound {
    fn from(value: &TopoDS_Compound) -> Self {
        Compound(TopoDS_Compound_to_owned(value))
    }
}

impl TryFrom<Compound> for Wire {
    type Error = Error;

    fn try_from(value: Compound) -> Result<Self, Self::Error> {
        let mut explorer = Explorer::<TopoDS_Edge>::new(value);
        let mut wire_builder = BRepBuilderAPI_MakeWire_ctor();

        while let Some(edge) = explorer.next() {
            wire_builder.pin_mut().add_edge(edge);
        }

        Ok(Wire(TopoDS_Shape_to_owned(Builder::try_build(
            &mut wire_builder,
        )?)))
    }
}
