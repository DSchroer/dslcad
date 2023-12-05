use crate::command::Builder;
use crate::explorer::Explorer;
use crate::shape_builder;
use crate::{Error, Point, Shape};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    cast_face_to_shape, BRepBuilderAPI_MakeShapeOnMesh, BRepBuilderAPI_MakeShapeOnMesh_ctor,
    BRepBuilderAPI_MakeSolid, BRepBuilderAPI_MakeSolid_ctor, BRep_Builder_ctor,
    BRep_Builder_upcast_to_topods_builder, Handle_Poly_Triangulation_ctor, Poly_Triangle_ctor,
    Poly_Triangulation, Poly_Triangulation_ctor, TopoDS_Face, TopoDS_Shape, TopoDS_Shell_as_shape,
    TopoDS_Shell_ctor, TopoDS_cast_to_shell,
};

pub struct TriangleMesh(UniquePtr<Poly_Triangulation>);

impl TriangleMesh {
    pub fn new(
        vertexes: impl IntoIterator<Item = Point>,
        triangles: impl IntoIterator<Item = [usize; 3]>,
    ) -> Self {
        let vertexes: Vec<_> = vertexes.into_iter().collect();
        let triangles: Vec<_> = triangles.into_iter().collect();

        let mut triangulation =
            Poly_Triangulation_ctor(vertexes.len() as i32, triangles.len() as i32, false, false);

        for (i, vertex) in vertexes.iter().enumerate() {
            triangulation
                .pin_mut()
                .SetNode((i + 1) as i32, vertex.as_ref())
        }

        for (i, triangle) in triangles.iter().enumerate() {
            let t = Poly_Triangle_ctor(
                (triangle[0] + 1) as i32,
                (triangle[1] + 1) as i32,
                (triangle[2] + 1) as i32,
            );
            triangulation.pin_mut().SetTriangle((i + 1) as i32, &t)
        }

        Self(triangulation)
    }
}

impl TryInto<Shape> for TriangleMesh {
    type Error = Error;

    fn try_into(self) -> Result<Shape, Self::Error> {
        let ptr = self.0.into_raw();
        let handle = unsafe { Handle_Poly_Triangulation_ctor(ptr) };
        let mut shape_on_mesh = BRepBuilderAPI_MakeShapeOnMesh_ctor(&handle);
        let shape: Shape = Builder::<TopoDS_Shape>::try_build(&mut shape_on_mesh)?.into();

        let mut shell = TopoDS_Shell_ctor();
        let builder = BRep_Builder_ctor();
        let builder = BRep_Builder_upcast_to_topods_builder(&builder);
        builder.MakeShell(shell.pin_mut());
        let mut shell = TopoDS_Shell_as_shape(shell);

        let mut faces = Explorer::<TopoDS_Face>::new(shape);
        while let Some(face) = faces.next() {
            builder.Add(shell.pin_mut(), cast_face_to_shape(face))
        }

        let mut make_solid = BRepBuilderAPI_MakeSolid_ctor(TopoDS_cast_to_shell(&shell));

        let solid = Builder::try_build(&mut make_solid)?;

        Ok(solid.into())
    }
}

shape_builder!(BRepBuilderAPI_MakeShapeOnMesh);
shape_builder!(BRepBuilderAPI_MakeSolid);
