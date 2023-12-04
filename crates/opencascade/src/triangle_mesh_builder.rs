use crate::command::Builder;
use crate::{Edge, Error, Point, Shape, WireFactory};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    BRepBuilderAPI_MakeFace_wire, BRep_Builder, BRep_Builder_ctor,
    BRep_Builder_upcast_to_topods_builder, TopoDS_Compound_as_shape, TopoDS_Compound_ctor,
    TopoDS_Shape,
};

pub struct TriangleMeshBuilder {
    result: UniquePtr<TopoDS_Shape>,
    builder: UniquePtr<BRep_Builder>,
}

impl Default for TriangleMeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TriangleMeshBuilder {
    pub fn new() -> Self {
        let builder = BRep_Builder_ctor();
        let mut compound = TopoDS_Compound_ctor();

        BRep_Builder_upcast_to_topods_builder(&builder).MakeCompound(compound.pin_mut());

        Self {
            result: TopoDS_Compound_as_shape(compound),
            builder: BRep_Builder_ctor(),
        }
    }

    pub fn add_triangle(&mut self, points: [Point; 3]) -> Result<(), Error> {
        let mut wf = WireFactory::new();
        wf.add_edge(&Edge::new_line(&points[0], &points[1])?);
        wf.add_edge(&Edge::new_line(&points[1], &points[2])?);
        wf.add_edge(&Edge::new_line(&points[2], &points[0])?);

        let mut binding = BRepBuilderAPI_MakeFace_wire(wf.build()?.wire(), true);
        let face = Builder::try_build(&mut binding)?;

        BRep_Builder_upcast_to_topods_builder(&self.builder).Add(self.result.pin_mut(), face);

        Ok(())
    }

    pub fn build(self) -> Result<Shape, Error> {
        Ok(Shape { shape: self.result })
    }
}
