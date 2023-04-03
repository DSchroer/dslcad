use dslcad_parser::Ast;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Render {
        ast: Ast,
    },
    RenderResults(Result<Render, CadError>, RenderMetadata),
    Export {
        render: Render,
        name: String,
        path: String,
    },
    ExportResults(),
    Error(CadError),
    CheatSheet(),
    CheatSheetResults {
        cheatsheet: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Render {
    pub parts: Vec<Part>,
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
