use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Type {
    Number,
    Bool,
    Text,
    List,
    Point,
    Edge,
    Shape,
    Function,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Number => f.write_str("number"),
            Type::Bool => f.write_str("bool"),
            Type::Text => f.write_str("text"),
            Type::List => f.write_str("list"),
            Type::Point => f.write_str("point"),
            Type::Edge => f.write_str("edge"),
            Type::Shape => f.write_str("shape"),
            Type::Function => f.write_str("function"),
        }
    }
}
