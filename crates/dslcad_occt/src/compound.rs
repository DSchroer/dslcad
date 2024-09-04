use crate::command::Builder;
use crate::explorer::Explorer;
use crate::{Edge, Error, Point, Wire};
use cxx::UniquePtr;
use log::debug;
use opencascade_sys::ffi::{
    cast_compound_to_shape, BRepBuilderAPI_MakeWire_ctor, BRepGProp_LinearProperties,
    GProp_GProps_CentreOfMass, GProp_GProps_ctor, TopoDS_Compound, TopoDS_Compound_to_owned,
    TopoDS_Edge, TopoDS_Edge_to_owned, TopoDS_Shape, TopoDS_Shape_to_owned,
};

pub struct Compound(pub(crate) UniquePtr<TopoDS_Compound>);

impl Compound {
    pub fn center_of_mass_2d(&self) -> Point {
        let mut props = GProp_GProps_ctor();
        BRepGProp_LinearProperties(self.as_ref(), props.pin_mut());
        GProp_GProps_CentreOfMass(&props).into()
    }
}

impl AsRef<TopoDS_Shape> for Compound {
    fn as_ref(&self) -> &TopoDS_Shape {
        cast_compound_to_shape(&self.0)
    }
}

impl From<&TopoDS_Compound> for Compound {
    fn from(value: &TopoDS_Compound) -> Self {
        Compound(TopoDS_Compound_to_owned(value))
    }
}

impl TryFrom<Compound> for Wire {
    type Error = Error;

    fn try_from(value: Compound) -> Result<Self, Self::Error> {
        debug!("converting compound to wire");
        let center = value.center_of_mass_2d();

        let mut explorer = Explorer::<TopoDS_Edge>::new(value);
        let mut wire_builder = BRepBuilderAPI_MakeWire_ctor();

        let mut edges: Vec<Edge> = Vec::new();
        let mut furthest = (0f64, 0);
        while let Some(edge) = explorer.next() {
            let edge: Edge = TopoDS_Edge_to_owned(edge).into();
            let (start, _) = edge.start_end();
            let dist = start.distance(&center);
            if dist > furthest.0 {
                furthest = (dist, edges.len());
            }
            edges.push(edge);
        }
        debug!("found {} edges in compound", edges.len());

        if edges.is_empty() {
            return Err("slice did not return geometry".into());
        }

        let first = edges.remove(furthest.1);
        let (_, mut end) = first.start_end();
        wire_builder.pin_mut().add_edge(&first.0);

        let mut added = 1;
        'main: loop {
            for i in 0..edges.len() {
                let (next_start, next_end) = edges[i].start_end();
                let dist = next_start.distance(&end);

                if dist < 0.0001 {
                    end = next_end;
                    added += 1;
                    wire_builder.pin_mut().add_edge(&edges.remove(i).0);
                    continue 'main;
                } else {
                    let dist = next_end.distance(&end);

                    if dist < 0.0001 {
                        end = next_start;
                        added += 1;
                        wire_builder.pin_mut().add_edge(&edges.remove(i).0);
                        continue 'main;
                    }
                }
            }
            break;
        }
        debug!("added {} edges to wire", added);

        Ok(Wire(TopoDS_Shape_to_owned(Builder::try_build(
            &mut wire_builder,
        )?)))
    }
}
