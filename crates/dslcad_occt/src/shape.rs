use crate::command::Builder;
use crate::explorer::UniqueExplorer;
use crate::shapes::DsShape;
use crate::{Error, Mesh, Point, Wire};
use cxx::UniquePtr;
use log::debug;
use opencascade_sys::ffi::{
    gp_Ax2_ctor, gp_DZ, gp_OX, gp_OY, gp_OZ, new_vec, transfer_shape, write_step,
    BRepAlgoAPI_Common, BRepAlgoAPI_Cut, BRepAlgoAPI_Fuse, BRepAlgoAPI_Section,
    BRepBuilderAPI_GTransform, BRepBuilderAPI_MakeFace, BRepBuilderAPI_MakeFace_wire,
    BRepBuilderAPI_Transform, BRepFilletAPI_MakeChamfer, BRepFilletAPI_MakeChamfer_ctor,
    BRepFilletAPI_MakeFillet, BRepFilletAPI_MakeFillet_ctor, BRepGProp_VolumeProperties,
    BRepMesh_IncrementalMesh_ctor, BRepPrimAPI_MakeBox, BRepPrimAPI_MakeBox_ctor,
    BRepPrimAPI_MakeCylinder, BRepPrimAPI_MakeCylinder_ctor, BRepPrimAPI_MakePrism,
    BRepPrimAPI_MakePrism_ctor, BRepPrimAPI_MakeRevol, BRepPrimAPI_MakeRevol_ctor,
    BRepPrimAPI_MakeSphere, BRepPrimAPI_MakeSphere_ctor, BRep_Tool_Pnt, BRep_Tool_Triangulation,
    GProp_GProps_CentreOfMass, GProp_GProps_ctor, HandlePoly_Triangulation_Get,
    IFSelect_ReturnStatus, Poly_Triangulation_Node, STEPControl_Writer_ctor,
    ShapeUpgrade_UnifySameDomain_ctor, TopAbs_Orientation, TopAbs_ShapeEnum, TopExp_Explorer_ctor,
    TopLoc_Location_ctor, TopoDS_Edge, TopoDS_Shape, TopoDS_Shape_to_owned, TopoDS_Vertex,
    TopoDS_cast_to_face,
};
use std::f64::consts::PI;
use std::path::Path;

pub struct Shape {
    pub(crate) shape: UniquePtr<TopoDS_Shape>,
}

impl AsRef<TopoDS_Shape> for Shape {
    fn as_ref(&self) -> &TopoDS_Shape {
        &self.shape
    }
}

#[derive(Clone, Copy)]
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
        Self::from_contours(wire, |contour| Self::extrude_contour(contour, x, y, z))
    }

    pub fn extrude_rotate(wire: &Wire, axis: Axis, degrees: f64) -> Result<Self, Error> {
        Self::from_contours(wire, |contour| {
            Self::revolve_contour(contour, axis, degrees)
        })
    }

    fn from_contours(
        wire: &Wire,
        build: impl Fn(&Wire) -> Result<Self, Error>,
    ) -> Result<Self, Error> {
        if !wire.is_compound() {
            return build(wire);
        }

        let contours = wire.contours();
        let polygons: Vec<Vec<[f64; 3]>> = contours.iter().map(Self::contour_polygon).collect();

        if polygons.iter().any(|polygon| polygon.is_empty()) {
            return Err("could not read contour geometry".into());
        }

        let mut parents = vec![None; contours.len()];
        for (index, polygon) in polygons.iter().enumerate() {
            parents[index] = (0..polygons.len())
                .filter(|other| {
                    *other != index && Self::polygon_contains(&polygons[*other], polygon[0])
                })
                .min_by(|left, right| {
                    Self::polygon_area(&polygons[*left])
                        .total_cmp(&Self::polygon_area(&polygons[*right]))
                });
        }

        let mut levels: Vec<Vec<usize>> = Vec::new();
        for index in 0..contours.len() {
            let mut depth = 0;
            let mut current = parents[index];
            while let Some(parent) = current {
                depth += 1;
                current = parents[parent];
            }

            if levels.len() <= depth {
                levels.resize(depth + 1, Vec::new());
            }
            levels[depth].push(index);
        }

        if levels[0].is_empty() {
            return Err("no outer contours to extrude".into());
        }

        let mut result: Option<Self> = None;
        for (depth, level) in levels.iter().enumerate() {
            if level.is_empty() {
                continue;
            }

            let mut region: Option<Self> = None;
            for index in level {
                let solid = build(&contours[*index])?;
                region = Some(match region {
                    Some(current) => current.fuse(&solid)?,
                    None => solid,
                });
            }

            let region = region.ok_or_else(|| "could not build contour".to_string())?;
            result = Some(match result {
                None => region,
                Some(current) if depth % 2 == 0 => current.fuse(&region)?,
                Some(current) => current.cut(&region)?,
            });
        }

        result.ok_or_else(|| "no contours to extrude".into())
    }

    fn contour_polygon(contour: &Wire) -> Vec<[f64; 3]> {
        contour
            .points(0.01)
            .unwrap_or_default()
            .into_iter()
            .flatten()
            .collect()
    }

    fn polygon_area(polygon: &[[f64; 3]]) -> f64 {
        let mut area = 0.0;
        for index in 0..polygon.len() {
            let previous = if index == 0 {
                polygon.len() - 1
            } else {
                index - 1
            };
            area +=
                polygon[previous][0] * polygon[index][1] - polygon[index][0] * polygon[previous][1];
        }
        area.abs() / 2.0
    }

    fn polygon_contains(polygon: &[[f64; 3]], point: [f64; 3]) -> bool {
        let mut inside = false;
        let mut previous = polygon.len() - 1;

        for current in 0..polygon.len() {
            let (x1, y1) = (polygon[current][0], polygon[current][1]);
            let (x2, y2) = (polygon[previous][0], polygon[previous][1]);

            if (y1 > point[1]) != (y2 > point[1])
                && point[0] < (x2 - x1) * (point[1] - y1) / (y2 - y1) + x1
            {
                inside = !inside;
            }

            previous = current;
        }

        inside
    }

    fn extrude_contour(wire: &Wire, x: f64, y: f64, z: f64) -> Result<Self, Error> {
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

    fn revolve_contour(wire: &Wire, axis: Axis, degrees: f64) -> Result<Self, Error> {
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

        let mut edge_explorer: UniqueExplorer<TopoDS_Edge> = UniqueExplorer::new(target);
        while let Some(edge) = edge_explorer.next() {
            fillet.pin_mut().add_edge(thickness, edge);
        }

        Ok(Builder::try_build(&mut fillet)?.into())
    }

    pub fn chamfer(target: &Shape, thickness: f64) -> Result<Self, Error> {
        let mut chamfer = BRepFilletAPI_MakeChamfer_ctor(&target.shape);

        let mut edge_explorer: UniqueExplorer<TopoDS_Edge> = UniqueExplorer::new(target);
        while let Some(edge) = edge_explorer.next() {
            chamfer.pin_mut().add_edge(thickness, edge);
        }

        Ok(Builder::try_build(&mut chamfer)?.into())
    }

    /// Merge faces and edges that share the same underlying geometry to
    /// reduce the complexity of the shape.
    pub fn simplify(&self) -> Result<Self, Error> {
        let mut upgrade = ShapeUpgrade_UnifySameDomain_ctor(&self.shape, true, true, true);
        upgrade.pin_mut().AllowInternalEdges(false);
        upgrade.pin_mut().Build();
        Ok(upgrade.Shape().into())
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

    pub fn write_step(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let mut writer = STEPControl_Writer_ctor();

        let status = transfer_shape(writer.pin_mut(), &self.shape);
        if status != IFSelect_ReturnStatus::IFSelect_RetDone {
            return Err("failed to transfer shape to STEP writer".into());
        }

        let status = write_step(
            writer.pin_mut(),
            path.as_ref().to_string_lossy().to_string(),
        );
        if status != IFSelect_ReturnStatus::IFSelect_RetDone {
            return Err("failed to write STEP file".into());
        }

        Ok(())
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
            if let Ok(triangulation) = HandlePoly_Triangulation_Get(&triangulation_handle) {
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
        let mut edge_explorer: UniqueExplorer<TopoDS_Edge> = UniqueExplorer::new(self);
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

        let mut vertex_explorer: UniqueExplorer<TopoDS_Vertex> = UniqueExplorer::new(self);

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
shape_builder!(BRepBuilderAPI_GTransform);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, WireFactory};

    fn square(x: f64, y: f64, size: f64) -> Wire {
        let mut wire = WireFactory::new();

        wire.add_edge(
            &Edge::new_line(&Point::new(x, y, 0.), &Point::new(x + size, y, 0.)).unwrap(),
        );
        wire.add_edge(
            &Edge::new_line(
                &Point::new(x + size, y, 0.),
                &Point::new(x + size, y + size, 0.),
            )
            .unwrap(),
        );
        wire.add_edge(
            &Edge::new_line(
                &Point::new(x + size, y + size, 0.),
                &Point::new(x, y + size, 0.),
            )
            .unwrap(),
        );
        wire.add_edge(
            &Edge::new_line(&Point::new(x, y + size, 0.), &Point::new(x, y, 0.)).unwrap(),
        );

        wire.build().unwrap()
    }

    #[test]
    fn it_only_returns_unique_edges_and_vertices() {
        let shape = Shape::cube(1., 1., 1.).unwrap();

        assert_eq!(8, shape.points().unwrap().len());
        assert_eq!(12, shape.lines(0.1).unwrap().len());
    }

    #[test]
    fn it_can_extrude_contours_with_holes() {
        let outer = square(0., 0., 10.);
        let inner = square(3., 3., 4.);
        let compound = Wire::compound(&[outer, inner]).unwrap();

        let shape = Shape::extrude(&compound, 0., 0., 1.).unwrap();

        assert!((shape.volume() - 84.).abs() < 0.01);
    }

    #[test]
    fn it_can_extrude_islands_inside_holes() {
        let outer = square(0., 0., 10.);
        let hole = square(1., 1., 8.);
        let island = square(3., 3., 4.);
        let compound = Wire::compound(&[outer, hole, island]).unwrap();

        let shape = Shape::extrude(&compound, 0., 0., 1.).unwrap();

        assert!((shape.volume() - 52.).abs() < 0.01);
    }

    #[test]
    fn it_can_extrude_separate_contours() {
        let left = square(0., 0., 4.);
        let right = square(10., 0., 4.);
        let compound = Wire::compound(&[left, right]).unwrap();

        let shape = Shape::extrude(&compound, 0., 0., 2.).unwrap();

        assert!((shape.volume() - 64.).abs() < 0.01);
    }

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
    fn it_can_simplify_a_shape() {
        let b = Shape::cube(10., 10., 10.).unwrap();
        let c = Shape::cube(5., 5., 5.).unwrap();
        let shape = b.fuse(&c).unwrap();

        let simplified = shape.simplify().unwrap();

        assert!((simplified.volume() - shape.volume()).abs() < 0.01);
        simplified.mesh(0.1).unwrap();
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

    #[test]
    fn it_can_write_step() {
        let shape = Shape::cube(10., 10., 10.).unwrap();
        let path = std::env::temp_dir().join("dslcad_test.step");

        shape.write_step(&path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("ISO-10303-21"));
        assert!(contents.contains("MANIFOLD_SOLID_BREP"));

        let _ = std::fs::remove_file(&path);
    }
}
