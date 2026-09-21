use crate::parser::{DocumentParseError, Literal, Reader};
use crate::resources::svg_loader::Svg;
use crate::resources::{Resource, ResourceLoader};
use crate::runtime::{RuntimeError, Value};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Write};
use std::path::Path;
use ttf_parser::{Face, OutlineBuilder};

/// Em size, in model units, used when the `size` argument is not given.
const DEFAULT_SIZE: f64 = 10.0;

pub struct TtfLoader;

impl ResourceLoader for TtfLoader {
    fn load(
        &self,
        path: &str,
        reader: &dyn Reader,
        arguments: &HashMap<String, Literal>,
    ) -> Result<Box<dyn Resource>, DocumentParseError> {
        let message = match arguments.get("message") {
            Some(Literal::Text(message)) => message.clone(),
            _ => {
                return Err(DocumentParseError::InvalidResource(
                    "a text 'message' argument is required".into(),
                ))
            }
        };

        let size = match arguments.get("size") {
            Some(Literal::Number(size)) => *size,
            None => DEFAULT_SIZE,
            _ => {
                return Err(DocumentParseError::InvalidResource(
                    "'size' must be a number".into(),
                ))
            }
        };

        let data = reader
            .read_bytes(Path::new(path))
            .map_err(|_| DocumentParseError::NoSuchFile())?;

        Ttf::parse(&data, &message, size)
            .map(|ttf| Box::new(ttf) as Box<dyn Resource>)
            .map_err(DocumentParseError::InvalidResource)
    }
}

pub struct Ttf {
    text: String,
    size: f64,
    svg: Svg,
}

impl Debug for Ttf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ttf")
            .field("text", &self.text)
            .field("size", &self.size)
            .finish()
    }
}

impl Resource for Ttf {
    fn to_instance(&self) -> Result<Value, RuntimeError> {
        self.svg.to_instance()
    }
}

impl Ttf {
    fn parse(data: &[u8], text: &str, size: f64) -> Result<Self, String> {
        let face = Face::parse(data, 0).map_err(|error| error.to_string())?;

        if face.units_per_em() == 0 {
            return Err("font has an invalid units per em".into());
        }

        let scale = size / f64::from(face.units_per_em());
        let path = outline_text(&face, text, scale);

        Ok(Ttf {
            text: text.to_string(),
            size,
            svg: Svg::from_paths([path])?,
        })
    }
}

/// Lays out `text` on a horizontal baseline and converts each glyph outline
/// into SVG path data, which the SVG contour pipeline turns into wires.
fn outline_text(face: &Face, text: &str, scale: f64) -> String {
    let mut path = String::new();
    let mut pen = 0.0;

    for character in text.chars() {
        let Some(glyph) = face.glyph_index(character) else {
            continue;
        };

        let mut glyph_path = String::new();
        {
            let mut builder = PathBuilder {
                path: &mut glyph_path,
                scale,
                offset: pen,
            };

            if face.outline_glyph(glyph, &mut builder).is_some() {
                path.push_str(&glyph_path);
            }
        }

        pen += f64::from(face.glyph_hor_advance(glyph).unwrap_or(0)) * scale;
    }

    path
}

struct PathBuilder<'a> {
    path: &'a mut String,
    scale: f64,
    offset: f64,
}

impl PathBuilder<'_> {
    fn point(&self, x: f32, y: f32) -> (f64, f64) {
        (
            f64::from(x) * self.scale + self.offset,
            f64::from(y) * self.scale,
        )
    }
}

impl OutlineBuilder for PathBuilder<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.point(x, y);
        let _ = write!(self.path, "M {x} {y} ");
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.point(x, y);
        let _ = write!(self.path, "L {x} {y} ");
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x1, y1) = self.point(x1, y1);
        let (x, y) = self.point(x, y);
        let _ = write!(self.path, "Q {x1} {y1} {x} {y} ");
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (x1, y1) = self.point(x1, y1);
        let (x2, y2) = self.point(x2, y2);
        let (x, y) = self.point(x, y);
        let _ = write!(self.path, "C {x1} {y1} {x2} {y2} {x} {y} ");
    }

    fn close(&mut self) {
        let _ = write!(self.path, "Z ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::FsReader;
    use dslcad_occt::DsShape;

    const FONT: &str = "../../examples/text/Electrolize-Regular.ttf";

    fn arguments(text: &str, size: f64) -> HashMap<String, Literal> {
        HashMap::from([
            ("message".to_string(), Literal::Text(text.to_string())),
            ("size".to_string(), Literal::Number(size)),
        ])
    }

    fn plane(arguments: &HashMap<String, Literal>) -> Value {
        TtfLoader
            .load(FONT, &FsReader, arguments)
            .unwrap()
            .to_instance()
            .unwrap()
    }

    #[test]
    fn it_renders_text_as_a_plane() {
        assert!(matches!(plane(&arguments("hi", 10.0)), Value::Plane(_)));
    }

    #[test]
    fn it_scales_text_with_the_size_argument() {
        let small = plane(&arguments("M", 10.0))
            .to_plane()
            .unwrap()
            .max_dimension()
            .unwrap();
        let large = plane(&arguments("M", 20.0))
            .to_plane()
            .unwrap()
            .max_dimension()
            .unwrap();

        assert!((large / small - 2.0).abs() < 1e-6);
    }

    #[test]
    fn it_requires_a_text_message() {
        assert!(TtfLoader.load(FONT, &FsReader, &HashMap::new()).is_err());

        assert!(TtfLoader
            .load(
                FONT,
                &FsReader,
                &HashMap::from([("message".to_string(), Literal::Number(1.0))]),
            )
            .is_err());
    }
}
