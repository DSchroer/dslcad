use cxx::UniquePtr;
use opencascade_sys::ffi::{
    map_shapes, new_indexed_map_of_shape, TopAbs_ShapeEnum, TopExp_Explorer, TopExp_Explorer_ctor,
    TopTools_IndexedMapOfShape, TopoDS_Edge, TopoDS_Face, TopoDS_Shape, TopoDS_Vertex, TopoDS_Wire,
    TopoDS_cast_to_edge, TopoDS_cast_to_face, TopoDS_cast_to_vertex, TopoDS_cast_to_wire,
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
    pub fn new(shape: impl AsRef<TopoDS_Shape>) -> Self {
        Explorer {
            explorer: TopExp_Explorer_ctor(shape.as_ref(), Self::shape()),
            first: true,
            _phantom: PhantomData,
        }
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

/// An explorer that visits each unique sub-shape once. The plain
/// [`TopExp_Explorer`] visits sub-shapes shared between parents (for example
/// the edges between two faces of a solid) multiple times.
pub(crate) struct UniqueExplorer<T> {
    map: UniquePtr<TopTools_IndexedMapOfShape>,
    index: i32,
    _phantom: PhantomData<T>,
}

impl<T> UniqueExplorer<T>
where
    UniqueExplorer<T>: GeomIterator<T>,
{
    pub fn new(shape: impl AsRef<TopoDS_Shape>) -> Self {
        let mut map = new_indexed_map_of_shape();
        map_shapes(shape.as_ref(), Self::shape(), map.pin_mut());
        UniqueExplorer {
            map,
            index: 0,
            _phantom: PhantomData,
        }
    }

    pub fn next(&mut self) -> Option<&T> {
        self.index += 1;
        if self.index > self.map.Extent() {
            return None;
        }

        Some(self.current())
    }
}

impl GeomIterator<TopoDS_Edge> for UniqueExplorer<TopoDS_Edge> {
    fn shape() -> TopAbs_ShapeEnum {
        TopAbs_ShapeEnum::TopAbs_EDGE
    }
    fn current(&self) -> &TopoDS_Edge {
        TopoDS_cast_to_edge(self.map.FindKey(self.index))
    }
}

impl GeomIterator<TopoDS_Face> for UniqueExplorer<TopoDS_Face> {
    fn shape() -> TopAbs_ShapeEnum {
        TopAbs_ShapeEnum::TopAbs_FACE
    }
    fn current(&self) -> &TopoDS_Face {
        TopoDS_cast_to_face(self.map.FindKey(self.index))
    }
}

impl GeomIterator<TopoDS_Vertex> for UniqueExplorer<TopoDS_Vertex> {
    fn shape() -> TopAbs_ShapeEnum {
        TopAbs_ShapeEnum::TopAbs_VERTEX
    }
    fn current(&self) -> &TopoDS_Vertex {
        TopoDS_cast_to_vertex(self.map.FindKey(self.index))
    }
}

impl GeomIterator<TopoDS_Wire> for UniqueExplorer<TopoDS_Wire> {
    fn shape() -> TopAbs_ShapeEnum {
        TopAbs_ShapeEnum::TopAbs_WIRE
    }
    fn current(&self) -> &TopoDS_Wire {
        TopoDS_cast_to_wire(self.map.FindKey(self.index))
    }
}
