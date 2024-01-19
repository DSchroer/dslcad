use quick_xml::{se, DeError};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{Seek, Write};
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

pub struct ThreeMF {
    content_types: Vec<ContentType>,
    relationships: Vec<Relationship>,
    model: Model,
}

impl Default for ThreeMF {
    fn default() -> Self {
        let content_types = vec![
            ContentType {
                extension: "rels",
                content_type: "application/vnd.openxmlformats-package.relationships+xml",
            },
            ContentType {
                extension: "model",
                content_type: "application/vnd.ms-package.3dmanufacturing-3dmodel+xml",
            },
        ];

        let relationships = vec![Relationship {
            target: "/3D/3dmodel.model",
            id: "rel0",
            relationship_type: RelationshipType::Model,
        }];

        Self {
            content_types,
            relationships,
            model: Model::default(),
        }
    }
}

impl ThreeMF {
    pub fn add_3d_model(&mut self, vertices: Vec<Vertex>, triangles: Vec<Triangle>) {
        self.model.add_object(vertices, triangles);
    }

    pub fn write_to_zip(&self, writer: impl Write + Seek) -> Result<(), Box<dyn Error>> {
        let mut zip = ZipWriter::new(writer);
        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

        fn write_xml_header(writer: &mut impl std::fmt::Write) -> std::fmt::Result {
            writer.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>")
        }

        {
            zip.start_file("[Content_Types].xml", options)?;
            let mut content_types_buf = String::new();
            write_xml_header(&mut content_types_buf)?;
            self.write_content_types(&mut content_types_buf)?;
            zip.write_all(content_types_buf.as_bytes())?;
        }

        {
            zip.start_file("_rels/.rels", options)?;
            let mut rel_buf = String::new();
            write_xml_header(&mut rel_buf)?;
            self.write_relationships(&mut rel_buf)?;
            zip.write_all(rel_buf.as_bytes())?;
        }

        {
            zip.start_file("3D/3dmodel.model", options)?;
            let mut model_buf = String::new();
            write_xml_header(&mut model_buf)?;
            Self::write_model(&mut model_buf, &self.model)?;
            zip.write_all(model_buf.as_bytes())?;
        }

        Ok(())
    }

    fn write_model(writer: impl std::fmt::Write, model: &Model) -> Result<(), DeError> {
        se::to_writer_with_root(writer, "model", &model)
    }

    fn write_content_types(&self, writer: impl std::fmt::Write) -> Result<(), DeError> {
        se::to_writer_with_root(
            writer,
            "Types",
            &ContentTypes {
                xmlns: "http://schemas.openxmlformats.org/package/2006/content-types",
                types: self.content_types.clone(),
            },
        )
    }

    fn write_relationships(&self, writer: impl std::fmt::Write) -> Result<(), DeError> {
        se::to_writer_with_root(
            writer,
            "Relationships",
            &Relationships {
                xmlns: "http://schemas.openxmlformats.org/package/2006/relationships",
                relationships: self.relationships.clone(),
            },
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct Model {
    #[serde(rename = "@unit")]
    unit: Unit,
    #[serde(rename = "@xml:lang", skip_deserializing)]
    lang: &'static str,
    #[serde(rename = "@xmlns", skip_deserializing)]
    xmlns: &'static str,
    resources: Resources,
    build: Build,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            unit: Unit::default(),
            lang: "en-US",
            xmlns: "http://schemas.microsoft.com/3dmanufacturing/core/2015/02",
            resources: Resources { objects: vec![] },
            build: Build { items: vec![] },
        }
    }
}

impl Model {
    pub fn add_object(&mut self, vertices: Vec<Vertex>, triangles: Vec<Triangle>) {
        let id = self.resources.objects.len() + 1;
        self.resources.objects.push(Object {
            id,
            object_type: "model",
            mesh: Mesh {
                vertices: Vertices { vertices },
                triangles: Triangles { triangles },
            },
        });
        self.build.items.push(BuildItem { object_id: id });
    }
}

#[derive(Serialize, Deserialize, Default)]
enum Unit {
    #[serde(rename = "millimeter")]
    #[default]
    Millimeter,
}

#[derive(Serialize, Deserialize)]
struct Resources {
    #[serde(rename = "object")]
    objects: Vec<Object>,
}

#[derive(Serialize, Deserialize)]
struct Object {
    #[serde(rename = "@id")]
    id: usize,
    #[serde(rename = "@type", skip_deserializing)]
    object_type: &'static str,
    mesh: Mesh,
}

#[derive(Serialize, Deserialize)]
struct Mesh {
    #[serde(rename = "vertices")]
    vertices: Vertices,
    triangles: Triangles,
}

#[derive(Serialize, Deserialize)]
struct Vertices {
    #[serde(rename = "vertex")]
    vertices: Vec<Vertex>,
}

#[derive(Serialize, Deserialize)]
struct Triangles {
    #[serde(rename = "triangle")]
    triangles: Vec<Triangle>,
}

#[derive(Serialize, Deserialize)]
pub struct Vertex {
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@z")]
    pub z: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Triangle {
    #[serde(rename = "@v1")]
    pub v1: usize,
    #[serde(rename = "@v2")]
    pub v2: usize,
    #[serde(rename = "@v3")]
    pub v3: usize,
}

#[derive(Serialize, Deserialize)]
struct Build {
    #[serde(rename = "item")]
    items: Vec<BuildItem>,
}

#[derive(Serialize, Deserialize)]
struct BuildItem {
    #[serde(rename = "@objectid")]
    object_id: usize,
}

#[derive(Serialize, Deserialize)]
struct ContentTypes {
    #[serde(rename = "@xmlns", skip_deserializing)]
    xmlns: &'static str,
    #[serde(rename = "Default")]
    types: Vec<ContentType>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ContentType {
    #[serde(rename = "@Extension", skip_deserializing)]
    extension: &'static str,
    #[serde(rename = "@ContentType", skip_deserializing)]
    content_type: &'static str,
}

#[derive(Serialize, Deserialize, Clone)]
struct Relationships {
    #[serde(rename = "@xmlns", skip_deserializing)]
    xmlns: &'static str,
    #[serde(rename = "Relationship")]
    relationships: Vec<Relationship>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Relationship {
    #[serde(rename = "@Target", skip_deserializing)]
    target: &'static str,
    #[serde(rename = "@Id", skip_deserializing)]
    id: &'static str,
    #[serde(rename = "@Type")]
    relationship_type: RelationshipType,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
enum RelationshipType {
    #[serde(rename = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel")]
    Model,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_writes_content_types() {
        let file = ThreeMF::default();
        let mut buf = String::new();
        file.write_content_types(&mut buf).unwrap();

        assert!(buf.contains("<Types"));
        assert!(buf.contains("<Default "));
    }

    #[test]
    fn it_writes_relationships() {
        let file = ThreeMF::default();

        let mut buf = String::new();
        file.write_relationships(&mut buf).unwrap();

        assert!(buf.contains("<Relationships "));
        assert!(buf.contains("<Relationship "));
    }

    #[test]
    fn it_writes_models() {
        let mut model = Model::default();
        model.add_object(
            vec![Vertex {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }],
            vec![Triangle {
                v1: 0,
                v2: 0,
                v3: 0,
            }],
        );

        let mut buf = String::new();
        ThreeMF::write_model(&mut buf, &model).unwrap();

        assert!(buf.contains("<vertices>"));
        assert!(buf.contains("<triangles>"));
    }
}
