use crate::command::Builder;
use crate::compound::Compound;
use crate::{Axis, Error, Point, Wire};
use opencascade_sys::ffi::{
    gp_OX, gp_OY, gp_OZ, new_gp_GTrsf, new_transform, BRepAlgoAPI_Common_ctor,
    BRepAlgoAPI_Cut_ctor, BRepAlgoAPI_Fuse_ctor, BRepAlgoAPI_Section_ctor,
    BRepBuilderAPI_GTransform_ctor, BRepBuilderAPI_MakeFace_wire, BRepBuilderAPI_Transform_ctor,
    TopoDS_Shape, TopoDS_cast_to_compound,
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

    fn transform(&self, values: &[f64]) -> Result<Self, Error> {
        assert_eq!(values.len(), 3 * 4, "transform must be 3 x 4 matrix");

        let mut transform = new_gp_GTrsf();
        let mut i = 0;
        for row in 1..=3 {
            for col in 1..=4 {
                transform.pin_mut().SetValue(row, col, values[i]);
                i += 1;
            }
        }

        Ok(Builder::try_build(&mut BRepBuilderAPI_GTransform_ctor(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Shape;

    #[test]
    fn it_can_raw_transform_shapes() {
        let identity = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];

        let cube = Shape::cube(1., 1., 1.).unwrap();
        let shape = cube.transform(&identity).unwrap();

        assert_eq!(
            shape.points().unwrap(),
            Shape::cube(1., 1., 1.).unwrap().points().unwrap()
        );
        dbg!(shape.points().unwrap());
    }
}
