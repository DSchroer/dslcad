use crate::command::Builder;
use crate::explorer::Explorer;
use crate::shapes::DsShape;
use crate::{Error, Mesh, Point, Wire};
use cxx::UniquePtr;
use log::debug;
use opencascade_sys::ffi::{
    gp_Ax2_ctor, gp_DZ, gp_OX, gp_OY, gp_OZ, new_vec, BRepAlgoAPI_Common, BRepAlgoAPI_Cut,
    BRepAlgoAPI_Fuse, BRepAlgoAPI_Section, BRepBuilderAPI_MakeFace, BRepBuilderAPI_MakeFace_wire,
    BRepBuilderAPI_Transform, BRepFilletAPI_MakeChamfer, BRepFilletAPI_MakeChamfer_ctor,
    BRepFilletAPI_MakeFillet, BRepFilletAPI_MakeFillet_ctor, BRepGProp_VolumeProperties,
    BRepMesh_IncrementalMesh_ctor, BRepPrimAPI_MakeBox, BRepPrimAPI_MakeBox_ctor,
    BRepPrimAPI_MakeCylinder, BRepPrimAPI_MakeCylinder_ctor, BRepPrimAPI_MakePrism,
    BRepPrimAPI_MakePrism_ctor, BRepPrimAPI_MakeRevol, BRepPrimAPI_MakeRevol_ctor,
    BRepPrimAPI_MakeSphere, BRepPrimAPI_MakeSphere_ctor, BRep_Tool_Pnt, BRep_Tool_Triangulation,
    GProp_GProps_CentreOfMass, GProp_GProps_ctor, Handle_Poly_Triangulation_Get,
    Poly_Triangulation_Node, TopAbs_Orientation, TopAbs_ShapeEnum, TopExp_Explorer_ctor,
    TopLoc_Location_ctor, TopoDS_Edge, TopoDS_Shape, TopoDS_Shape_to_owned, TopoDS_cast_to_face,
};
use std::f64::consts::PI;

pub struct Shape {
    pub(crate) shape: UniquePtr<TopoDS_Shape>,
}

impl AsRef<TopoDS_Shape> for Shape {
    fn as_ref(&self) -> &TopoDS_Shape {
        &self.shape
    }
}

pub enum Axis {
    X,
    Y,
    Z,
}

impl DsShape for Shape {
    fn shape(&self) -> &TopoDS_Shape {
        &self.shape
    }
}

impl Shape {
    pub fn cube(dx: f64, dy: f64, dz: f64) -> Result<Self, Error> {
        let origin = Point::new(0., 0., 0.);
        let mut b = BRepPrimAPI_MakeBox_ctor(&origin.point, dx, dy, dz);
        Ok(Builder::try_build(&mut b)?.into())
    }

    pub fn sphere(r: f64) -> Result<Self, Error> {
        let axis = gp_Ax2_ctor(&Point::default().point, gp_DZ());
        let mut sphere = BRepPrimAPI_MakeSphere_ctor(&axis, r, 2. * PI);
        Ok(Builder::try_build(&mut sphere)?.into())
    }

    pub fn cylinder(radius: f64, height: f64) -> Result<Self, Error> {
        let origin = Point::new(radius, radius, 0.);
        let axis = gp_Ax2_ctor(&origin.point, gp_DZ());
        let mut cylinder = BRepPrimAPI_MakeCylinder_ctor(&axis, radius, height);
        Ok(Builder::try_build(&mut cylinder)?.into())
    }

    pub fn extrude(wire: &Wire, x: f64, y: f64, z: f64) -> Result<Self, Error> {
        let mut face_profile = BRepBuilderAPI_MakeFace_wire(wire.wire(), false);
        let prism_vec = new_vec(x, y, z);

        let mut body = BRepPrimAPI_MakePrism_ctor(
            Builder::try_build(&mut face_profile)?,
            &prism_vec,
            true,
            true,
        );
        Ok(Builder::try_build(&mut body)?.into())
    }

    pub fn extrude_rotate(wire: &Wire, axis: Axis, degrees: f64) -> Result<Self, Error> {
        let mut face_profile = BRepBuilderAPI_MakeFace_wire(wire.wire(), false);

        let radians = degrees * (std::f64::consts::PI / 180.);
        let gp_axis = match axis {
            Axis::X => gp_OX(),
            Axis::Y => gp_OY(),
            Axis::Z => gp_OZ(),
        };

        let mut body = BRepPrimAPI_MakeRevol_ctor(
            Builder::try_build(&mut face_profile)?,
            gp_axis,
            radians,
            true,
        );
        Ok(Builder::try_build(&mut body)?.into())
    }

    pub fn fillet(target: &Shape, thickness: f64) -> Result<Self, Error> {
        let mut fillet = BRepFilletAPI_MakeFillet_ctor(&target.shape);

        let mut edge_explorer: Explorer<TopoDS_Edge> = Explorer::new(target);
        while let Some(edge) = edge_explorer.next() {
            fillet.pin_mut().add_edge(thickness, edge);
        }

        Ok(Builder::try_build(&mut fillet)?.into())
    }

    pub fn chamfer(target: &Shape, thickness: f64) -> Result<Self, Error> {
        let mut chamfer = BRepFilletAPI_MakeChamfer_ctor(&target.shape);

        let mut edge_explorer: Explorer<TopoDS_Edge> = Explorer::new(target);
        while let Some(edge) = edge_explorer.next() {
            chamfer.pin_mut().add_edge(thickness, edge);
        }

        Ok(Builder::try_build(&mut chamfer)?.into())
    }

    pub fn center_of_mass(&self) -> Point {
        let mut props = GProp_GProps_ctor();
        BRepGProp_VolumeProperties(self.shape(), props.pin_mut());
        GProp_GProps_CentreOfMass(&props).into()
    }

    pub fn volume(&self) -> f64 {
        let mut props = GProp_GProps_ctor();
        BRepGProp_VolumeProperties(self.shape(), props.pin_mut());
        props.Mass()
    }

    pub fn mesh(&self, deflection: f64) -> Result<Mesh, Error> {
        let mut incremental_mesh = BRepMesh_IncrementalMesh_ctor(&self.shape, deflection);
        if !incremental_mesh.IsDone() {
            return Err("unable to build incremental mesh".into());
        }

        let mut mesh = Mesh::default();

        let mut edge_explorer = TopExp_Explorer_ctor(
            incremental_mesh.pin_mut().Shape(),
            TopAbs_ShapeEnum::TopAbs_FACE,
        );
        while edge_explorer.More() {
            let face = TopoDS_cast_to_face(edge_explorer.Current());
            let mut location = TopLoc_Location_ctor();

            let triangulation_handle = BRep_Tool_Triangulation(face, location.pin_mut());
            if let Ok(triangulation) = Handle_Poly_Triangulation_Get(&triangulation_handle) {
                let index_offset = mesh.vertices.len();
                for index in 1..=triangulation.NbNodes() {
                    let node = Poly_Triangulation_Node(triangulation, index);
                    mesh.vertices.push([node.X(), node.Y(), node.Z()]);
                }

                for index in 1..=triangulation.NbTriangles() {
                    let triangle = triangulation.Triangle(index);
                    if face.Orientation() == TopAbs_Orientation::TopAbs_FORWARD {
                        mesh.triangles.push([
                            index_offset + triangle.Value(1) as usize - 1,
                            index_offset + triangle.Value(2) as usize - 1,
                            index_offset + triangle.Value(3) as usize - 1,
                        ]);
                    } else {
                        mesh.triangles.push([
                            index_offset + triangle.Value(3) as usize - 1,
                            index_offset + triangle.Value(2) as usize - 1,
                            index_offset + triangle.Value(1) as usize - 1,
                        ]);
                    }
                }
            }

            edge_explorer.pin_mut().Next();
        }

        Ok(mesh)
    }

    pub fn lines(&self, deflection: f64) -> Result<Vec<Vec<[f64; 3]>>, Error> {
        let mut lines = Vec::new();

        let mut stats_length = 0;
        let mut edge_explorer: Explorer<TopoDS_Edge> = Explorer::new(self);
        while let Some(edge) = edge_explorer.next() {
            if let Some(line) = Wire::extract_line(edge, deflection) {
                stats_length += line.len();
                lines.push(line);
            }
        }

        debug!(
            "avg number of points per line: {}",
            stats_length / lines.len()
        );

        Ok(lines)
    }

    pub fn points(&self) -> Result<Vec<[f64; 3]>, Error> {
        let mut points = Vec::new();

        let mut vertex_explorer = Explorer::new(self);

        while let Some(vertex) = vertex_explorer.next() {
            let point: Point = BRep_Tool_Pnt(vertex).into();
            points.push(point.into());
        }

        Ok(points)
    }
}

impl From<&TopoDS_Shape> for Shape {
    fn from(value: &TopoDS_Shape) -> Self {
        Shape {
            shape: TopoDS_Shape_to_owned(value),
        }
    }
}

#[macro_export]
macro_rules! shape_builder {
    ($type_name: ty) => {
        impl $crate::command::Command for $type_name {
            fn name() -> &'static str {
                stringify!($type_name)
            }

            fn is_done(&self) -> bool {
                self.IsDone()
            }

            fn build(
                self: core::pin::Pin<&mut Self>,
                progress: &opencascade_sys::ffi::Message_ProgressRange,
            ) {
                self.Build(progress)
            }
        }

        impl Builder<TopoDS_Shape> for $type_name {
            unsafe fn value(self: core::pin::Pin<&mut Self>) -> &TopoDS_Shape {
                self.Shape()
            }
        }
    };
}

shape_builder!(BRepPrimAPI_MakeBox);
shape_builder!(BRepPrimAPI_MakeSphere);
shape_builder!(BRepPrimAPI_MakeCylinder);
shape_builder!(BRepPrimAPI_MakePrism);
shape_builder!(BRepFilletAPI_MakeFillet);
shape_builder!(BRepFilletAPI_MakeChamfer);
shape_builder!(BRepPrimAPI_MakeRevol);
shape_builder!(BRepAlgoAPI_Fuse);
shape_builder!(BRepAlgoAPI_Cut);
shape_builder!(BRepAlgoAPI_Common);
shape_builder!(BRepAlgoAPI_Section);
shape_builder!(BRepBuilderAPI_Transform);
shape_builder!(BRepBuilderAPI_MakeFace);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_write_box_stl() {
        let shape = Shape::cube(1., 10., 1.).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_write_sphere_stl() {
        let shape = Shape::sphere(1.).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_mesh_box_stl() {
        let shape = Shape::cube(1., 10., 1.).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_fillet_box_stl() {
        let b = Shape::cube(10., 10., 10.).unwrap();
        let shape = Shape::fillet(&b, 0.5).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_chamfer_box_stl() {
        let b = Shape::cube(10., 10., 10.).unwrap();
        let shape = Shape::chamfer(&b, 0.5).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_write_cylinder_stl() {
        let shape = Shape::cylinder(10., 100.).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_write_translated_stl() {
        let b = Shape::cube(10., 10., 10.).unwrap();
        let shape = Shape::translate(&b, &Point::new(10., 0., 0.)).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_write_rotated_stl() {
        let b = Shape::cube(10., 10., 10.).unwrap();
        let shape = Shape::rotate(&b, Axis::X, 45.).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_write_scaled_stl() {
        let b = Shape::cube(1., 1., 1.).unwrap();
        let shape = Shape::scale(&b, 10.).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_write_mirrored_stl() {
        let b = Shape::cube(1., 1., 1.).unwrap();
        let shape = Shape::mirror(&b, Axis::X).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_write_fuse_stl() {
        let b = Shape::cube(15., 15., 1.).unwrap();
        let c = Shape::cylinder(10., 100.).unwrap();
        let shape = Shape::fuse(&b, &c).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_write_cut_stl() {
        let b = Shape::cube(15., 15., 1.).unwrap();
        let c = Shape::cylinder(10., 100.).unwrap();
        let shape = Shape::cut(&b, &c).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_write_intersect_stl() {
        let b = Shape::cube(15., 15., 1.).unwrap();
        let c = Shape::cylinder(10., 100.).unwrap();
        let shape = Shape::intersect(&b, &c).unwrap();
        shape.mesh(0.1).unwrap();
    }
}
