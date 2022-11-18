use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Type {
    Number,
    Bool,
    Text,
    Point,
    Edge,
    Shape,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Number => f.write_str("number"),
            Type::Bool => f.write_str("bool"),
            Type::Text => f.write_str("text"),
            Type::Point => f.write_str("point"),
            Type::Edge => f.write_str("edge"),
            Type::Shape => f.write_str("shape"),
        }
    }
}
