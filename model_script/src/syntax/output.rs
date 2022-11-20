use opencascade::IndexedMesh;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum Output {
    Value(String),
    Figure(Vec<Vec<[f64; 3]>>),
    Shape(IndexedMesh),
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Output::Value(v) => f.write_str(v),
            Output::Figure(_) => todo!(),
            Output::Shape(_) => todo!(),
        }
    }
}
