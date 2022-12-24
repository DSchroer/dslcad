use crate::command::{Builder, Command};
use crate::explorer::Explorer;
use crate::{Error, Mesh, Point, Wire};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    gp_Ax2_ctor, gp_DZ, gp_OX, gp_OY, gp_OZ, new_transform, new_vec, BRepAlgoAPI_Common,
    BRepAlgoAPI_Common_ctor, BRepAlgoAPI_Cut, BRepAlgoAPI_Cut_ctor, BRepAlgoAPI_Fuse,
    BRepAlgoAPI_Fuse_ctor, BRepBuilderAPI_MakeFace_wire, BRepBuilderAPI_Transform,
    BRepBuilderAPI_Transform_ctor, BRepFilletAPI_MakeChamfer, BRepFilletAPI_MakeChamfer_ctor,
    BRepFilletAPI_MakeFillet, BRepFilletAPI_MakeFillet_ctor, BRepMesh_IncrementalMesh_ctor,
    BRepPrimAPI_MakeBox, BRepPrimAPI_MakeBox_ctor, BRepPrimAPI_MakeCylinder,
    BRepPrimAPI_MakeCylinder_ctor, BRepPrimAPI_MakePrism, BRepPrimAPI_MakePrism_ctor,
    BRepPrimAPI_MakeRevol, BRepPrimAPI_MakeRevol_ctor, BRepPrimAPI_MakeSphere,
    BRepPrimAPI_MakeSphere_ctor, BRep_Tool_Curve, BRep_Tool_Pnt, BRep_Tool_Triangulation,
    HandleGeomCurve_Value, Message_ProgressRange_ctor, Poly_Triangulation_Node, TopAbs_Orientation,
    TopAbs_ShapeEnum, TopExp_Explorer_ctor, TopLoc_Location_ctor, TopoDS_Edge, TopoDS_Shape,
    TopoDS_cast_to_face, TopoDS_cast_to_vertex,
};

pub enum Shape {
    Box(UniquePtr<BRepPrimAPI_MakeBox>),
    Cylinder(UniquePtr<BRepPrimAPI_MakeCylinder>),
    Sphere(UniquePtr<BRepPrimAPI_MakeSphere>),
    Fuse(UniquePtr<BRepAlgoAPI_Fuse>),
    Cut(UniquePtr<BRepAlgoAPI_Cut>),
    Intersect(UniquePtr<BRepAlgoAPI_Common>),
    Fillet(UniquePtr<BRepFilletAPI_MakeFillet>),
    Chamfer(UniquePtr<BRepFilletAPI_MakeChamfer>),
    Transformed(UniquePtr<BRepBuilderAPI_Transform>),
    Prism(UniquePtr<BRepPrimAPI_MakePrism>),
    Revol(UniquePtr<BRepPrimAPI_MakeRevol>),
}

pub enum Axis {
    X,
    Y,
    Z,
}

impl Shape {
    pub fn cube(dx: f64, dy: f64, dz: f64) -> Result<Self, Error> {
        let origin = Point::new(0., 0., 0.);
        Ok(Shape::Box(BRepPrimAPI_MakeBox_ctor(
            &origin.point,
            dx,
            dy,
            dz,
        )))
    }

    pub fn sphere(r: f64) -> Result<Self, Error> {
        Ok(Shape::Sphere(BRepPrimAPI_MakeSphere_ctor(r)))
    }

    pub fn cylinder(radius: f64, height: f64) -> Result<Self, Error> {
        let origin = Point::new(radius, radius, 0.);
        let axis = gp_Ax2_ctor(&origin.point, gp_DZ());
        Ok(Shape::Cylinder(BRepPrimAPI_MakeCylinder_ctor(
            &axis, radius, height,
        )))
    }

    pub fn fuse(left: &mut Shape, right: &mut Shape) -> Result<Self, Error> {
        Ok(Shape::Fuse(BRepAlgoAPI_Fuse_ctor(
            left.try_build()?,
            right.try_build()?,
        )))
    }

    pub fn cut(left: &mut Shape, right: &mut Shape) -> Result<Self, Error> {
        Ok(Shape::Cut(BRepAlgoAPI_Cut_ctor(
            left.try_build()?,
            right.try_build()?,
        )))
    }

    pub fn intersect(left: &mut Shape, right: &mut Shape) -> Result<Self, Error> {
        Ok(Shape::Intersect(BRepAlgoAPI_Common_ctor(
            left.try_build()?,
            right.try_build()?,
        )))
    }

    pub fn extrude(wire: &mut Wire, x: f64, y: f64, z: f64) -> Result<Self, Error> {
        let mut face_profile = BRepBuilderAPI_MakeFace_wire(wire.try_build()?, false);
        let prism_vec = new_vec(x, y, z);

        if !face_profile.IsDone() {
            let progress = Message_ProgressRange_ctor();
            face_profile.pin_mut().Build(&progress);

            if !face_profile.IsDone() {
                return Err("unable to compute shape".to_string().into());
            }
        }
        let body =
            BRepPrimAPI_MakePrism_ctor(face_profile.pin_mut().Shape(), &prism_vec, true, true);
        Ok(Shape::Prism(body))
    }

    pub fn extrude_rotate(wire: &mut Wire, axis: Axis, degrees: f64) -> Result<Self, Error> {
        let mut face_profile = BRepBuilderAPI_MakeFace_wire(wire.try_build()?, false);

        let radians = degrees * (std::f64::consts::PI / 180.);
        let gp_axis = match axis {
            Axis::X => gp_OX(),
            Axis::Y => gp_OY(),
            Axis::Z => gp_OZ(),
        };

        if !face_profile.IsDone() {
            let progress = Message_ProgressRange_ctor();
            face_profile.pin_mut().Build(&progress);

            if !face_profile.IsDone() {
                return Err("unable to compute shape".to_string().into());
            }
        }
        let body =
            BRepPrimAPI_MakeRevol_ctor(face_profile.pin_mut().Shape(), gp_axis, radians, true);
        Ok(Shape::Revol(body))
    }

    pub fn translate(left: &mut Shape, point: &Point) -> Result<Self, Error> {
        let mut transform = new_transform();
        transform
            .pin_mut()
            .SetTranslation(&Point::new(0., 0., 0.).point, &point.point);

        Ok(Shape::Transformed(BRepBuilderAPI_Transform_ctor(
            left.try_build()?,
            &transform,
            true,
        )))
    }

    pub fn rotate(left: &mut Shape, axis: Axis, degrees: f64) -> Result<Self, Error> {
        let mut transform = new_transform();
        let gp_axis = match axis {
            Axis::X => gp_OX(),
            Axis::Y => gp_OY(),
            Axis::Z => gp_OZ(),
        };
        let radians = degrees * (std::f64::consts::PI / 180.);
        transform.pin_mut().SetRotation(gp_axis, radians);

        Ok(Shape::Transformed(BRepBuilderAPI_Transform_ctor(
            left.try_build()?,
            &transform,
            true,
        )))
    }

    pub fn scale(left: &mut Shape, scale: f64) -> Result<Self, Error> {
        let mut transform = new_transform();
        transform
            .pin_mut()
            .SetScale(&Point::new(0., 0., 0.).point, scale);

        Ok(Shape::Transformed(BRepBuilderAPI_Transform_ctor(
            left.try_build()?,
            &transform,
            true,
        )))
    }

    pub fn mirror(left: &mut Shape, axis: Axis) -> Result<Self, Error> {
        let mut transform = new_transform();
        let gp_axis = match axis {
            Axis::X => gp_OX(),
            Axis::Y => gp_OY(),
            Axis::Z => gp_OZ(),
        };
        transform.pin_mut().set_mirror_axis(gp_axis);

        Ok(Shape::Transformed(BRepBuilderAPI_Transform_ctor(
            left.try_build()?,
            &transform,
            true,
        )))
    }

    pub fn fillet(target: &mut Shape, thickness: f64) -> Result<Self, Error> {
        let mut fillet = BRepFilletAPI_MakeFillet_ctor(target.try_build()?);

        let mut edge_explorer: Explorer<TopoDS_Edge> = Explorer::new(target)?;
        while let Some(edge) = edge_explorer.next() {
            fillet.pin_mut().add_edge(thickness, edge);
        }

        Ok(Shape::Fillet(fillet))
    }

    pub fn chamfer(target: &mut Shape, thickness: f64) -> Result<Self, Error> {
        let mut chamfer = BRepFilletAPI_MakeChamfer_ctor(target.try_build()?);

        let mut edge_explorer: Explorer<TopoDS_Edge> = Explorer::new(target)?;
        while let Some(edge) = edge_explorer.next() {
            chamfer.pin_mut().add_edge(thickness, edge);
        }

        Ok(Shape::Chamfer(chamfer))
    }

    pub fn mesh(&mut self) -> Result<Mesh, Error> {
        let mut incremental_mesh = BRepMesh_IncrementalMesh_ctor(self.try_build()?, 0.01);
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

            let triangulation_handle = BRep_Tool_Triangulation(&face, location.pin_mut());
            if !triangulation_handle.IsNull() {
                let triangulation = unsafe { &*triangulation_handle.get() };

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

    pub fn lines(&mut self) -> Result<Vec<Vec<[f64; 3]>>, Error> {
        let mut lines = Vec::new();

        let mut edge_explorer: Explorer<TopoDS_Edge> = Explorer::new(self)?;
        while let Some(edge) = edge_explorer.next() {
            if let Some(line) = Self::extract_line(edge) {
                lines.push(line);
            }
        }

        Ok(lines)
    }

    fn extract_line(edge: &TopoDS_Edge) -> Option<Vec<[f64; 3]>> {
        let mut first = 0.;
        let mut last = 0.;
        let curve = BRep_Tool_Curve(edge, &mut first, &mut last);
        if curve.IsNull() {
            return None;
        }

        let mut points = Vec::new();
        let cuts = 50;
        for u in 0..=cuts {
            let point: Point = HandleGeomCurve_Value(
                &curve,
                first + (((last - first) / (cuts as f64)) * u as f64),
            )
            .into();
            points.push(point.into())
        }
        Some(points)
    }

    pub fn points(&mut self) -> Result<Vec<[f64; 3]>, Error> {
        let mut points = Vec::new();

        let mut edge_explorer =
            TopExp_Explorer_ctor(self.try_build()?, TopAbs_ShapeEnum::TopAbs_VERTEX);
        while edge_explorer.More() {
            let vertex = TopoDS_cast_to_vertex(edge_explorer.Current());
            let point: Point = BRep_Tool_Pnt(vertex).into();
            points.push(point.into());
            edge_explorer.pin_mut().Next();
        }

        Ok(points)
    }
}

impl Command for Shape {
    fn is_done(&self) -> bool {
        match self {
            Shape::Box(b) => b.IsDone(),
            Shape::Sphere(b) => b.IsDone(),
            Shape::Cylinder(c) => c.IsDone(),
            Shape::Fuse(f) => f.IsDone(),
            Shape::Cut(f) => f.IsDone(),
            Shape::Intersect(f) => f.IsDone(),
            Shape::Fillet(f) => f.IsDone(),
            Shape::Chamfer(f) => f.IsDone(),
            Shape::Transformed(f) => f.IsDone(),
            Shape::Prism(p) => p.IsDone(),
            Shape::Revol(p) => p.IsDone(),
        }
    }

    fn build(&mut self, progress: &opencascade_sys::ffi::Message_ProgressRange) {
        match self {
            Shape::Box(b) => b.pin_mut().Build(progress),
            Shape::Sphere(b) => b.pin_mut().Build(progress),
            Shape::Cylinder(c) => c.pin_mut().Build(progress),
            Shape::Fuse(f) => f.pin_mut().Build(progress),
            Shape::Cut(f) => f.pin_mut().Build(progress),
            Shape::Intersect(f) => f.pin_mut().Build(progress),
            Shape::Fillet(f) => f.pin_mut().Build(progress),
            Shape::Chamfer(f) => f.pin_mut().Build(progress),
            Shape::Transformed(f) => f.pin_mut().Build(progress),
            Shape::Prism(p) => p.pin_mut().Build(progress),
            Shape::Revol(p) => p.pin_mut().Build(progress),
        }
    }
}

impl Builder<TopoDS_Shape> for Shape {
    unsafe fn value(&mut self) -> &TopoDS_Shape {
        match self {
            Shape::Box(b) => b.pin_mut().Shape(),
            Shape::Sphere(b) => b.pin_mut().Shape(),
            Shape::Cylinder(c) => c.pin_mut().Shape(),
            Shape::Fuse(f) => f.pin_mut().Shape(),
            Shape::Cut(f) => f.pin_mut().Shape(),
            Shape::Intersect(f) => f.pin_mut().Shape(),
            Shape::Fillet(f) => f.pin_mut().Shape(),
            Shape::Chamfer(f) => f.pin_mut().Shape(),
            Shape::Transformed(f) => f.pin_mut().Shape(),
            Shape::Prism(p) => p.pin_mut().Shape(),
            Shape::Revol(p) => p.pin_mut().Shape(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_write_box_stl() {
        let mut shape = Shape::cube(1., 10., 1.).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_write_sphere_stl() {
        let mut shape = Shape::sphere(1.).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_mesh_box_stl() {
        let mut shape = Shape::cube(1., 10., 1.).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_fillet_box_stl() {
        let mut b = Shape::cube(10., 10., 10.).unwrap();
        let mut shape = Shape::fillet(&mut b, 0.5).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_chamfer_box_stl() {
        let mut b = Shape::cube(10., 10., 10.).unwrap();
        let mut shape = Shape::chamfer(&mut b, 0.5).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_write_cylinder_stl() {
        let mut shape = Shape::cylinder(10., 100.).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_write_translated_stl() {
        let mut b = Shape::cube(10., 10., 10.).unwrap();
        let mut shape = Shape::translate(&mut b, &Point::new(10., 0., 0.)).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_write_rotated_stl() {
        let mut b = Shape::cube(10., 10., 10.).unwrap();
        let mut shape = Shape::rotate(&mut b, Axis::X, 45.).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_write_scaled_stl() {
        let mut b = Shape::cube(1., 1., 1.).unwrap();
        let mut shape = Shape::scale(&mut b, 10.).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_write_mirrored_stl() {
        let mut b = Shape::cube(1., 1., 1.).unwrap();
        let mut shape = Shape::mirror(&mut b, Axis::X).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_write_fuse_stl() {
        let mut b = Shape::cube(15., 15., 1.).unwrap();
        let mut c = Shape::cylinder(10., 100.).unwrap();
        let mut shape = Shape::fuse(&mut b, &mut c).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_write_cut_stl() {
        let mut b = Shape::cube(15., 15., 1.).unwrap();
        let mut c = Shape::cylinder(10., 100.).unwrap();
        let mut shape = Shape::cut(&mut b, &mut c).unwrap();
        shape.mesh().unwrap();
    }

    #[test]
    fn it_can_write_intersect_stl() {
        let mut b = Shape::cube(15., 15., 1.).unwrap();
        let mut c = Shape::cylinder(10., 100.).unwrap();
        let mut shape = Shape::intersect(&mut b, &mut c).unwrap();
        shape.mesh().unwrap();
    }
}
