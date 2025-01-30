mod aabb;

use crate::threemf::{ThreeMF, Triangle, Vertex};
pub use aabb::BoundingBox;
pub use bincode::Error as BincodeError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Render {
    pub parts: Vec<Part>,
    pub stdout: String,
}

impl TryFrom<&[u8]> for Render {
    type Error = BincodeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        bincode::deserialize::<Render>(value)
    }
}

impl TryFrom<Render> for Vec<u8> {
    type Error = BincodeError;

    fn try_from(value: Render) -> Result<Self, Self::Error> {
        bincode::serialize(&value)
    }
}

impl From<Render> for ThreeMF {
    fn from(value: Render) -> Self {
        let mut tmf = ThreeMF::default();
        for part in value.parts.into_iter() {
            if let Part::Object { mesh, .. } = part {
                tmf.add_3d_model(
                    mesh.vertices.into_iter().map(Into::into).collect(),
                    mesh.triangles.into_iter().map(Into::into).collect(),
                );
            }
        }
        tmf
    }
}

pub type Vec3<T> = [T; 3];
pub type Point = Vec3<f64>;

impl From<Vec3<usize>> for Triangle {
    fn from(value: Vec3<usize>) -> Self {
        Triangle {
            v1: value[0],
            v2: value[1],
            v3: value[2],
        }
    }
}

impl From<Vec3<f64>> for Vertex {
    fn from(value: Vec3<f64>) -> Self {
        Vertex {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Part {
    Empty,
    Planar {
        points: Vec<Point>,
        lines: Vec<Vec<Point>>,
    },
    Object {
        points: Vec<Point>,
        lines: Vec<Vec<Point>>,
        mesh: Mesh,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Point>,
    pub triangles: Vec<Vec3<usize>>,
    pub normals: Vec<Point>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serializes_and_deserializes() {
        let render = Render {
            parts: vec![
                Part::Empty,
                Part::Planar {
                    points: vec![],
                    lines: vec![],
                },
                Part::Object {
                    points: vec![Point::from((0.0, 1.0, 2.0))],
                    lines: vec![],
                    mesh: Mesh {
                        vertices: vec![],
                        triangles: vec![],
                        normals: vec![],
                    },
                },
            ],
            stdout: String::from("hello"),
        };

        let serialized: Vec<u8> = render.clone().try_into().unwrap();
        let deserialized: Render = serialized.as_slice().try_into().unwrap();

        assert_eq!(render, deserialized);
    }
}
