use std::io::ErrorKind;
use path_absolutize::*;
use std::path::{PathBuf};
use crate::Point;
use cxx::{UniquePtr};
use opencascade_sys::ffi::{BRepAlgoAPI_Cut, BRepAlgoAPI_Cut_ctor, BRepFilletAPI_MakeChamfer, TopoDS_Shape, BRepPrimAPI_MakeBox, BRepPrimAPI_MakeCylinder, BRepAlgoAPI_Fuse, BRepFilletAPI_MakeFillet, BRepMesh_IncrementalMesh_ctor, BRepPrimAPI_MakeBox_ctor, StlAPI_Writer_ctor, write_stl, BRepPrimAPI_MakeCylinder_ctor, BRepAlgoAPI_Fuse_ctor, BRepFilletAPI_MakeFillet_ctor, gp_Ax2_ctor, gp_DZ, TopExp_Explorer_ctor, TopAbs_ShapeEnum, TopoDS_cast_to_edge, BRepFilletAPI_MakeChamfer_ctor};

pub enum Shape {
    Box(Box<UniquePtr<BRepPrimAPI_MakeBox>>),
    Cylinder(Box<UniquePtr<BRepPrimAPI_MakeCylinder>>),
    Fuse(Box<UniquePtr<BRepAlgoAPI_Fuse>>),
    Cut(Box<UniquePtr<BRepAlgoAPI_Cut>>),
    Fillet(Box<UniquePtr<BRepFilletAPI_MakeFillet>>),
    Chamfer(Box<UniquePtr<BRepFilletAPI_MakeChamfer>>)
}

impl Shape {
    pub fn cube(dx: f64, dy: f64, dz: f64) -> Self {
        // SAFETY: cross C++ boundary
        unsafe {
            let origin = Point::new(0., 0., 0.);
            Shape::Box(Box::new(BRepPrimAPI_MakeBox_ctor(&origin.point, dx, dy, dz)))
        }
    }

    pub fn cylinder(radius: f64, height: f64) -> Self {
        // SAFETY: cross C++ boundary
        unsafe {
            let origin = Point::new(0., 0., 0.);
            let axis = gp_Ax2_ctor(&origin.point, gp_DZ());
            Shape::Cylinder(Box::new(BRepPrimAPI_MakeCylinder_ctor(&axis, radius, height)))
        }
    }

    pub fn fuse(left: &mut Shape, right: &mut Shape) -> Shape {
        // SAFETY: cross C++ boundary
        unsafe {
            Shape::Fuse(Box::new(BRepAlgoAPI_Fuse_ctor(left.shape(), right.shape())))
        }
    }

    pub fn cut(left: &mut Shape, right: &mut Shape) -> Shape {
        // SAFETY: cross C++ boundary
        unsafe {
            Shape::Cut(Box::new(BRepAlgoAPI_Cut_ctor(left.shape(), right.shape())))
        }
    }

    pub fn fillet(target: &mut Shape, thickness: f64) -> Shape {
        // SAFETY: cross C++ boundary
        unsafe {
            let mut fillet = BRepFilletAPI_MakeFillet_ctor(target.shape());

            let mut edge_explorer = TopExp_Explorer_ctor(&target.shape(), TopAbs_ShapeEnum::TopAbs_EDGE);
            while edge_explorer.More() {
                let edge = TopoDS_cast_to_edge(edge_explorer.Current());
                fillet.pin_mut().add_edge(thickness, edge);
                edge_explorer.pin_mut().Next();
            }

            Shape::Fillet(Box::new(fillet))
        }
    }

    pub fn chamfer(target: &mut Shape, thickness: f64) -> Shape {
        // SAFETY: cross C++ boundary
        unsafe {
            let mut chamfer = BRepFilletAPI_MakeChamfer_ctor(target.shape());

            let mut edge_explorer = TopExp_Explorer_ctor(&target.shape(), TopAbs_ShapeEnum::TopAbs_EDGE);
            while edge_explorer.More() {
                let edge = TopoDS_cast_to_edge(edge_explorer.Current());
                chamfer.pin_mut().add_edge(thickness, edge);
                edge_explorer.pin_mut().Next();
            }

            Shape::Chamfer(Box::new(chamfer))
        }
    }

    fn shape(&mut self) -> &TopoDS_Shape {
        match self {
            Shape::Box(b) => b.pin_mut().Shape(),
            Shape::Cylinder(c) => c.pin_mut().Shape(),
            Shape::Fuse(f) => f.pin_mut().Shape(),
            Shape::Cut(f) => f.pin_mut().Shape(),
            Shape::Fillet(f) => f.pin_mut().Shape(),
            Shape::Chamfer(f) => f.pin_mut().Shape()
        }
    }

    pub fn write_stl(&mut self, stl_path: &str) -> Result<(), std::io::Error> {
        let buf = PathBuf::from(stl_path);
        let stl_path = buf.absolutize()?.to_str().unwrap().to_string();

        // SAFETY: cross C++ boundary
        let _res = unsafe {
            let mut writer = StlAPI_Writer_ctor();
            let shape = self.shape();
            let triangulation = BRepMesh_IncrementalMesh_ctor(shape, 0.01);
            write_stl(writer.pin_mut(), triangulation.Shape(), stl_path)
        };
        match _res {
            true => Ok(()),
            false => Err(std::io::Error::from(ErrorKind::Other))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;

    #[test]
    fn it_can_write_box_stl() {
        let mut shape = Shape::cube(1., 10., 1.);
        shape.write_stl("./demo.stl").unwrap();
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
