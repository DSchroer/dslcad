//! Drawing annotations: measurements, dimensions and labels.
//!
//! Annotations are ordinary values. They carry line geometry that the viewer
//! overlays on top of the model, and they never contribute to the part bounds.
//! Text is outlined into polylines with the built-in font so labels render in
//! both the preview and the screenshot without a text overlay pass.

use crate::resources::{text_fill_lines, text_to_lines};
use crate::runtime::{RuntimeError, Value};
use dslcad_occt::{DsShape, Point, Shape};
use dslcad_storage::protocol::{Annotation, AnnotationKind, TextBlock, TextPlane};
use std::rc::Rc;

type V3 = [f64; 3];

const DEFAULT_TEXT_SIZE: f64 = 5.0;
const DEFAULT_OFFSET: f64 = 5.0;
const DEFAULT_ARROW: f64 = 2.0;

// ---------------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------------

/// Distance between two points, in millimetres.
pub fn measure_points(start: &Point, end: &Point) -> Result<Value, RuntimeError> {
    Ok(start.distance(end).into())
}

/// The x, y and z extents of a shape's bounding box.
pub fn measure_shape(shape: &Shape) -> Result<Value, RuntimeError> {
    let (minimum, maximum) = shape.bounds()?;
    Ok(Point::new(
        maximum.x() - minimum.x(),
        maximum.y() - minimum.y(),
        maximum.z() - minimum.z(),
    )
    .into())
}

/// Convert a value between units. Millimetres are the canonical unit.
pub fn convert(value: f64, source: String, target: String) -> Result<Value, RuntimeError> {
    let source = factor(&source)?;
    let target = factor(&target)?;
    Ok((value * source / target).into())
}

fn factor(unit: &str) -> Result<f64, RuntimeError> {
    match unit {
        "mm" => Ok(1.0),
        "cm" => Ok(10.0),
        "m" => Ok(1000.0),
        "in" => Ok(25.4),
        "ft" => Ok(304.8),
        other => Err(RuntimeError::UserDefined(format!("unknown unit '{other}'"))),
    }
}

fn format_measurement(value_mm: f64, units: Option<String>, precision: Option<f64>) -> String {
    let unit = units.unwrap_or_else(|| "mm".to_string());
    let precision = precision.unwrap_or(1.0).clamp(0.0, 10.0) as usize;
    let converted = factor(&unit).map(|f| value_mm / f).unwrap_or(value_mm);
    let suffix = if factor(&unit).is_ok() {
        unit.as_str()
    } else {
        "mm"
    };
    format!("{converted:.precision$} {suffix}")
}

// ---------------------------------------------------------------------------
// Dimensions
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn dimension(
    start: &Point,
    end: &Point,
    offset: Option<f64>,
    axis: Option<String>,
    text: Option<String>,
    units: Option<String>,
    precision: Option<f64>,
    _arrow: Option<String>,
    plane: Option<String>,
) -> Result<Value, RuntimeError> {
    let a = to_v3(start);
    let b = to_v3(end);
    let offset = offset.unwrap_or(DEFAULT_OFFSET);

    let (p0, p1) = match axis.as_deref() {
        Some("x") => ([a[0], a[1], a[2]], [b[0], a[1], a[2]]),
        Some("y") => ([a[0], a[1], a[2]], [a[0], b[1], a[2]]),
        Some("z") => ([a[0], a[1], a[2]], [a[0], a[1], b[2]]),
        Some(other) => {
            return Err(RuntimeError::UserDefined(format!(
                "unknown dimension axis '{other}'"
            )))
        }
        None => (a, b),
    };

    let direction = vsub(p1, p0);
    let length = vlen(direction);

    if length < 1e-9 {
        return Err(RuntimeError::UserDefined(
            "dimension has zero length".to_string(),
        ));
    }

    let along = vnorm(direction);
    // The plane decides which way the dimension is offset, so the annotation
    // lands in the same plane the view shows.
    let perpendicular = offset_normal(direction, plane.as_deref());
    let shift = vscale(perpendicular, offset);

    let d0 = vadd(p0, shift);
    let d1 = vadd(p1, shift);

    let mut drawing = Drawing::new(AnnotationKind::Dimension);

    if offset.abs() > 1e-9 {
        drawing.line(vec![p0, d0]);
        drawing.line(vec![p1, d1]);
    }

    drawing.line(vec![d0, d1]);

    let arrow = DEFAULT_ARROW.min(length * 0.2);
    drawing.line(arrowhead(d0, along, arrow));
    drawing.line(arrowhead(d1, vscale(along, -1.0), arrow));

    let label = text.unwrap_or_else(|| format_measurement(length, units, precision));
    let size = DEFAULT_TEXT_SIZE.min(length * 0.5).max(1.0);
    let label_at = vadd(
        vmid(d0, d1),
        vscale(perpendicular, offset.signum() * (size + 1.0)),
    );
    drawing.text(&label, label_at, size, None)?;

    Ok(drawing.finish())
}

pub fn dimension_radial(
    radius: f64,
    center: &Point,
    at: Option<Rc<Point>>,
    text: Option<String>,
    units: Option<String>,
    precision: Option<f64>,
) -> Result<Value, RuntimeError> {
    let c = to_v3(center);
    let target = at
        .as_ref()
        .map(|p| to_v3(p))
        .unwrap_or_else(|| vadd(c, [radius, 0.0, 0.0]));

    let mut drawing = Drawing::new(AnnotationKind::Dimension);

    drawing.line(vec![c, target]);

    let direction = vsub(target, c);
    let length = vlen(direction);
    if length > 1e-9 {
        drawing.line(arrowhead(
            target,
            direction,
            DEFAULT_ARROW.min(length * 0.2),
        ));
    }

    let label =
        text.unwrap_or_else(|| format!("R{}", format_measurement(radius, units, precision)));
    drawing.text(&label, vmid(c, target), DEFAULT_TEXT_SIZE, None)?;

    Ok(drawing.finish())
}

pub fn dimension_angular(
    angle: f64,
    center: &Point,
    start: &Point,
    end: &Point,
    units: Option<String>,
    precision: Option<f64>,
) -> Result<Value, RuntimeError> {
    let c = to_v3(center);
    let s = to_v3(start);
    let e = to_v3(end);

    let v1 = vsub(s, c);
    let v2 = vsub(e, c);
    let axis = vcross(v1, v2);
    let axis_length = vlen(axis);

    if axis_length < 1e-9 || vlen(v1) < 1e-9 {
        return Err(RuntimeError::UserDefined(
            "angular dimension points must not be collinear".to_string(),
        ));
    }

    let axis = vnorm(axis);
    let radius = vlen(v1);
    let steps = 24;

    let mut arc = Vec::with_capacity(steps + 1);
    for step in 0..=steps {
        let fraction = step as f64 / steps as f64;
        let rotated = rotate_about(v1, axis, angle.to_radians() * fraction);
        arc.push(vadd(c, rotated));
    }

    let mut drawing = Drawing::new(AnnotationKind::Dimension);
    drawing.line(arc.clone());
    drawing.line(vec![c, s]);
    drawing.line(vec![c, e]);
    drawing.line(arrowhead(s, vsub(s, c), DEFAULT_ARROW.min(radius * 0.2)));
    if let Some(last) = arc.last() {
        drawing.line(arrowhead(
            *last,
            vsub(*last, c),
            DEFAULT_ARROW.min(radius * 0.2),
        ));
    }

    let label = format!("{angle}°");
    let _ = (units, precision);
    drawing.text(
        &label,
        vadd(c, vscale(vnorm(vadd(v1, v2)), radius + 3.0)),
        DEFAULT_TEXT_SIZE,
        None,
    )?;

    Ok(drawing.finish())
}

// ---------------------------------------------------------------------------
// Labels and architectural components
// ---------------------------------------------------------------------------

pub fn label(
    text: String,
    at: &Point,
    anchor: Option<Rc<Point>>,
    size: Option<f64>,
    plane: Option<String>,
) -> Result<Value, RuntimeError> {
    let mut drawing = Drawing::new(AnnotationKind::Label);

    if let Some(anchor) = anchor {
        drawing.line(vec![to_v3(&anchor), to_v3(at)]);
    }

    drawing.text(
        &text,
        to_v3(at),
        size.unwrap_or(DEFAULT_TEXT_SIZE),
        plane.as_deref(),
    )?;

    Ok(drawing.finish())
}

pub fn leader(text: String, at: &Point, to: &Point) -> Result<Value, RuntimeError> {
    let mut drawing = Drawing::new(AnnotationKind::Leader);

    let arrow = vsub(to_v3(at), to_v3(to));
    drawing.line(vec![to_v3(to), to_v3(at)]);
    if vlen(arrow) > 1e-9 {
        drawing.line(arrowhead(to_v3(to), vscale(arrow, -1.0), DEFAULT_ARROW));
    }
    drawing.text(&text, to_v3(at), DEFAULT_TEXT_SIZE, None)?;

    Ok(drawing.finish())
}

pub fn level(z: f64, text: Option<String>, at: Option<Rc<Point>>) -> Result<Value, RuntimeError> {
    let (x, y) = at.as_ref().map(|p| (p.x(), p.y())).unwrap_or((0.0, 0.0));

    let mut drawing = Drawing::new(AnnotationKind::Level);

    drawing.line(vec![[x - 6.0, y, z], [x + 6.0, y, z]]);
    drawing.line(vec![
        [x - 1.5, y, z],
        [x + 1.5, y, z],
        [x, y, z + 3.0],
        [x - 1.5, y, z],
    ]);

    let text = text.unwrap_or_else(|| format!("{z}"));
    drawing.text(&text, [x + 3.0, y, z], DEFAULT_TEXT_SIZE, None)?;

    Ok(drawing.finish())
}

pub fn north(angle: Option<f64>, at: Option<Rc<Point>>) -> Result<Value, RuntimeError> {
    let center = at.as_ref().map(|p| to_v3(p)).unwrap_or([0.0, 0.0, 0.0]);
    let radians = angle.unwrap_or(0.0).to_radians();
    let direction = [-radians.sin(), radians.cos(), 0.0];

    let tip = vadd(center, vscale(direction, 20.0));
    let back = vscale(direction, -4.0);
    let side = vscale([direction[1], -direction[0], 0.0], 2.0);

    let mut drawing = Drawing::new(AnnotationKind::North);

    drawing.line(vec![center, tip]);
    drawing.line(vec![tip, vadd(vadd(tip, back), side)]);
    drawing.line(vec![tip, vsub(vadd(tip, back), side)]);
    drawing.text(
        "N",
        vadd(center, vscale(direction, 24.0)),
        DEFAULT_TEXT_SIZE,
        None,
    )?;

    Ok(drawing.finish())
}

pub fn centerline(start: &Point, end: &Point) -> Result<Value, RuntimeError> {
    let mut drawing = Drawing::new(AnnotationKind::Centerline);

    for segment in dashes(to_v3(start), to_v3(end), 5.0, 3.0) {
        drawing.line(segment);
    }

    Ok(drawing.finish())
}

pub fn title(
    text: String,
    subtitle: Option<String>,
    scale: Option<String>,
) -> Result<Value, RuntimeError> {
    let mut drawing = Drawing::new(AnnotationKind::Title);

    drawing.text(&text, [0.0, 0.0, 0.0], 6.0, None)?;

    if let Some(subtitle) = subtitle {
        drawing.text(&subtitle, [0.0, -8.0, 0.0], 4.0, None)?;
    }

    if let Some(scale) = scale {
        drawing.text(&format!("SCALE {scale}"), [0.0, -14.0, 0.0], 4.0, None)?;
    }

    Ok(drawing.finish())
}

// ---------------------------------------------------------------------------
// Drawing builder
// ---------------------------------------------------------------------------

struct Drawing {
    kind: AnnotationKind,
    lines: Vec<Vec<V3>>,
    texts: Vec<TextBlock>,
}

impl Drawing {
    fn new(kind: AnnotationKind) -> Self {
        Drawing {
            kind,
            lines: Vec::new(),
            texts: Vec::new(),
        }
    }

    fn line(&mut self, points: Vec<V3>) {
        if points.len() >= 2 {
            self.lines.push(points);
        }
    }

    fn text(
        &mut self,
        text: &str,
        origin: V3,
        size: f64,
        plane: Option<&str>,
    ) -> Result<(), RuntimeError> {
        let plane = match plane {
            Some("xy") => TextPlane::Xy,
            Some("yz") => TextPlane::Yz,
            Some("xz") => TextPlane::Xz,
            _ => TextPlane::Billboard,
        };

        self.texts.push(TextBlock {
            lines: text_fill_lines(text, size)?,
            outline: text_to_lines(text, size)?,
            origin,
            plane,
        });

        Ok(())
    }

    fn finish(self) -> Value {
        let mut annotation = Annotation::new(self.kind, self.lines);
        annotation.texts = self.texts;
        annotation.into()
    }
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

fn to_v3(point: &Point) -> V3 {
    [point.x(), point.y(), point.z()]
}

fn vadd(left: V3, right: V3) -> V3 {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn vsub(left: V3, right: V3) -> V3 {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn vscale(vector: V3, scale: f64) -> V3 {
    [vector[0] * scale, vector[1] * scale, vector[2] * scale]
}

fn vlen(vector: V3) -> f64 {
    (vector[0].powi(2) + vector[1].powi(2) + vector[2].powi(2)).sqrt()
}

fn vnorm(vector: V3) -> V3 {
    let length = vlen(vector);
    if length < 1e-12 {
        vector
    } else {
        vscale(vector, 1.0 / length)
    }
}

fn vcross(left: V3, right: V3) -> V3 {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn vmid(left: V3, right: V3) -> V3 {
    vscale(vadd(left, right), 0.5)
}

/// A unit vector perpendicular to `direction`, chosen to be stable for
/// directions that are parallel to a world axis.
fn perpendicular_of(direction: V3) -> V3 {
    if direction[2].abs() > 0.9 {
        vnorm(vcross(direction, [1.0, 0.0, 0.0]))
    } else {
        vnorm(vcross(direction, [0.0, 0.0, 1.0]))
    }
}

/// A unit vector perpendicular to `direction` that stays inside the given
/// drawing plane (`"xy"`, `"yz"` or `"xz"`). Without a plane it falls back to
/// [`perpendicular_of`].
fn offset_normal(direction: V3, plane: Option<&str>) -> V3 {
    let reference = match plane {
        Some("xy") => [0.0, 0.0, 1.0],
        Some("xz") => [0.0, 1.0, 0.0],
        Some("yz") => [1.0, 0.0, 0.0],
        _ => return perpendicular_of(direction),
    };

    let normal = vcross(direction, reference);
    if vlen(normal) < 1e-9 {
        perpendicular_of(direction)
    } else {
        vnorm(normal)
    }
}

/// A two-segment "V" arrowhead with its tip at `tip`, pointing away from `back`.
fn arrowhead(tip: V3, back: V3, size: f64) -> Vec<V3> {
    let direction = vnorm(back);
    let up = if direction[2].abs() > 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let side = vnorm(vcross(direction, up));
    let base = vadd(tip, vscale(direction, size));

    vec![
        vadd(base, vscale(side, size * 0.35)),
        tip,
        vsub(base, vscale(side, size * 0.35)),
    ]
}

/// Split a line into dashes separated by gaps.
fn dashes(start: V3, end: V3, dash: f64, gap: f64) -> Vec<Vec<V3>> {
    let direction = vsub(end, start);
    let length = vlen(direction);

    if length < 1e-9 {
        return Vec::new();
    }

    let unit = vscale(direction, 1.0 / length);
    let mut segments = Vec::new();
    let mut position = 0.0;

    while position < length {
        let stop = (position + dash).min(length);
        segments.push(vec![
            vadd(start, vscale(unit, position)),
            vadd(start, vscale(unit, stop)),
        ]);
        position = stop + gap;
    }

    segments
}

/// Rotate `vector` around the unit `axis` by `radians` (Rodrigues' formula).
fn rotate_about(vector: V3, axis: V3, radians: f64) -> V3 {
    let (sin, cos) = radians.sin_cos();
    let term1 = vscale(vector, cos);
    let term2 = vscale(vcross(axis, vector), sin);
    let term3 = vscale(axis, dot(axis, vector) * (1.0 - cos));

    vadd(vadd(term1, term2), term3)
}

fn dot(left: V3, right: V3) -> f64 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::Library;
    use crate::parser::{Ast, DocId, Parser, Reader};
    use crate::runtime::Engine;
    use std::collections::HashMap;
    use std::io::Error;
    use std::path::Path;

    fn run(code: &'static str) -> Value {
        let ast: Ast = Parser::new(TestReader(code), DocId::new("test".to_string()))
            .parse()
            .unwrap();
        let lib = Library::default();
        let mut engine = Engine::new(&lib, &ast);
        engine.eval_root(HashMap::new()).unwrap()
    }

    fn annotation(code: &'static str) -> Annotation {
        run(code)
            .to_annotation()
            .unwrap_or_else(|_| panic!("expected an annotation for {code}"))
            .as_ref()
            .clone()
    }

    #[test]
    fn it_measures_points_and_shapes() {
        assert_eq!(
            Ok(5.0),
            run("measure(start=point(x=0,y=0), end=point(x=3,y=4));").to_number()
        );

        let extents = run("measure(shape=cube(x=2, y=3, z=4));")
            .to_point()
            .unwrap();
        assert!((extents.x() - 2.0).abs() < 1e-6);
        assert!((extents.y() - 3.0).abs() < 1e-6);
        assert!((extents.z() - 4.0).abs() < 1e-6);
    }

    #[test]
    fn it_converts_units() {
        assert_eq!(
            Ok(25.4),
            run(r#"convert(value=1, source="in", target="mm");"#).to_number()
        );
        assert_eq!(
            Ok(1.0),
            run(r#"convert(value=25.4, source="mm", target="in");"#).to_number()
        );
        assert!(crate::library::drawing::convert(1.0, "foo".into(), "mm".into()).is_err());
    }

    #[test]
    fn it_builds_a_linear_dimension() {
        let annotation =
            annotation("dimension(start=point(x=0,y=0), end=point(x=60,y=0), offset=8);");

        assert_eq!(AnnotationKind::Dimension, annotation.kind);
        // Extension lines, the dimension line and two arrowheads.
        assert!(
            annotation.lines.len() >= 5,
            "got {} lines",
            annotation.lines.len()
        );
        // The measured value is drawn as text.
        assert_eq!(1, annotation.texts.len());
        assert!(!annotation.texts[0].lines.is_empty());
    }

    #[test]
    fn it_labels_the_measured_value_in_the_chosen_unit() {
        assert_eq!("60.0 mm", format_measurement(60.0, None, None));
        assert_eq!(
            "2.4 in",
            format_measurement(60.0, Some("in".into()), Some(1.0))
        );
        assert_eq!(
            "6.00 cm",
            format_measurement(60.0, Some("cm".into()), Some(2.0))
        );
    }

    #[test]
    fn it_builds_labels_and_components() {
        for code in [
            r#"label(text="Bracket", at=point(x=1,y=2,z=3));"#,
            r#"leader(text="Note", at=point(x=0,y=0,z=10), to=point(x=5,y=0,z=0));"#,
            "level(z=12, at=point(x=1,y=2));",
            "north(angle=30, at=point(x=0,y=0));",
            "centerline(start=point(x=0,y=0,z=0), end=point(x=10,y=0,z=0));",
            r#"title(text="Plan", subtitle="1:50", scale="1:50");"#,
            r#"dimension(radius=5, center=point(x=0,y=0), at=point(x=5,y=0));"#,
            r#"dimension(angle=90, center=point(x=0,y=0,z=0), start=point(x=10,y=0,z=0), end=point(x=0,y=10,z=0));"#,
        ] {
            let annotation = annotation(code);
            assert!(
                !annotation.lines.is_empty() || !annotation.texts.is_empty(),
                "no geometry for {code}"
            );
        }
    }

    #[test]
    fn it_renders_labels_as_filled_text() {
        let plain = annotation(r#"label(text="hello", at=point(x=0,y=0,z=0));"#);
        assert_eq!(1, plain.texts.len());

        let text = &plain.texts[0];
        assert_eq!(TextPlane::Billboard, text.plane);
        // Scanline fill produces many short horizontal segments.
        assert!(text.lines.len() > 10, "got {} text lines", text.lines.len());
        assert!(text
            .lines
            .iter()
            .all(|line| (line[0][1] - line[1][1]).abs() < 1e-9));
    }

    pub struct TestReader(pub &'static str);
    impl Reader for TestReader {
        fn read_bytes(&self, _: &Path) -> Result<Vec<u8>, Error> {
            todo!()
        }

        fn read(&self, _: &Path) -> Result<String, std::io::Error> {
            Ok(self.0.to_string())
        }
    }
}
