use crate::command::Builder;
use crate::explorer::UniqueExplorer;
use crate::shapes::DsShape;
use crate::{Error, Mesh, Point, Wire};
use cxx::UniquePtr;
use log::debug;
use opencascade_sys::ffi::{
    gp_Ax2_ctor, gp_DZ, gp_OX, gp_OY, gp_OZ, new_list_of_shape, new_vec, transfer_shape,
    write_step, BRepAlgoAPI_Common, BRepAlgoAPI_Cut, BRepAlgoAPI_Fuse, BRepAlgoAPI_Section,
    BRepBuilderAPI_GTransform, BRepBuilderAPI_MakeFace, BRepBuilderAPI_MakeFace_wire,
    BRepBuilderAPI_Transform, BRepGProp_VolumeProperties, BRepMesh_IncrementalMesh_ctor,
    BRepOffsetAPI_MakeThickSolid_ctor, BRepOffsetAPI_ThruSections, BRepOffsetAPI_ThruSections_ctor,
    BRepPrimAPI_MakeBox, BRepPrimAPI_MakeBox_ctor, BRepPrimAPI_MakeCone,
    BRepPrimAPI_MakeCone_ctor, BRepPrimAPI_MakeCylinder, BRepPrimAPI_MakeCylinder_ctor,
    BRepPrimAPI_MakePrism, BRepPrimAPI_MakePrism_ctor, BRepPrimAPI_MakeRevol,
    BRepPrimAPI_MakeRevol_ctor, BRepPrimAPI_MakeSphere, BRepPrimAPI_MakeSphere_ctor,
    BRepPrimAPI_MakeTorus, BRepPrimAPI_MakeTorus_ctor, BRep_Tool_Pnt, BRep_Tool_Triangulation,
    GProp_GProps_CentreOfMass, GProp_GProps_ctor, HandlePoly_Triangulation_Get,
    IFSelect_ReturnStatus, MakeThickSolidByJoin, Poly_Triangulation_Node, STEPControl_Writer_ctor,
    ShapeUpgrade_UnifySameDomain_ctor, TopAbs_Orientation, TopAbs_ShapeEnum, TopExp_Explorer_ctor,
    TopLoc_Location_ctor, TopoDS_Edge, TopoDS_Shape, TopoDS_Shape_to_owned, TopoDS_Vertex,
    TopoDS_cast_to_face,
};
use std::f64::consts::PI;
use std::os::raw::c_void;
use std::path::Path;

extern "C" {
    fn dslcad_sweep_shape(profile: *const c_void, path: *const c_void) -> *mut c_void;
    fn dslcad_fillet(
        shape: *const c_void,
        radius: f64,
        positions: *const f64,
        radii: *const f64,
        count: i32,
        axis_mask: i32,
    ) -> *mut c_void;
    fn dslcad_chamfer(shape: *const c_void, distance: f64, axis_mask: i32) -> *mut c_void;
}

/// Pack the selected axes into the bit mask the C++ helpers expect.
fn axis_mask(axes: &[Axis]) -> i32 {
    axes.iter().fold(0, |mask, axis| {
        mask | match axis {
            Axis::X => 1,
            Axis::Y => 2,
            Axis::Z => 4,
        }
    })
}

pub struct Shape {
    pub(crate) shape: UniquePtr<TopoDS_Shape>,
}

impl AsRef<TopoDS_Shape> for Shape {
    fn as_ref(&self) -> &TopoDS_Shape {
        &self.shape
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

    /// A cone or truncated cone with `radius1` at the base and `radius2` at the
    /// top. The base sits on the xy plane, at the same place a cylinder of the
    /// same radius would sit.
    pub fn cone(radius1: f64, radius2: f64, height: f64) -> Result<Self, Error> {
        let base = radius1.max(radius2);
        let origin = Point::new(base, base, 0.);
        let axis = gp_Ax2_ctor(&origin.point, gp_DZ());
        let mut cone = BRepPrimAPI_MakeCone_ctor(&axis, radius1, radius2, height, 2. * PI);
        Ok(Builder::try_build(&mut cone)?.into())
    }

    /// A torus around the z axis. `radius` is the distance from the center to
    /// the middle of the tube and `tube` is the radius of the tube. The torus
    /// sits on the xy plane, spanning the same positive quadrant as a sphere.
    pub fn torus(radius: f64, tube: f64) -> Result<Self, Error> {
        let origin = Point::new(radius + tube, radius + tube, tube);
        let axis = gp_Ax2_ctor(&origin.point, gp_DZ());
        let mut torus = BRepPrimAPI_MakeTorus_ctor(&axis, radius, tube, 0., 2. * PI, 2. * PI);
        Ok(Builder::try_build(&mut torus)?.into())
    }

    pub fn extrude(wire: &Wire, x: f64, y: f64, z: f64) -> Result<Self, Error> {
        Self::from_contours(wire, |contour| Self::extrude_contour(contour, x, y, z))
    }

    pub fn extrude_rotate(wire: &Wire, axis: Axis, degrees: f64) -> Result<Self, Error> {
        Self::from_contours(wire, |contour| {
            Self::revolve_contour(contour, axis, degrees)
        })
    }

    /// Loft a solid through a list of section wires.
    pub fn loft(sections: &[Wire]) -> Result<Self, Error> {
        if sections.len() < 2 {
            return Err("a loft needs at least two sections".into());
        }

        let mut loft = BRepOffsetAPI_ThruSections_ctor(true);
        for section in sections {
            if section.is_compound() {
                return Err("loft sections must each be a single contour".into());
            }
            loft.pin_mut().AddWire(section.wire());
        }
        loft.pin_mut().CheckCompatibility(true);

        Ok(Builder::try_build(&mut loft)?.into())
    }

    /// Sweep a profile wire along a path wire.
    pub fn sweep(profile: &Wire, path: &Wire) -> Result<Self, Error> {
        if profile.is_compound() {
            return Err("sweep profiles must be a single contour".into());
        }
        if path.is_compound() {
            return Err("sweep paths must be a single wire".into());
        }

        let mut face_builder = BRepBuilderAPI_MakeFace_wire(profile.wire(), false);
        let face = Builder::try_build(&mut face_builder)?;

        let raw = unsafe {
            dslcad_sweep_shape(
                face as *const TopoDS_Shape as *const c_void,
                path.as_ref() as *const TopoDS_Shape as *const c_void,
            )
        };
        if raw.is_null() {
            return Err("could not sweep the profile".into());
        }

        Ok(Shape {
            shape: unsafe { UniquePtr::from_raw(raw as *mut TopoDS_Shape) },
        })
    }

    /// Hollow a solid, leaving walls of the given thickness.
    pub fn shell(shape: &Shape, thickness: f64) -> Result<Self, Error> {
        if thickness <= 0.0 {
            return Err("shell thickness must be positive".into());
        }

        // Offsetting the solid inward gives the cavity; cutting it out leaves
        // the walls.
        let closing_faces = new_list_of_shape();
        let mut offset = BRepOffsetAPI_MakeThickSolid_ctor();
        MakeThickSolidByJoin(
            offset.pin_mut(),
            shape.shape(),
            &closing_faces,
            -thickness,
            1e-3,
        );

        if !offset.IsDone() {
            return Err("could not shell the shape".into());
        }

        let inner = Shape::from(offset.pin_mut().Shape());
        if inner.shape.ShapeType() != TopAbs_ShapeEnum::TopAbs_SOLID {
            return Err("could not shell the shape".into());
        }

        shape.cut(&inner)
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

    pub fn fillet(target: &Shape, radius: f64) -> Result<Self, Error> {
        Self::fillet_edges(target, radius, &[])
    }

    /// Fillet the edges that run along one of the given axes. Without axes
    /// every edge is filleted.
    pub fn fillet_edges(target: &Shape, radius: f64, axes: &[Axis]) -> Result<Self, Error> {
        if radius <= 0.0 {
            return Err("fillet radius must be positive".into());
        }

        let raw = unsafe {
            dslcad_fillet(
                target.shape() as *const TopoDS_Shape as *const c_void,
                radius,
                std::ptr::null(),
                std::ptr::null(),
                0,
                axis_mask(axes),
            )
        };

        Self::from_raw(raw, "could not fillet the shape")
    }

    /// Fillet the edges with a radius that varies along each edge. Every entry
    /// is a `(position, radius)` pair, where the position runs from 0 at the
    /// start of the edge to 1 at the end.
    pub fn fillet_variable(
        target: &Shape,
        radii: &[(f64, f64)],
        axes: &[Axis],
    ) -> Result<Self, Error> {
        if radii.is_empty() {
            return Err("a variable fillet needs at least one radius".into());
        }

        let positions: Vec<f64> = radii.iter().map(|(position, _)| *position).collect();
        let values: Vec<f64> = radii.iter().map(|(_, radius)| *radius).collect();

        let raw = unsafe {
            dslcad_fillet(
                target.shape() as *const TopoDS_Shape as *const c_void,
                0.0,
                positions.as_ptr(),
                values.as_ptr(),
                radii.len() as i32,
                axis_mask(axes),
            )
        };

        Self::from_raw(raw, "could not fillet the shape")
    }

    pub fn chamfer(target: &Shape, distance: f64) -> Result<Self, Error> {
        Self::chamfer_edges(target, distance, &[])
    }

    /// Chamfer the edges that run along one of the given axes. Without axes
    /// every edge is chamfered.
    pub fn chamfer_edges(target: &Shape, distance: f64, axes: &[Axis]) -> Result<Self, Error> {
        if distance <= 0.0 {
            return Err("chamfer distance must be positive".into());
        }

        let raw = unsafe {
            dslcad_chamfer(
                target.shape() as *const TopoDS_Shape as *const c_void,
                distance,
                axis_mask(axes),
            )
        };

        Self::from_raw(raw, "could not chamfer the shape")
    }

    fn from_raw(raw: *mut c_void, message: &'static str) -> Result<Self, Error> {
        if raw.is_null() {
            return Err(message.into());
        }

        Ok(Shape {
            shape: unsafe { UniquePtr::from_raw(raw as *mut TopoDS_Shape) },
        })
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
shape_builder!(BRepPrimAPI_MakeCone);
shape_builder!(BRepPrimAPI_MakeTorus);
shape_builder!(BRepPrimAPI_MakePrism);
shape_builder!(BRepPrimAPI_MakeRevol);
shape_builder!(BRepOffsetAPI_ThruSections);
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

    fn column() -> Shape {
        Shape::cylinder(10., 100.)
            .unwrap()
            .translate(&Point::new(0.3, 0.3, 0.))
            .unwrap()
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
    fn it_fillets_selected_edges() {
        let cube = Shape::cube(10., 10., 10.).unwrap();
        let filleted = Shape::fillet_edges(&cube, 1., &[Axis::Z]).unwrap();

        // Each of the four vertical edges loses (1 - pi/4) * 1 * 10.
        let expected = 1000. - 4. * 10. * (1. - std::f64::consts::FRAC_PI_4);
        assert!((filleted.volume() - expected).abs() < 0.05);
    }

    #[test]
    fn it_chamfers_selected_edges() {
        let cube = Shape::cube(10., 10., 10.).unwrap();
        let chamfered = Shape::chamfer_edges(&cube, 1., &[Axis::Z]).unwrap();

        // Each of the four vertical edges loses a 1x10 triangle.
        let expected = 1000. - 4. * 0.5 * 10.;
        assert!((chamfered.volume() - expected).abs() < 0.05);
    }

    #[test]
    fn it_fillets_with_a_variable_radius() {
        let cube = Shape::cube(10., 10., 10.).unwrap();
        let filleted = Shape::fillet_variable(&cube, &[(0., 0.5), (1., 1.)], &[Axis::Z]).unwrap();

        // The radius runs from 0.5 to 1 along each vertical edge, each slice
        // losing the square outside the quarter circle: r^2 * (1 - pi / 4).
        let profile = |t: f64| {
            let r = 0.5 + 0.5 * t;
            r * r * (1. - std::f64::consts::FRAC_PI_4)
        };
        let average = (profile(0.) + 4. * profile(0.5) + profile(1.)) / 6.;
        let expected = 1000. - 4. * 10. * average;
        assert!((filleted.volume() - expected).abs() < 0.1);
    }

    #[test]
    fn it_can_write_cylinder_stl() {
        let shape = Shape::cylinder(10., 100.).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_make_a_cone() {
        let cone = Shape::cone(2., 1., 4.).unwrap();

        // Volume of a truncated cone: 1/3 * pi * h * (r1^2 + r1 * r2 + r2^2).
        let expected = PI / 3. * 4. * (4. + 2. + 1.);
        assert!((cone.volume() - expected).abs() < 1e-6);
    }

    #[test]
    fn it_can_make_a_torus() {
        let torus = Shape::torus(2., 0.5).unwrap();

        // Volume of a torus: 2 * pi^2 * R * r^2.
        let expected = 2. * PI * PI * 2. * 0.25;
        assert!((torus.volume() - expected).abs() < 1e-6);
    }

    #[test]
    fn it_can_loft_between_sections() {
        let bottom = square(0., 0., 10.);
        let top = square(4., 4., 2.)
            .translate(&Point::new(0., 0., 10.))
            .unwrap();

        let shape = Shape::loft(&[bottom, top]).unwrap();

        // A linear transition between two centered squares gives
        // h/3 * (A1 + A2 + sqrt(A1 * A2)).
        let expected = 10. / 3. * (100. + 4. + (100. * 4.0f64).sqrt());
        assert!((shape.volume() - expected).abs() < 0.1);
    }

    #[test]
    fn it_can_sweep_a_profile_along_a_path() {
        let profile = square(-1., -1., 2.);

        let mut path = WireFactory::new();
        path.add_edge(&Edge::new_line(&Point::new(0., 0., 0.), &Point::new(0., 0., 10.)).unwrap());
        let path = path.build().unwrap();

        let shape = Shape::sweep(&profile, &path).unwrap();

        assert!((shape.volume() - 40.).abs() < 0.01);
    }

    #[test]
    fn it_can_shell_a_shape() {
        let cube = Shape::cube(10., 10., 10.).unwrap();
        let hollow = Shape::shell(&cube, 1.).unwrap();

        assert!((hollow.volume() - 488.).abs() < 0.1);
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
        let c = column();
        let shape = Shape::fuse(&b, &c).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_write_cut_stl() {
        let b = Shape::cube(15., 15., 1.).unwrap();
        let c = column();
        let shape = Shape::cut(&b, &c).unwrap();
        shape.mesh(0.1).unwrap();
    }

    #[test]
    fn it_can_write_intersect_stl() {
        let b = Shape::cube(15., 15., 1.).unwrap();
        let c = column();
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
