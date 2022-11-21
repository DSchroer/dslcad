use opencascade::{Edge, IndexedMesh, Point, Shape};
use std::cell::{Ref, RefMut};
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct Output {
    text: String,
    points: Vec<[f64; 3]>,
    lines: Vec<Vec<[f64; 3]>>,
    mesh: IndexedMesh,
}

impl Output {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn points(&self) -> &Vec<[f64; 3]> {
        &self.points
    }

    pub fn lines(&self) -> &Vec<Vec<[f64; 3]>> {
        &self.lines
    }

    pub fn mesh(&self) -> &IndexedMesh {
        &self.mesh
    }
}

impl From<&f64> for Output {
    fn from(value: &f64) -> Self {
        Output {
            text: value.to_string(),
            ..Default::default()
        }
    }
}

impl From<&bool> for Output {
    fn from(value: &bool) -> Self {
        Output {
            text: value.to_string(),
            ..Default::default()
        }
    }
}

impl From<&String> for Output {
    fn from(value: &String) -> Self {
        Output {
            text: value.to_string(),
            ..Default::default()
        }
    }
}

impl From<Ref<'_, Point>> for Output {
    fn from(value: Ref<Point>) -> Self {
        Output {
            points: vec![[value.x(), value.y(), value.z()]],
            ..Default::default()
        }
    }
}

impl From<RefMut<'_, Edge>> for Output {
    fn from(mut value: RefMut<Edge>) -> Self {
        Output {
            lines: value.points(),
            ..Default::default()
        }
    }
}

impl TryFrom<RefMut<'_, Shape>> for Output {
    type Error = std::io::Error;

    fn try_from(mut value: RefMut<'_, Shape>) -> Result<Self, Self::Error> {
        Ok(Output {
            mesh: value.mesh()?,
            ..Default::default()
        })
    }
}

impl Default for Output {
    fn default() -> Self {
        Output {
            text: String::new(),
            points: Vec::new(),
            lines: Vec::new(),
            mesh: IndexedMesh {
                faces: Vec::new(),
                vertices: Vec::new(),
            },
        }
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}
