use crate::protocol::{Part, Point, Render};

#[derive(Debug, Clone)]
pub struct BoundingBox {
    x_range: (f64, f64),
    y_range: (f64, f64),
    z_range: (f64, f64),
}

impl Default for BoundingBox {
    fn default() -> Self {
        BoundingBox {
            x_range: (f64::MAX, f64::MIN),
            y_range: (f64::MAX, f64::MIN),
            z_range: (f64::MAX, f64::MIN),
        }
    }
}

impl BoundingBox {
    pub fn center(&self) -> Point {
        [
            (self.x_range.0 + self.x_range.1) / 2.,
            (self.y_range.0 + self.y_range.1) / 2.,
            (self.z_range.0 + self.z_range.1) / 2.,
        ]
    }

    pub fn x_len(&self) -> f64 {
        self.x_range.1 - self.x_range.0
    }

    pub fn y_len(&self) -> f64 {
        self.y_range.1 - self.y_range.0
    }

    pub fn z_len(&self) -> f64 {
        self.z_range.1 - self.z_range.0
    }

    pub fn max_len(&self) -> f64 {
        f64::max(self.x_len(), f64::max(self.y_len(), self.z_len()))
    }

    fn update_from_points<'a>(&mut self, points: impl Iterator<Item = &'a Point>) {
        for p in points {
            self.x_range = (
                f64::min(self.x_range.0, p[0]),
                f64::max(self.x_range.1, p[0]),
            );
            self.y_range = (
                f64::min(self.y_range.0, p[1]),
                f64::max(self.y_range.1, p[1]),
            );
            self.z_range = (
                f64::min(self.z_range.0, p[2]),
                f64::max(self.z_range.1, p[2]),
            );
        }
    }
}

impl Render {
    pub fn aabb(&self) -> BoundingBox {
        let mut aabb = BoundingBox::default();

        for part in &self.parts {
            match part {
                Part::Empty => {}
                Part::Planar { points, lines } => {
                    aabb.update_from_points(points.iter());
                    aabb.update_from_points(lines.iter().flatten());
                }
                Part::Object { mesh, .. } => aabb.update_from_points(mesh.vertices.iter()),
            }
        }

        aabb
    }
}
