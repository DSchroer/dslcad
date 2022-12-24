use crate::command::Builder;
use crate::Error;
use cxx::UniquePtr;
use opencascade_sys::ffi::{
    TopAbs_ShapeEnum, TopExp_Explorer, TopExp_Explorer_ctor, TopoDS_Edge, TopoDS_Shape,
    TopoDS_cast_to_edge,
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
    pub fn new(shape: &mut impl Builder<TopoDS_Shape>) -> Result<Self, Error> {
        Ok(Explorer {
            explorer: TopExp_Explorer_ctor(shape.try_build()?, Self::shape()),
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
