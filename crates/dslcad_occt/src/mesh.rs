#[derive(Default, Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vec3<f64>>,
    pub triangles: Vec<Vec3<usize>>,
}

impl Mesh {
    pub fn vertex_f32(&self, index: usize) -> Vec3<f32> {
        self.vertices[index].map(|c| c as f32)
    }

    pub fn triangles_with_normals(&self) -> impl Iterator<Item = (&Vec3<usize>, Vec3<f64>)> {
        self.triangles.iter().map(|tri| {
            let dir = vec_cross(
                vec_sub(self.vertices[tri[1]], self.vertices[tri[0]]),
                vec_sub(self.vertices[tri[2]], self.vertices[tri[0]]),
            );
            let normal = vec_div(dir, vec_len(dir));
            (tri, normal)
        })
    }
}

type Vec3<T> = [T; 3];

const X: usize = 0;
const Y: usize = 1;
const Z: usize = 2;

fn vec_sub(a: Vec3<f64>, b: Vec3<f64>) -> Vec3<f64> {
    [a[X] - b[X], a[Y] - b[Y], a[Z] - b[Z]]
}

fn vec_cross(a: Vec3<f64>, b: Vec3<f64>) -> Vec3<f64> {
    [
        (a[Y] * b[Z]) - (a[Z] * b[Y]),
        (a[Z] * b[X]) - (a[X] * b[Z]),
        (a[X] * b[Y]) - (a[Y] * b[X]),
    ]
}

fn vec_len(a: Vec3<f64>) -> f64 {
    f64::sqrt(a[X].powi(2) + a[Y].powi(2) + a[Z].powi(2))
}

fn vec_div(a: Vec3<f64>, v: f64) -> Vec3<f64> {
    [a[X] / v, a[Y] / v, a[Z] / v]
}
