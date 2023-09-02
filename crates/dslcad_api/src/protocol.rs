use serde::{Deserialize, Serialize};
use serde_binary::binary_stream::Endian;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Render {
    pub parts: Vec<Part>,
}

impl Render {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_binary::to_vec(self, Endian::Big).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        serde_binary::from_slice(bytes, Endian::Big).unwrap()
    }
}

pub type Vec3<T> = [T; 3];
pub type Point = Vec3<f64>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Part {
    Data {
        text: String,
    },
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Point>,
    pub triangles: Vec<Vec3<usize>>,
    pub normals: Vec<Point>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RenderMetadata {}

#[derive(Serialize, Deserialize, Debug)]
pub struct CadError {
    pub error: String,
}

impl Display for CadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.error))
    }
}

impl Error for CadError {}
