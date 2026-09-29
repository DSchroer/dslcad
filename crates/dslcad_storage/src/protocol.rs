mod aabb;
mod stl;

use crate::threemf::{ThreeMF, Triangle, Vertex};
pub use aabb::BoundingBox;
pub use bincode::Error as BincodeError;
use serde::{Deserialize, Serialize};
pub use stl::StlError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Render {
    pub parts: Vec<Part>,
    pub stdout: String,
    /// Drawing overlays (dimensions, labels, markers) that are drawn on top of
    /// the model and do not take part in framing or export.
    #[serde(default)]
    pub annotations: Vec<Annotation>,
    /// Named views that group a camera with the annotations it draws.
    #[serde(default)]
    pub views: Vec<ViewDef>,
    /// Top level parameters a script declares, so an editor can offer widgets
    /// for them. Empty when a model has no parameters.
    #[serde(default)]
    pub parameters: Vec<Parameter>,
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

/// The kind of a drawing annotation. It is kept in the protocol so the viewer
/// can style and toggle annotations independently of the model.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnotationKind {
    Dimension,
    Label,
    Leader,
    Level,
    North,
    Centerline,
    Title,
}

/// A drawing overlay, already reduced to line and point geometry. Annotations
/// never contribute to the model bounds.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Annotation {
    pub kind: AnnotationKind,
    pub lines: Vec<Vec<Point>>,
    pub points: Vec<Point>,
    /// Text drawn by the viewer so it can stay filled, upright and facing the
    /// camera regardless of the model orientation.
    #[serde(default)]
    pub texts: Vec<TextBlock>,
}

impl Annotation {
    pub fn new(kind: AnnotationKind, lines: Vec<Vec<Point>>) -> Self {
        Annotation {
            kind,
            lines,
            points: Vec::new(),
            texts: Vec::new(),
        }
    }
}

/// The plane a [`TextBlock`] lies on, or `Billboard` to always face the camera.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextPlane {
    Billboard,
    Xy,
    Yz,
    Xz,
}

/// Text geometry in a local 2D frame (z is 0) plus where to place it. The
/// viewer orients the local frame using [`TextPlane`].
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TextBlock {
    /// Solid fill made of closely spaced scanlines.
    pub lines: Vec<Vec<Point>>,
    /// The glyph contours, drawn dark so the fill stays readable on any color.
    #[serde(default)]
    pub outline: Vec<Vec<Point>>,
    pub origin: Point,
    pub plane: TextPlane,
}

/// Camera rotations around the part axes, in degrees. An unset axis keeps the
/// viewer's default.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewAngles {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Projection {
    #[default]
    Perspective,
    Orthographic,
}

/// Which features a view draws.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShowFlags {
    pub model: bool,
    pub points: bool,
    pub lines: bool,
    pub mesh: bool,
    pub annotations: bool,
}

impl Default for ShowFlags {
    fn default() -> Self {
        ShowFlags {
            model: true,
            points: true,
            lines: true,
            mesh: true,
            annotations: true,
        }
    }
}

/// A named camera together with the geometry and annotations it draws.
///
/// A view is empty by default: `parts` holds only the geometry authored in the
/// view's body (including a `model()` reference), never the document's shared
/// scene. The shared scene is [`Render::parts`].
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ViewDef {
    pub name: Option<String>,
    pub angle: ViewAngles,
    pub projection: Projection,
    pub zoom: Option<f32>,
    pub target: Option<Point>,
    pub fit: bool,
    pub show: ShowFlags,
    #[serde(default)]
    pub parts: Vec<Part>,
    pub annotations: Vec<Annotation>,
}

/// The type a script parameter declares, used to pick an editor widget.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterType {
    Number,
    Integer,
    Bool,
    Text,
}

/// The current value of a script parameter.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ParameterValue {
    Number(f64),
    Bool(bool),
    Text(String),
}

/// A top level `var name(...) = value;` parameter with the metadata an editor
/// needs to build a control for it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub kind: ParameterType,
    pub value: ParameterValue,
    /// Lower bound, when declared.
    #[serde(default)]
    pub min: Option<f64>,
    /// Upper bound, when declared.
    #[serde(default)]
    pub max: Option<f64>,
    /// Increment used by a slider, when declared.
    #[serde(default)]
    pub step: Option<f64>,
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
            annotations: vec![Annotation::new(
                AnnotationKind::Dimension,
                vec![vec![
                    Point::from((0.0, 0.0, 0.0)),
                    Point::from((1.0, 0.0, 0.0)),
                ]],
            )],
            views: Vec::new(),
            parameters: Vec::new(),
        };

        let serialized: Vec<u8> = render.clone().try_into().unwrap();
        let deserialized: Render = serialized.as_slice().try_into().unwrap();

        assert_eq!(render, deserialized);
    }
}
