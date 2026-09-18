use crate::parser::{DocumentParseError, Reader};
use crate::resources::{Resource, ResourceLoader};
use crate::runtime::{RuntimeError, Value};
use dslcad_occt::{Edge, Point, Wire, WireFactory};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader as XmlReader;
use std::f64::consts::PI;
use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::rc::Rc;

const EPSILON: f64 = 1e-9;

pub struct SvgLoader;

impl<R: Reader> ResourceLoader<R> for SvgLoader {
    fn load(&self, path: &str, reader: &R) -> Result<Box<dyn Resource>, DocumentParseError> {
        let data = reader
            .read(Path::new(path))
            .map_err(|_| DocumentParseError::NoSuchFile())?;

        Ok(Box::new(
            Svg::parse(&data).map_err(DocumentParseError::InvalidResource)?,
        ))
    }
}

pub struct Svg {
    contours: Vec<Wire>,
}

unsafe impl Sync for Svg {}

impl Debug for Svg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Svg")
            .field("contours", &self.contours.len())
            .finish()
    }
}

impl Resource for Svg {
    fn to_instance(&self) -> Result<Value, RuntimeError> {
        let wire = match self.contours.as_slice() {
            [contour] => contour.clone(),
            contours => Wire::compound(contours)?,
        };

        Ok(Value::Plane(Rc::new(wire)))
    }
}

impl Svg {
    fn parse(data: &str) -> Result<Self, String> {
        let mut reader = XmlReader::from_str(data);
        reader.config_mut().trim_text(true);

        let mut buffer = Vec::new();
        let mut stack: Vec<Transform> = Vec::new();
        let mut transform = Transform::identity();
        let mut contours = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(element)) => {
                    transform = transform.multiply(&element_transform(&element));
                    stack.push(transform);
                    parse_element(&element, &transform, &mut contours)?;
                }
                Ok(Event::Empty(element)) => {
                    let local = transform.multiply(&element_transform(&element));
                    parse_element(&element, &local, &mut contours)?;
                }
                Ok(Event::End(_)) => {
                    stack.pop();
                    transform = stack.last().copied().unwrap_or_else(Transform::identity);
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(error) => return Err(error.to_string()),
            }

            buffer.clear();
        }

        Ok(Svg { contours })
    }
}

fn parse_element(
    element: &BytesStart,
    transform: &Transform,
    contours: &mut Vec<Wire>,
) -> Result<(), String> {
    match element.name().as_ref() {
        b"path" => {
            if let Some(data) = attribute(element, b"d") {
                contours.extend(parse_path(&data, transform)?);
            }
        }
        b"rect" => parse_rect(element, transform, contours)?,
        b"circle" => parse_circle(element, transform, contours)?,
        b"ellipse" => parse_ellipse(element, transform, contours)?,
        b"line" => parse_line(element, transform, contours)?,
        b"polyline" => parse_polygon(element, transform, contours, false)?,
        b"polygon" => parse_polygon(element, transform, contours, true)?,
        _ => {}
    }

    Ok(())
}

fn parse_rect(
    element: &BytesStart,
    transform: &Transform,
    contours: &mut Vec<Wire>,
) -> Result<(), String> {
    let x = number_attribute(element, b"x").unwrap_or(0.0);
    let y = number_attribute(element, b"y").unwrap_or(0.0);
    let width = number_attribute(element, b"width").unwrap_or(0.0);
    let height = number_attribute(element, b"height").unwrap_or(0.0);

    if width <= 0.0 || height <= 0.0 {
        return Ok(());
    }

    let mut rx = number_attribute(element, b"rx").unwrap_or(0.0).abs();
    let mut ry = number_attribute(element, b"ry").unwrap_or(0.0).abs();

    if rx > 0.0 && ry == 0.0 {
        ry = rx;
    }
    if ry > 0.0 && rx == 0.0 {
        rx = ry;
    }

    rx = rx.min(width / 2.0);
    ry = ry.min(height / 2.0);

    let path = if rx > 0.0 && ry > 0.0 {
        format!(
            "M {} {} H {} A {} {} 0 0 1 {} {} V {} A {} {} 0 0 1 {} {} H {} A {} {} 0 0 1 {} {} V {} A {} {} 0 0 1 {} {} Z",
            x + rx,
            y,
            x + width - rx,
            rx,
            ry,
            x + width,
            y + ry,
            y + height - ry,
            rx,
            ry,
            x + width - rx,
            y + height,
            x + rx,
            rx,
            ry,
            x,
            y + height - ry,
            y + ry,
            rx,
            ry,
            x + rx,
            y,
        )
    } else {
        format!("M {} {} H {} V {} H {} Z", x, y, x + width, y + height, x)
    };

    contours.extend(parse_path(&path, transform)?);
    Ok(())
}

fn parse_circle(
    element: &BytesStart,
    transform: &Transform,
    contours: &mut Vec<Wire>,
) -> Result<(), String> {
    let cx = number_attribute(element, b"cx").unwrap_or(0.0);
    let cy = number_attribute(element, b"cy").unwrap_or(0.0);
    let radius = number_attribute(element, b"r").unwrap_or(0.0).abs();

    if radius <= 0.0 {
        return Ok(());
    }

    let path = format!(
        "M {} {} A {} {} 0 1 1 {} {} A {} {} 0 1 1 {} {} Z",
        cx + radius,
        cy,
        radius,
        radius,
        cx - radius,
        cy,
        radius,
        radius,
        cx + radius,
        cy,
    );

    contours.extend(parse_path(&path, transform)?);
    Ok(())
}

fn parse_ellipse(
    element: &BytesStart,
    transform: &Transform,
    contours: &mut Vec<Wire>,
) -> Result<(), String> {
    let cx = number_attribute(element, b"cx").unwrap_or(0.0);
    let cy = number_attribute(element, b"cy").unwrap_or(0.0);
    let rx = number_attribute(element, b"rx").unwrap_or(0.0).abs();
    let ry = number_attribute(element, b"ry").unwrap_or(0.0).abs();

    if rx <= 0.0 || ry <= 0.0 {
        return Ok(());
    }

    let path = format!(
        "M {} {} A {} {} 0 1 1 {} {} A {} {} 0 1 1 {} {} Z",
        cx + rx,
        cy,
        rx,
        ry,
        cx - rx,
        cy,
        rx,
        ry,
        cx + rx,
        cy,
    );

    contours.extend(parse_path(&path, transform)?);
    Ok(())
}

fn parse_line(
    element: &BytesStart,
    transform: &Transform,
    contours: &mut Vec<Wire>,
) -> Result<(), String> {
    let x1 = number_attribute(element, b"x1").unwrap_or(0.0);
    let y1 = number_attribute(element, b"y1").unwrap_or(0.0);
    let x2 = number_attribute(element, b"x2").unwrap_or(0.0);
    let y2 = number_attribute(element, b"y2").unwrap_or(0.0);

    let path = format!("M {} {} L {} {}", x1, y1, x2, y2);

    contours.extend(parse_path(&path, transform)?);
    Ok(())
}

fn parse_polygon(
    element: &BytesStart,
    transform: &Transform,
    contours: &mut Vec<Wire>,
    close: bool,
) -> Result<(), String> {
    let Some(points) = attribute(element, b"points") else {
        return Ok(());
    };

    let mut position = 0;
    let mut coordinates = Vec::new();
    while let Some(number) = scan_number(points.as_bytes(), &mut position) {
        coordinates.push(number);
    }

    let mut path = String::new();
    for (index, point) in coordinates.chunks_exact(2).enumerate() {
        let command = if index == 0 { 'M' } else { 'L' };
        path.push_str(&format!("{command} {} {} ", point[0], point[1]));
    }

    if close {
        path.push('Z');
    }

    contours.extend(parse_path(&path, transform)?);
    Ok(())
}

fn parse_path(data: &str, transform: &Transform) -> Result<Vec<Wire>, String> {
    PathParser::new(data.as_bytes()).parse(transform)
}

struct PathParser<'a> {
    data: &'a [u8],
    position: usize,
    current: [f64; 2],
    start: [f64; 2],
    cubic_control: Option<[f64; 2]>,
    quad_control: Option<[f64; 2]>,
}

impl<'a> PathParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        PathParser {
            data,
            position: 0,
            current: [0.0, 0.0],
            start: [0.0, 0.0],
            cubic_control: None,
            quad_control: None,
        }
    }

    fn parse(mut self, transform: &Transform) -> Result<Vec<Wire>, String> {
        let mut contours = Vec::new();
        let mut edges = Vec::new();
        let mut previous: Option<u8> = None;
        let mut started = false;

        loop {
            self.skip_separators();
            if self.position >= self.data.len() {
                break;
            }

            let next = self.data[self.position];
            let command = if next.is_ascii_alphabetic() {
                self.position += 1;
                previous = Some(next);
                next
            } else if is_number_start(next) {
                match previous {
                    Some(b'M') => b'L',
                    Some(b'm') => b'l',
                    Some(command) => command,
                    None => return Err("path data must start with a move command".into()),
                }
            } else {
                return Err(format!(
                    "unexpected character '{}' in path data",
                    next as char
                ));
            };

            if !started && !matches!(command, b'M' | b'm') {
                return Err("path data must start with a move command".into());
            }

            match command {
                b'M' | b'm' => {
                    let relative = command == b'm';
                    let end = self.next_coordinate(relative)?;
                    self.finish(&mut edges, &mut contours, transform, false)?;
                    self.current = end;
                    self.start = end;
                    self.cubic_control = None;
                    self.quad_control = None;
                    started = true;
                }
                b'L' | b'l' => {
                    let relative = command == b'l';
                    let end = self.next_coordinate(relative)?;
                    self.add_line(&mut edges, end, transform)?;
                }
                b'H' | b'h' => {
                    let x = self.next_number().ok_or("expected x coordinate")?;
                    let end = if command == b'h' {
                        [self.current[0] + x, self.current[1]]
                    } else {
                        [x, self.current[1]]
                    };
                    self.add_line(&mut edges, end, transform)?;
                }
                b'V' | b'v' => {
                    let y = self.next_number().ok_or("expected y coordinate")?;
                    let end = if command == b'v' {
                        [self.current[0], self.current[1] + y]
                    } else {
                        [self.current[0], y]
                    };
                    self.add_line(&mut edges, end, transform)?;
                }
                b'C' | b'c' => {
                    let relative = command == b'c';
                    let control_1 = self.next_coordinate(relative)?;
                    let control_2 = self.next_coordinate(relative)?;
                    let end = self.next_coordinate(relative)?;
                    self.add_cubic(&mut edges, control_1, control_2, end, transform)?;
                }
                b'S' | b's' => {
                    let relative = command == b's';
                    let control_2 = self.next_coordinate(relative)?;
                    let end = self.next_coordinate(relative)?;
                    let control_1 = self
                        .cubic_control
                        .map(|control| self.reflect(control))
                        .unwrap_or(self.current);
                    self.add_cubic(&mut edges, control_1, control_2, end, transform)?;
                }
                b'Q' | b'q' => {
                    let relative = command == b'q';
                    let control = self.next_coordinate(relative)?;
                    let end = self.next_coordinate(relative)?;
                    self.add_quadratic(&mut edges, control, end, transform)?;
                }
                b'T' | b't' => {
                    let relative = command == b't';
                    let end = self.next_coordinate(relative)?;
                    let control = self
                        .quad_control
                        .map(|control| self.reflect(control))
                        .unwrap_or(self.current);
                    self.add_quadratic(&mut edges, control, end, transform)?;
                }
                b'A' | b'a' => {
                    let relative = command == b'a';
                    let rx = self.next_number().ok_or("expected x radius")?;
                    let ry = self.next_number().ok_or("expected y radius")?;
                    let rotation = self.next_number().ok_or("expected rotation")?;
                    let large_arc = self.next_flag().ok_or("expected large arc flag")?;
                    let sweep = self.next_flag().ok_or("expected sweep flag")?;
                    let end = self.next_coordinate(relative)?;
                    self.add_arc(
                        &mut edges, rx, ry, rotation, large_arc, sweep, end, transform,
                    )?;
                }
                b'Z' | b'z' => {
                    self.finish(&mut edges, &mut contours, transform, true)?;
                    self.current = self.start;
                    self.cubic_control = None;
                    self.quad_control = None;
                    previous = None;
                }
                _ => {
                    return Err(format!("unknown path command '{}'", command as char));
                }
            }
        }

        self.finish(&mut edges, &mut contours, transform, false)?;
        Ok(contours)
    }

    fn reflect(&self, control: [f64; 2]) -> [f64; 2] {
        [
            2.0 * self.current[0] - control[0],
            2.0 * self.current[1] - control[1],
        ]
    }

    fn add_line(
        &mut self,
        edges: &mut Vec<Edge>,
        end: [f64; 2],
        transform: &Transform,
    ) -> Result<(), String> {
        let start = transform.point(self.current);
        let end_point = transform.point(end);

        if start.distance(&end_point) > EPSILON {
            edges.push(Edge::new_line(&start, &end_point).map_err(|error| error.to_string())?);
        }

        self.current = end;
        self.cubic_control = None;
        self.quad_control = None;
        Ok(())
    }

    fn add_cubic(
        &mut self,
        edges: &mut Vec<Edge>,
        control_1: [f64; 2],
        control_2: [f64; 2],
        end: [f64; 2],
        transform: &Transform,
    ) -> Result<(), String> {
        let points = [
            transform.point(self.current),
            transform.point(control_1),
            transform.point(control_2),
            transform.point(end),
        ];

        edges.push(Edge::new_bezier(&points).map_err(|error| error.to_string())?);

        self.current = end;
        self.cubic_control = Some(control_2);
        self.quad_control = None;
        Ok(())
    }

    fn add_quadratic(
        &mut self,
        edges: &mut Vec<Edge>,
        control: [f64; 2],
        end: [f64; 2],
        transform: &Transform,
    ) -> Result<(), String> {
        let points = [
            transform.point(self.current),
            transform.point(control),
            transform.point(end),
        ];

        edges.push(Edge::new_bezier(&points).map_err(|error| error.to_string())?);

        self.current = end;
        self.cubic_control = None;
        self.quad_control = Some(control);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn add_arc(
        &mut self,
        edges: &mut Vec<Edge>,
        rx: f64,
        ry: f64,
        rotation: f64,
        large_arc: bool,
        sweep: bool,
        end: [f64; 2],
        transform: &Transform,
    ) -> Result<(), String> {
        if rx.abs() < EPSILON || ry.abs() < EPSILON {
            return self.add_line(edges, end, transform);
        }

        for cubic in arc_to_cubics(self.current, end, rx, ry, rotation, large_arc, sweep) {
            let points = cubic.map(|point| transform.point(point));

            if points[0].distance(&points[3]) < EPSILON {
                continue;
            }

            edges.push(Edge::new_bezier(&points).map_err(|error| error.to_string())?);
        }

        self.current = end;
        self.cubic_control = None;
        self.quad_control = None;
        Ok(())
    }

    fn finish(
        &mut self,
        edges: &mut Vec<Edge>,
        contours: &mut Vec<Wire>,
        transform: &Transform,
        close: bool,
    ) -> Result<(), String> {
        if edges.is_empty() {
            return Ok(());
        }

        let mut factory = WireFactory::new();
        for edge in edges.drain(..) {
            factory.add_edge(&edge);
        }

        if close {
            let start = transform.point(self.start);
            let end = transform.point(self.current);

            if start.distance(&end) > EPSILON {
                factory.add_edge(&Edge::new_line(&end, &start).map_err(|error| error.to_string())?);
            }
        }

        contours.push(factory.build().map_err(|error| error.to_string())?);
        Ok(())
    }

    fn skip_separators(&mut self) {
        while self.position < self.data.len() {
            let byte = self.data[self.position];
            if byte.is_ascii_whitespace() || byte == b',' {
                self.position += 1;
            } else {
                break;
            }
        }
    }

    fn next_number(&mut self) -> Option<f64> {
        scan_number(self.data, &mut self.position)
    }

    fn next_coordinate(&mut self, relative: bool) -> Result<[f64; 2], String> {
        let x = self.next_number().ok_or("expected x coordinate")?;
        let y = self.next_number().ok_or("expected y coordinate")?;

        if relative {
            Ok([self.current[0] + x, self.current[1] + y])
        } else {
            Ok([x, y])
        }
    }

    fn next_flag(&mut self) -> Option<bool> {
        self.skip_separators();

        match self.data.get(self.position) {
            Some(b'0') => {
                self.position += 1;
                Some(false)
            }
            Some(b'1') => {
                self.position += 1;
                Some(true)
            }
            _ => None,
        }
    }
}

fn arc_to_cubics(
    start: [f64; 2],
    end: [f64; 2],
    rx: f64,
    ry: f64,
    rotation: f64,
    large_arc: bool,
    sweep: bool,
) -> Vec<[[f64; 2]; 4]> {
    if start[0] == end[0] && start[1] == end[1] {
        return Vec::new();
    }

    let mut rx = rx.abs();
    let mut ry = ry.abs();

    if rx < EPSILON || ry < EPSILON {
        return vec![[start, start, end, end]];
    }

    let phi = rotation.to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();

    let half_x = (start[0] - end[0]) / 2.0;
    let half_y = (start[1] - end[1]) / 2.0;

    let x1 = cos_phi * half_x + sin_phi * half_y;
    let y1 = -sin_phi * half_x + cos_phi * half_y;

    let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lambda > 1.0 {
        let scale = lambda.sqrt();
        rx *= scale;
        ry *= scale;
    }

    let denominator = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let numerator = rx * rx * ry * ry - denominator;
    let sign = if large_arc != sweep { 1.0 } else { -1.0 };
    let coefficient = sign * (numerator / denominator).max(0.0).sqrt();

    let center_x = coefficient * rx * y1 / ry;
    let center_y = -coefficient * ry * x1 / rx;

    let cx = cos_phi * center_x - sin_phi * center_y + (start[0] + end[0]) / 2.0;
    let cy = sin_phi * center_x + cos_phi * center_y + (start[1] + end[1]) / 2.0;

    let theta_1 = ((y1 - center_y) / ry).atan2((x1 - center_x) / rx);
    let theta_2 = ((-y1 - center_y) / ry).atan2((-x1 - center_x) / rx);

    let mut delta = theta_2 - theta_1;
    if !sweep && delta > 0.0 {
        delta -= 2.0 * PI;
    } else if sweep && delta < 0.0 {
        delta += 2.0 * PI;
    }

    let segments = (delta.abs() / (PI / 2.0)).ceil().max(1.0) as usize;
    let step = delta / segments as f64;

    let map = |x: f64, y: f64| {
        let x = rx * x;
        let y = ry * y;
        [
            cx + cos_phi * x - sin_phi * y,
            cy + sin_phi * x + cos_phi * y,
        ]
    };

    let mut cubics = Vec::with_capacity(segments);
    let mut theta = theta_1;
    let mut previous = start;

    for index in 0..segments {
        let next_theta = theta + step;
        let alpha = 4.0 / 3.0 * ((next_theta - theta) / 4.0).tan();

        let (sin_start, cos_start) = theta.sin_cos();
        let (sin_end, cos_end) = next_theta.sin_cos();

        let control_1 = map(cos_start - alpha * sin_start, sin_start + alpha * cos_start);
        let control_2 = map(cos_end + alpha * sin_end, sin_end - alpha * cos_end);

        let end_point = if index == segments - 1 {
            end
        } else {
            map(cos_end, sin_end)
        };

        cubics.push([previous, control_1, control_2, end_point]);
        previous = end_point;
        theta = next_theta;
    }

    cubics
}

#[derive(Clone, Copy, Debug)]
struct Transform {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl Transform {
    fn identity() -> Self {
        Transform {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    fn multiply(&self, other: &Transform) -> Transform {
        Transform {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    fn apply(&self, point: [f64; 2]) -> [f64; 2] {
        [
            self.a * point[0] + self.c * point[1] + self.e,
            self.b * point[0] + self.d * point[1] + self.f,
        ]
    }

    fn point(&self, point: [f64; 2]) -> Point {
        let [x, y] = self.apply(point);
        Point::new_2d(x, y)
    }

    fn parse(data: &str) -> Transform {
        let bytes = data.as_bytes();
        let mut position = 0;
        let mut transform = Transform::identity();

        loop {
            while position < bytes.len() && !bytes[position].is_ascii_alphabetic() {
                position += 1;
            }

            let name_start = position;
            while position < bytes.len() && bytes[position].is_ascii_alphabetic() {
                position += 1;
            }

            if name_start == position {
                break;
            }

            let name = &data[name_start..position];

            while position < bytes.len() && bytes[position].is_ascii_whitespace() {
                position += 1;
            }

            if position >= bytes.len() || bytes[position] != b'(' {
                break;
            }
            position += 1;

            let mut arguments = Vec::new();
            loop {
                while position < bytes.len()
                    && (bytes[position].is_ascii_whitespace() || bytes[position] == b',')
                {
                    position += 1;
                }

                if position >= bytes.len() || bytes[position] == b')' {
                    position += 1;
                    break;
                }

                match scan_number(bytes, &mut position) {
                    Some(number) => arguments.push(number),
                    None => break,
                }
            }

            transform = transform.multiply(&Transform::from_function(name, &arguments));
        }

        transform
    }

    fn from_function(name: &str, arguments: &[f64]) -> Transform {
        let identity = Transform::identity();

        match (name, arguments) {
            ("matrix", [a, b, c, d, e, f]) => Transform {
                a: *a,
                b: *b,
                c: *c,
                d: *d,
                e: *e,
                f: *f,
            },
            ("translate", [x]) => Transform { e: *x, ..identity },
            ("translate", [x, y, ..]) => Transform {
                e: *x,
                f: *y,
                ..identity
            },
            ("scale", [x]) => Transform {
                a: *x,
                d: *x,
                ..identity
            },
            ("scale", [x, y, ..]) => Transform {
                a: *x,
                d: *y,
                ..identity
            },
            ("rotate", [angle]) => {
                let (sin, cos) = angle.to_radians().sin_cos();
                Transform {
                    a: cos,
                    b: sin,
                    c: -sin,
                    d: cos,
                    ..identity
                }
            }
            ("rotate", [angle, cx, cy, ..]) => {
                let (sin, cos) = angle.to_radians().sin_cos();
                Transform {
                    a: cos,
                    b: sin,
                    c: -sin,
                    d: cos,
                    e: *cx,
                    f: *cy,
                }
                .multiply(&Transform {
                    e: -cx,
                    f: -cy,
                    ..identity
                })
            }
            ("skewX", [angle, ..]) => Transform {
                c: angle.to_radians().tan(),
                ..identity
            },
            ("skewY", [angle, ..]) => Transform {
                b: angle.to_radians().tan(),
                ..identity
            },
            _ => identity,
        }
    }
}

fn element_transform(element: &BytesStart) -> Transform {
    attribute(element, b"transform")
        .map(|value| Transform::parse(&value))
        .unwrap_or_else(Transform::identity)
}

fn attribute(element: &BytesStart, name: &[u8]) -> Option<String> {
    element
        .attributes()
        .flatten()
        .find(|attribute| attribute.key.as_ref() == name)
        .and_then(|attribute| {
            attribute
                .unescape_value()
                .ok()
                .map(|value| value.into_owned())
        })
}

fn number_attribute(element: &BytesStart, name: &[u8]) -> Option<f64> {
    attribute(element, name).and_then(|value| {
        let mut position = 0;
        scan_number(value.as_bytes(), &mut position)
    })
}

fn scan_number(data: &[u8], position: &mut usize) -> Option<f64> {
    while *position < data.len() {
        let byte = data[*position];
        if byte.is_ascii_whitespace() || byte == b',' {
            *position += 1;
        } else {
            break;
        }
    }

    let start = *position;
    let mut end = start;

    if end < data.len() && (data[end] == b'+' || data[end] == b'-') {
        end += 1;
    }

    let mut digits = false;
    while end < data.len() && data[end].is_ascii_digit() {
        end += 1;
        digits = true;
    }

    if end < data.len() && data[end] == b'.' {
        end += 1;
        while end < data.len() && data[end].is_ascii_digit() {
            end += 1;
            digits = true;
        }
    }

    if !digits {
        return None;
    }

    if end < data.len() && (data[end] == b'e' || data[end] == b'E') {
        let mut exponent = end + 1;
        if exponent < data.len() && (data[exponent] == b'+' || data[exponent] == b'-') {
            exponent += 1;
        }

        let mut exponent_digits = false;
        while exponent < data.len() && data[exponent].is_ascii_digit() {
            exponent += 1;
            exponent_digits = true;
        }

        if exponent_digits {
            end = exponent;
        }
    }

    let value = std::str::from_utf8(&data[start..end]).ok()?.parse().ok()?;
    *position = end;
    Some(value)
}

fn is_number_start(byte: u8) -> bool {
    byte.is_ascii_digit() || byte == b'-' || byte == b'+' || byte == b'.'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::FsReader;
    use dslcad_occt::{Axis, DsShape, Shape};

    fn identity() -> Transform {
        Transform::identity()
    }

    #[test]
    fn it_parses_absolute_lines() {
        let contours = parse_path("M 0 0 L 10 0 L 10 10 Z", &identity()).unwrap();

        assert_eq!(1, contours.len());

        let points = contours[0].points(0.1).unwrap();
        assert_eq!(3, points.len());
        assert_eq!([0., 0., 0.], points[0][0]);
        assert_eq!([10., 10., 0.], points[1][1]);
        assert_eq!([0., 0., 0.], points[2][1]);
    }

    #[test]
    fn it_parses_relative_commands() {
        let contours = parse_path("m 5 5 l 5 0 l 0 5 z", &identity()).unwrap();

        let points = contours[0].points(0.1).unwrap();
        assert_eq!(3, points.len());
        assert_eq!([5., 5., 0.], points[0][0]);
        assert_eq!([10., 5., 0.], points[0][1]);
    }

    #[test]
    fn it_parses_horizontal_and_vertical_lines() {
        let contours = parse_path("M 0 0 H 10 V 10 h -10 v -10 Z", &identity()).unwrap();

        let points = contours[0].points(0.1).unwrap();
        assert_eq!(4, points.len());
        assert_eq!([10., 10., 0.], points[1][1]);
    }

    #[test]
    fn it_parses_multiple_subpaths() {
        let contours = parse_path("M 0 0 L 1 0 L 1 1 Z M 5 5 L 6 5 L 6 6 Z", &identity()).unwrap();

        assert_eq!(2, contours.len());
    }

    #[test]
    fn it_parses_implicit_commands() {
        let contours = parse_path("M 0 0 10 0 10 10 Z", &identity()).unwrap();

        let points = contours[0].points(0.1).unwrap();
        assert_eq!(3, points.len());
        assert_eq!([10., 0., 0.], points[0][1]);
    }

    #[test]
    fn it_applies_transforms() {
        let transform = Transform::parse("translate(10 20) scale(2)");
        let contours = parse_path("M 0 0 L 1 0 L 1 1 Z", &transform).unwrap();

        let points = contours[0].points(0.1).unwrap();
        assert_eq!([10., 20., 0.], points[0][0]);
        assert_eq!([12., 20., 0.], points[0][1]);
    }

    #[test]
    fn it_parses_rotate_transforms() {
        let transform = Transform::parse("rotate(90)");
        let [x, y] = transform.apply([1.0, 0.0]);

        assert!(x.abs() < EPSILON);
        assert!((y - 1.0).abs() < EPSILON);
    }

    #[test]
    fn it_parses_curves_and_arcs() {
        let contours = parse_path(
            "M 0 0 C 1 1 2 1 3 0 S 5 -1 6 0 A 2 2 0 0 1 10 0 Z",
            &identity(),
        )
        .unwrap();

        assert_eq!(1, contours.len());

        let points = contours[0].points(0.1).unwrap();
        assert_eq!(5, points.len());
        assert_eq!([10., 0., 0.], points[3].last().unwrap().clone());
        assert_eq!([0., 0., 0.], points[4][1]);
    }

    #[test]
    fn it_parses_quadratic_curves() {
        let contours = parse_path("M 0 0 Q 1 1 2 0 T 4 0 Z", &identity()).unwrap();

        let points = contours[0].points(0.1).unwrap();
        assert_eq!(3, points.len());
        assert_eq!([4., 0., 0.], points[1].last().unwrap().clone());
    }

    #[test]
    fn it_parses_circles() {
        let svg = Svg::parse(r#"<svg><circle cx="5" cy="5" r="3"/></svg>"#).unwrap();

        let points: Vec<_> = svg.contours[0]
            .points(0.01)
            .unwrap()
            .into_iter()
            .flatten()
            .collect();

        for point in points {
            let distance = ((point[0] - 5.).powi(2) + (point[1] - 5.).powi(2)).sqrt();
            assert!(
                (distance - 3.).abs() < 0.01,
                "{point:?} is not on the circle"
            );
        }
    }

    #[test]
    fn it_parses_arc_directions() {
        let contours = parse_path("M 0 0 A 1 1 0 0 1 1 1", &identity()).unwrap();

        let points: Vec<_> = contours[0]
            .points(0.01)
            .unwrap()
            .into_iter()
            .flatten()
            .collect();

        let expected = [0.7071067811865476, 0.2928932188134524, 0.];
        let distance = points
            .iter()
            .map(|point| {
                ((point[0] - expected[0]).powi(2) + (point[1] - expected[1]).powi(2)).sqrt()
            })
            .fold(f64::MAX, f64::min);

        assert!(distance < 0.01, "arc did not pass through {expected:?}");
    }

    #[test]
    fn it_parses_basic_shapes() {
        let svg = Svg::parse(
            r#"
            <svg>
                <rect x="1" y="2" width="4" height="6"/>
                <rect x="1" y="2" width="4" height="6" rx="1"/>
                <circle cx="0" cy="0" r="3"/>
                <ellipse cx="0" cy="0" rx="3" ry="2"/>
                <line x1="0" y1="0" x2="1" y2="1"/>
                <polyline points="0,0 1,0 1,1"/>
                <polygon points="0,0 1,0 1,1"/>
            </svg>
            "#,
        )
        .unwrap();

        assert_eq!(7, svg.contours.len());
    }

    #[test]
    fn it_parses_nested_transforms() {
        let svg = Svg::parse(
            r#"
            <svg>
                <g transform="translate(10 0)">
                    <g transform="scale(2)">
                        <path d="M 0 0 L 1 0 L 1 1 Z"/>
                    </g>
                </g>
            </svg>
            "#,
        )
        .unwrap();

        assert_eq!(1, svg.contours.len());

        let points = svg.contours[0].points(0.1).unwrap();
        assert_eq!([10., 0., 0.], points[0][0]);
        assert_eq!([12., 0., 0.], points[0][1]);
    }

    #[test]
    fn it_returns_single_contours_directly() {
        let svg = Svg::parse(r#"<svg><path d="M 0 0 L 1 0 L 1 1 Z"/></svg>"#).unwrap();
        let value = svg.to_instance().unwrap();

        match value {
            Value::Plane(line) => assert!(!line.is_compound()),
            _ => panic!("expected a plane"),
        }
    }

    #[test]
    fn it_rejects_invalid_path_data() {
        assert!(parse_path("L 0 0", &identity()).is_err());
        assert!(parse_path("M 0 0 L", &identity()).is_err());
        assert!(parse_path("M 0 0 L 1 1 Q", &identity()).is_err());
    }

    #[test]
    fn it_reports_errors() {
        assert!(SvgLoader
            .load("../../examples/svg_import/missing.svg", &FsReader)
            .is_err());
        assert!(Svg::parse(r#"<svg><path d="M 0 0 L"/></svg>"#).is_err());
    }

    #[test]
    fn it_loads_the_stamp_example() {
        let resource = SvgLoader
            .load("../../examples/svg_import/stamp-heat-mesh.svg", &FsReader)
            .unwrap();
        let value = resource.to_instance().unwrap();

        match value {
            Value::Plane(line) => {
                let contours = line.contours();
                assert_eq!(5, contours.len());

                let polygons: Vec<Vec<[f64; 3]>> = contours
                    .iter()
                    .map(|contour| {
                        contour
                            .points(0.01)
                            .unwrap()
                            .into_iter()
                            .flatten()
                            .collect()
                    })
                    .collect();

                let mut area = 0.0;
                for (index, polygon) in polygons.iter().enumerate() {
                    let depth = (0..polygons.len())
                        .filter(|other| {
                            *other != index && polygon_contains(&polygons[*other], polygon[0])
                        })
                        .count();

                    if depth % 2 == 0 {
                        area += polygon_area(polygon);
                    } else {
                        area -= polygon_area(polygon);
                    }
                }

                let shape = Shape::extrude(&line, 0., 0., 1.).unwrap();
                assert!(shape.volume() > 0.);
                assert!((shape.volume() - area).abs() / area < 0.01);
            }
            _ => panic!("expected a plane"),
        }
    }

    #[test]
    fn it_can_transform_loaded_lines() {
        let resource = SvgLoader
            .load("../../examples/svg_import/stamp-heat-mesh.svg", &FsReader)
            .unwrap();
        let value = resource.to_instance().unwrap();

        match value {
            Value::Plane(line) => {
                let translated = line.translate(&Point::new(1., 2., 3.)).unwrap();
                assert!(translated.points(0.1).is_ok());

                let rotated = line.rotate(Axis::Z, 45.).unwrap();
                assert!(rotated.points(0.1).is_ok());

                let scaled = line.scale(2.).unwrap();
                assert!(scaled.points(0.1).is_ok());

                let center = line.center_of_mass();
                assert!(center.x() > 0.);
            }
            _ => panic!("expected a plane"),
        }
    }

    fn polygon_area(points: &[[f64; 3]]) -> f64 {
        let mut area = 0.0;
        for index in 0..points.len() {
            let previous = if index == 0 {
                points.len() - 1
            } else {
                index - 1
            };
            area += points[previous][0] * points[index][1] - points[index][0] * points[previous][1];
        }
        area.abs() / 2.0
    }

    fn polygon_contains(points: &[[f64; 3]], point: [f64; 3]) -> bool {
        let mut inside = false;
        let mut previous = points.len() - 1;

        for current in 0..points.len() {
            let (x1, y1) = (points[current][0], points[current][1]);
            let (x2, y2) = (points[previous][0], points[previous][1]);

            if (y1 > point[1]) != (y2 > point[1])
                && point[0] < (x2 - x1) * (point[1] - y1) / (y2 - y1) + x1
            {
                inside = !inside;
            }

            previous = current;
        }

        inside
    }
}
