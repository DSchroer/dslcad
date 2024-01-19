use crate::command::Builder;
use crate::compound::Compound;
use crate::{Axis, Error, Point, Wire};
use opencascade_sys::ffi::{
    gp_OX, gp_OY, gp_OZ, new_transform, BRepAlgoAPI_Common_ctor, BRepAlgoAPI_Cut_ctor,
    BRepAlgoAPI_Fuse_ctor, BRepAlgoAPI_Section_ctor, BRepBuilderAPI_MakeFace_wire,
    BRepBuilderAPI_Transform_ctor, TopoDS_Shape, TopoDS_cast_to_compound,
};

pub trait DsShape: for<'a> From<&'a TopoDS_Shape> {
    fn shape(&self) -> &TopoDS_Shape;

    fn translate(&self, point: &Point) -> Result<Self, Error> {
        let mut transform = new_transform();
        transform
            .pin_mut()
            .SetTranslation(&Point::new(0., 0., 0.).point, &point.point);

        let mut transform_builder = BRepBuilderAPI_Transform_ctor(self.shape(), &transform, true);
        Ok(Builder::try_build(&mut transform_builder)?.into())
    }

    fn rotate(&self, axis: Axis, degrees: f64) -> Result<Self, Error> {
        let mut transform = new_transform();
        let gp_axis = match axis {
            Axis::X => gp_OX(),
            Axis::Y => gp_OY(),
            Axis::Z => gp_OZ(),
        };
        let radians = degrees * (std::f64::consts::PI / 180.);
        transform.pin_mut().SetRotation(gp_axis, radians);

        Ok(Builder::try_build(&mut BRepBuilderAPI_Transform_ctor(
            self.shape(),
            &transform,
            true,
        ))?
        .into())
    }

    fn scale(&self, scale: f64) -> Result<Self, Error> {
        let mut transform = new_transform();
        transform
            .pin_mut()
            .SetScale(&Point::new(0., 0., 0.).point, scale);

        Ok(Builder::try_build(&mut BRepBuilderAPI_Transform_ctor(
            self.shape(),
            &transform,
            true,
        ))?
        .into())
    }

    fn mirror(&self, axis: Axis) -> Result<Self, Error> {
        let mut transform = new_transform();
        let gp_axis = match axis {
            Axis::X => gp_OX(),
            Axis::Y => gp_OY(),
            Axis::Z => gp_OZ(),
        };
        transform.pin_mut().set_mirror_axis(gp_axis);

        Ok(Builder::try_build(&mut BRepBuilderAPI_Transform_ctor(
            self.shape(),
            &transform,
            true,
        ))?
        .into())
    }

    fn fuse(&self, right: &Self) -> Result<Self, Error> {
        Ok(Builder::try_build(&mut BRepAlgoAPI_Fuse_ctor(self.shape(), right.shape()))?.into())
    }

    fn cut(&self, right: &Self) -> Result<Self, Error> {
        Ok(Builder::try_build(&mut BRepAlgoAPI_Cut_ctor(self.shape(), right.shape()))?.into())
    }

    fn intersect(&self, right: &Self) -> Result<Self, Error> {
        Ok(Builder::try_build(&mut BRepAlgoAPI_Common_ctor(self.shape(), right.shape()))?.into())
    }

    fn section_2d(&self, right: &Wire) -> Result<Wire, Error> {
        let mut face_builder = BRepBuilderAPI_MakeFace_wire(right.wire(), false);
        let face = Builder::try_build(&mut face_builder)?;
        let binding = &mut BRepAlgoAPI_Section_ctor(self.shape(), face);
        let compound_shape: Compound = TopoDS_cast_to_compound(Builder::try_build(binding)?).into();
        compound_shape.try_into()
    }

    fn section(&self, right: &Self) -> Result<Wire, Error> {
        let binding = &mut BRepAlgoAPI_Section_ctor(self.shape(), right.shape());
        let compound_shape: Compound = TopoDS_cast_to_compound(Builder::try_build(binding)?).into();
        compound_shape.try_into()
    }
}
