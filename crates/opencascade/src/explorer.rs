use crate::Error;
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    TopAbs_ShapeEnum, TopExp_Explorer, TopExp_Explorer_ctor, TopoDS_Edge, TopoDS_Face,
    TopoDS_Shape, TopoDS_Vertex, TopoDS_Wire, TopoDS_cast_to_edge, TopoDS_cast_to_face,
    TopoDS_cast_to_vertex, TopoDS_cast_to_wire,
};
use std::marker::PhantomData;

pub(crate) struct Explorer<T> {
    explorer: UniquePtr<TopExp_Explorer>,
    first: bool,
    _phantom: PhantomData<T>,
}

pub(crate) trait GeomIterator<T> {
    fn shape() -> TopAbs_ShapeEnum;
    fn current(&self) -> &T;
}

impl<T> Explorer<T>
where
    Explorer<T>: GeomIterator<T>,
{
    pub fn new(shape: &TopoDS_Shape) -> Result<Self, Error> {
        Ok(Explorer {
            explorer: TopExp_Explorer_ctor(shape, Self::shape()),
            first: true,
            _phantom: PhantomData::default(),
        })
    }

    pub fn next(&mut self) -> Option<&T> {
        if self.first {
            self.first = false;
        } else {
            self.explorer.pin_mut().Next();
        }

        if self.explorer.More() {
            Some(self.current())
        } else {
            None
        }
    }
}

impl GeomIterator<TopoDS_Edge> for Explorer<TopoDS_Edge> {
    fn shape() -> TopAbs_ShapeEnum {
        TopAbs_ShapeEnum::TopAbs_EDGE
    }
    fn current(&self) -> &TopoDS_Edge {
        TopoDS_cast_to_edge(self.explorer.Current())
    }
}

impl GeomIterator<TopoDS_Shape> for Explorer<TopoDS_Shape> {
    fn shape() -> TopAbs_ShapeEnum {
        TopAbs_ShapeEnum::TopAbs_SHAPE
    }
    fn current(&self) -> &TopoDS_Shape {
        self.explorer.Current()
    }
}

impl GeomIterator<TopoDS_Face> for Explorer<TopoDS_Face> {
    fn shape() -> TopAbs_ShapeEnum {
        TopAbs_ShapeEnum::TopAbs_FACE
    }
    fn current(&self) -> &TopoDS_Face {
        TopoDS_cast_to_face(self.explorer.Current())
    }
}

impl GeomIterator<TopoDS_Vertex> for Explorer<TopoDS_Vertex> {
    fn shape() -> TopAbs_ShapeEnum {
        TopAbs_ShapeEnum::TopAbs_VERTEX
    }
    fn current(&self) -> &TopoDS_Vertex {
        TopoDS_cast_to_vertex(self.explorer.Current())
    }
}

impl GeomIterator<TopoDS_Wire> for Explorer<TopoDS_Wire> {
    fn shape() -> TopAbs_ShapeEnum {
        TopAbs_ShapeEnum::TopAbs_WIRE
    }
    fn current(&self) -> &TopoDS_Wire {
        TopoDS_cast_to_wire(self.explorer.Current())
    }
}
