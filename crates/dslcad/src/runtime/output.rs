use crate::elapsed;
use opencascade::{Error, Point, Shape, Wire};
use persistence::protocol::Part;

pub trait IntoPart {
    fn into_part(self, deflection: f64) -> Result<Part, Error>;
}

impl IntoPart for &Point {
    fn into_part(self, _: f64) -> Result<Part, Error> {
        Ok(Part::Planar {
            points: vec![[self.x(), self.y(), self.z()]],
            lines: Vec::new(),
        })
    }
}

impl IntoPart for &Wire {
    fn into_part(self, deflection: f64) -> Result<Part, Error> {
        Ok(Part::Planar {
            points: [self.start()?, self.end()?]
                .into_iter()
                .flatten()
                .map(|p| p.into())
                .collect(),
            lines: self.points(deflection)?,
        })
    }
}

impl IntoPart for &Shape {
    fn into_part(self, deflection: f64) -> Result<Part, Error> {
        let original = elapsed!("generated mesh", self.mesh(deflection)?);
        let mut mesh = persistence::protocol::Mesh {
            vertices: original.vertices.clone(),
            triangles: vec![],
            normals: vec![],
        };

        for (tri, normal) in original.triangles_with_normals() {
            mesh.triangles.push(*tri);
            mesh.normals.push(normal);
        }

        Ok(Part::Object {
            points: elapsed!("generated points", self.points()?),
            lines: elapsed!("generated lines", self.lines(deflection)?),
            mesh,
        })
    }
}
