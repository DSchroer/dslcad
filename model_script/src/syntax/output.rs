use opencascade::IndexedMesh;

#[derive(Clone)]
pub enum Output {
    Value(String),
    Figure(Vec<Vec<[f64; 3]>>),
    Shape(IndexedMesh),
}
