use opencascade::IndexedMesh;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum Output {
    Value(String),
    Figure(),
    Shape(IndexedMesh),
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Output::Value(s) => f.write_str(s),
            Output::Figure() => f.write_str("TODO: FIGURE"),
            Output::Shape(_) => f.write_str("TODO: SHAPE"),
        }
    }
}
