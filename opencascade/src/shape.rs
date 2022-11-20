use crate::{Edge, Point};
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    gp_Ax2_ctor, gp_DZ, gp_OX, gp_OY, gp_OZ, new_transform, new_vec, write_stl, BRepAlgoAPI_Cut,
    BRepAlgoAPI_Cut_ctor, BRepAlgoAPI_Fuse, BRepAlgoAPI_Fuse_ctor, BRepBuilderAPI_MakeFace_wire,
    BRepBuilderAPI_Transform, BRepBuilderAPI_Transform_ctor, BRepFilletAPI_MakeChamfer,
    BRepFilletAPI_MakeChamfer_ctor, BRepFilletAPI_MakeFillet, BRepFilletAPI_MakeFillet_ctor,
    BRepMesh_IncrementalMesh_ctor, BRepPrimAPI_MakeBox, BRepPrimAPI_MakeBox_ctor,
    BRepPrimAPI_MakeCylinder, BRepPrimAPI_MakeCylinder_ctor, BRepPrimAPI_MakePrism,
    BRepPrimAPI_MakePrism_ctor, BRepPrimAPI_MakeRevol, BRepPrimAPI_MakeRevol_ctor,
    StlAPI_Writer_ctor, TopAbs_ShapeEnum, TopExp_Explorer_ctor, TopoDS_Shape, TopoDS_cast_to_edge,
};
use std::env;
use std::fs::File;
use std::io::ErrorKind;

pub use stl_io::IndexedMesh;

pub enum Shape {
    Box(UniquePtr<BRepPrimAPI_MakeBox>),
    Cylinder(UniquePtr<BRepPrimAPI_MakeCylinder>),
    Fuse(UniquePtr<BRepAlgoAPI_Fuse>),
    Cut(UniquePtr<BRepAlgoAPI_Cut>),
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
    pub fn cube(dx: f64, dy: f64, dz: f64) -> Self {
        let origin = Point::new(0., 0., 0.);
        Shape::Box(BRepPrimAPI_MakeBox_ctor(&origin.point, dx, dy, dz))
    }

    pub fn cylinder(radius: f64, height: f64) -> Self {
        let origin = Point::new(radius, radius, 0.);
        let axis = gp_Ax2_ctor(&origin.point, gp_DZ());
        Shape::Cylinder(BRepPrimAPI_MakeCylinder_ctor(&axis, radius, height))
    }

    pub fn fuse(left: &mut Shape, right: &mut Shape) -> Shape {
        Shape::Fuse(BRepAlgoAPI_Fuse_ctor(left.shape(), right.shape()))
    }

    pub fn cut(left: &mut Shape, right: &mut Shape) -> Shape {
        Shape::Cut(BRepAlgoAPI_Cut_ctor(left.shape(), right.shape()))
    }

    pub fn extrude(wire: &mut Edge, x: f64, y: f64, z: f64) -> Shape {
        let mut face_profile = BRepBuilderAPI_MakeFace_wire(wire.0.pin_mut().Wire(), false);
        let prism_vec = new_vec(x, y, z);
        // We're calling Shape here instead of Face(), hope that's also okay.
        let body =
            BRepPrimAPI_MakePrism_ctor(face_profile.pin_mut().Shape(), &prism_vec, true, true);
        Shape::Prism(body)
    }

    pub fn extrude_rotate(wire: &mut Edge, axis: Axis, degrees: f64) -> Shape {
        let mut face_profile = BRepBuilderAPI_MakeFace_wire(wire.0.pin_mut().Wire(), false);

        let radians = degrees * (std::f64::consts::PI / 180.);
        let gp_axis = match axis {
            Axis::X => gp_OX(),
            Axis::Y => gp_OY(),
            Axis::Z => gp_OZ(),
        };

        let body =
            BRepPrimAPI_MakeRevol_ctor(face_profile.pin_mut().Shape(), gp_axis, radians, true);
        Shape::Revol(body)
    }

    pub fn translate(left: &mut Shape, point: &Point) -> Shape {
        let mut transform = new_transform();
        transform
            .pin_mut()
            .SetTranslation(&Point::new(0., 0., 0.).point, &point.point);

        Shape::Transformed(BRepBuilderAPI_Transform_ctor(
            left.shape(),
            &transform,
            true,
        ))
    }

    pub fn rotate(left: &mut Shape, axis: Axis, degrees: f64) -> Shape {
        let mut transform = new_transform();
        let gp_axis = match axis {
            Axis::X => gp_OX(),
            Axis::Y => gp_OY(),
            Axis::Z => gp_OZ(),
        };
        let radians = degrees * (std::f64::consts::PI / 180.);
        transform.pin_mut().SetRotation(gp_axis, radians);

        Shape::Transformed(BRepBuilderAPI_Transform_ctor(
            left.shape(),
            &transform,
            true,
        ))
    }

    pub fn scale(left: &mut Shape, scale: f64) -> Shape {
        let mut transform = new_transform();
        transform
            .pin_mut()
            .SetScale(&Point::new(0., 0., 0.).point, scale);

        Shape::Transformed(BRepBuilderAPI_Transform_ctor(
            left.shape(),
            &transform,
            true,
        ))
    }

    pub fn mirror(left: &mut Shape, axis: Axis) -> Shape {
        let mut transform = new_transform();
        let gp_axis = match axis {
            Axis::X => gp_OX(),
            Axis::Y => gp_OY(),
            Axis::Z => gp_OZ(),
        };
        transform.pin_mut().set_mirror_axis(gp_axis);

        Shape::Transformed(BRepBuilderAPI_Transform_ctor(
            left.shape(),
            &transform,
            true,
        ))
    }

    pub fn fillet(target: &mut Shape, thickness: f64) -> Shape {
        let mut fillet = BRepFilletAPI_MakeFillet_ctor(target.shape());

        let mut edge_explorer = TopExp_Explorer_ctor(target.shape(), TopAbs_ShapeEnum::TopAbs_EDGE);
        while edge_explorer.More() {
            let edge = TopoDS_cast_to_edge(edge_explorer.Current());
            fillet.pin_mut().add_edge(thickness, edge);
            edge_explorer.pin_mut().Next();
        }

        Shape::Fillet(fillet)
    }

    pub fn chamfer(target: &mut Shape, thickness: f64) -> Shape {
        let mut chamfer = BRepFilletAPI_MakeChamfer_ctor(target.shape());

        let mut edge_explorer = TopExp_Explorer_ctor(target.shape(), TopAbs_ShapeEnum::TopAbs_EDGE);
        while edge_explorer.More() {
            let edge = TopoDS_cast_to_edge(edge_explorer.Current());
            chamfer.pin_mut().add_edge(thickness, edge);
            edge_explorer.pin_mut().Next();
        }

        Shape::Chamfer(chamfer)
    }

    pub fn mesh(&mut self) -> Result<IndexedMesh, std::io::Error> {
        let dir = env::temp_dir();
        let file = dir.join("a.stl");

        self.write_stl(file.to_str().unwrap())?;
        let mut file = File::open(file)?;

        let stl = stl_io::read_stl(&mut file).unwrap();
        Ok(stl)
    }

    fn shape(&mut self) -> &TopoDS_Shape {
        match self {
            Shape::Box(b) => b.pin_mut().Shape(),
            Shape::Cylinder(c) => c.pin_mut().Shape(),
            Shape::Fuse(f) => f.pin_mut().Shape(),
            Shape::Cut(f) => f.pin_mut().Shape(),
            Shape::Fillet(f) => f.pin_mut().Shape(),
            Shape::Chamfer(f) => f.pin_mut().Shape(),
            Shape::Transformed(f) => f.pin_mut().Shape(),
            Shape::Prism(p) => p.pin_mut().Shape(),
            Shape::Revol(p) => p.pin_mut().Shape(),
        }
    }

    fn write_stl(&mut self, stl_path: &str) -> Result<(), std::io::Error> {
        let mut writer = StlAPI_Writer_ctor();
        let shape = self.shape();
        let triangulation = BRepMesh_IncrementalMesh_ctor(shape, 0.01);
        let res = write_stl(
            writer.pin_mut(),
            triangulation.Shape(),
            stl_path.to_string(),
        );

        match res {
            true => Ok(()),
            false => Err(std::io::Error::from(ErrorKind::Other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_write_box_stl() {
        let mut shape = Shape::cube(1., 10., 1.);
        shape.write_stl("./demo.stl").unwrap();
    }

    #[test]
    fn it_can_mesh_box_stl() {
        let mut shape = Shape::cube(1., 10., 1.);
        println!("{:?}", shape.mesh())
    }

    #[test]
    fn it_can_fillet_box_stl() {
        let mut b = Shape::cube(10., 10., 10.);
        let mut shape = Shape::fillet(&mut b, 0.5);
        shape.write_stl("./demo.stl").unwrap();
    }

    #[test]
    fn it_can_chamfer_box_stl() {
        let mut b = Shape::cube(10., 10., 10.);
        let mut shape = Shape::chamfer(&mut b, 0.5);
        shape.write_stl("./demo.stl").unwrap();
    }

    #[test]
    fn it_can_write_cylinder_stl() {
        let mut shape = Shape::cylinder(10., 100.);
        shape.write_stl("./demo.stl").unwrap();
    }

    #[test]
    fn it_can_write_translated_stl() {
        let mut b = Shape::cube(10., 10., 10.);
        let mut shape = Shape::translate(&mut b, &Point::new(10., 0., 0.));
        shape.write_stl("./demo.stl").unwrap();
    }

    #[test]
    fn it_can_write_rotated_stl() {
        let mut b = Shape::cube(10., 10., 10.);
        let mut shape = Shape::rotate(&mut b, Axis::X, 45.);
        shape.write_stl("./demo.stl").unwrap();
    }

    #[test]
    fn it_can_write_scaled_stl() {
        let mut b = Shape::cube(1., 1., 1.);
        let mut shape = Shape::scale(&mut b, 10.);
        shape.write_stl("./demo.stl").unwrap();
    }

    #[test]
    fn it_can_write_mirrored_stl() {
        let mut b = Shape::cube(1., 1., 1.);
        let mut shape = Shape::mirror(&mut b, Axis::X);
        shape.write_stl("./demo.stl").unwrap();
    }

    #[test]
    fn it_can_write_fuse_stl() {
        let mut b = Shape::cube(15., 15., 1.);
        let mut c = Shape::cylinder(10., 100.);
        let mut shape = Shape::fuse(&mut b, &mut c);
        shape.write_stl("./demo.stl").unwrap();
    }

    #[test]
    fn it_can_write_cut_stl() {
        let mut b = Shape::cube(15., 15., 1.);
        let mut c = Shape::cylinder(10., 100.);
        let mut shape = Shape::cut(&mut b, &mut c);
        shape.write_stl("./demo.stl").unwrap();
    }
}
