mod faces;
mod math;
mod shapes;

use crate::runtime::RuntimeError;
use crate::syntax::{Type, Value};
use std::collections::{HashMap};

type Function = dyn Fn(&HashMap<String, Value>) -> Result<Value, RuntimeError>;

pub struct Signature {
    name: &'static str,
    arguments: HashMap<&'static str, Access>,
    function: &'static Function,
    description: &'static str
}

#[derive(Debug, PartialEq)]
enum Access{
    Required(Type),
    Optional(Type)
}

macro_rules! bind {
    ($name: ident, $func: path[$($arg_name:ident=$arg_value:ident), *], $desc: literal) => {{
        Signature{
            name: stringify!($name),
            arguments: arguments!($($arg_name=$arg_value), *).into_iter().collect(),
            function: invoke!($func[$($arg_name=$arg_value), *]),
            description: $desc,
        }
    }};
}

macro_rules! arguments {
    (number) => {Access::Required(Type::Number)};
    (option_number) => {Access::Optional(Type::Number)};
    (point) => {Access::Required(Type::Point)};
    (edge) => {Access::Required(Type::Edge)};
    (shape) => {Access::Required(Type::Shape)};
    ($($name: ident=$value: ident), *) => {vec![$((stringify!($name),arguments!($value))), *]};
}

macro_rules! invoke {
    ($map: ident, $name: ident=number) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_number()
            .ok_or(RuntimeError::UnexpectedType(value.clone()))?
    }};
    ($map: ident, $name: ident=option_number) => {{
        match $map.get(stringify!($name)) {
            Some(value) => Some(value
                .to_number()
                .ok_or(RuntimeError::UnexpectedType(value.clone()))?),
            None => None,
        }
    }};
    ($map: ident, $name: ident=point) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_point()
            .ok_or(RuntimeError::UnexpectedType(value.clone()))?
    }};
    ($map: ident, $name: ident=shape) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_shape()
            .ok_or(RuntimeError::UnexpectedType(value.clone()))?
    }};
    ($map: ident, $name: ident=edge) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_line()
            .ok_or(RuntimeError::UnexpectedType(value.clone()))?
    }};
    ($func: path[$name:ident=$value:ident, $($name1: ident=$value1: ident), *]) => {&|a|{
        let $name = invoke!(a, $name=$value);
        $(let $name1 = invoke!(a, $name1=$value1);)*
        $func($name, $($name1),*)
    }};
}

pub struct Library{
    signatures: Vec<Signature>,
    lookup: HashMap<&'static str, Vec<usize>>
}

impl Library {
    pub fn new() -> Self {
        let signatures = vec![
            bind!(add, math::add[left=number, right=number], "add numbers"),
            bind!(subtract, math::subtract[left=number, right=number], "subtract numbers"),
            bind!(multiply, math::multiply[left=number, right=number], "multiply numbers"),
            bind!(divide, math::divide[left=number, right=number], "divide numbers"),

            bind!(point, faces::point[x=option_number, y=option_number, z=option_number], "create a new 2D point"),
            bind!(line, faces::line[start=point, end=point], "create a line between two points"),
            bind!(arc, faces::arc[start=point, center=point, end=point], "create an arcing line between three points"),
            bind!(extrude, faces::extrude[shape=edge, x=option_number, y=option_number, z=option_number], "extrude a face into a 3D shape"),
            bind!(revolve, faces::revolve[shape=edge, x=option_number, y=option_number, z=option_number], "extrude a face into a 3D shape around an axis"),
            bind!(union, faces::union_edge[left=edge, right=edge], "combine two edges"),

            bind!(cube, shapes::cube[x=option_number, y=option_number, z=option_number], "create a cube"),
            bind!(cylinder, shapes::cylinder[radius=option_number, height=option_number], "create a cylinder"),
            bind!(union, shapes::union_shape[left=shape, right=shape], "combine two shapes"),
            bind!(chamfer, shapes::chamfer[shape=shape, radius=number], "chamfer edges"),
            bind!(fillet, shapes::fillet[shape=shape, radius=number], "fillet edges"),
            bind!(fillet, shapes::fillet[shape=shape, radius=number], "fillet edges"),
            bind!(difference, shapes::difference[left=shape, right=shape], "cut one shape out of another"),
            bind!(translate, shapes::translate[shape=shape, x=option_number, y=option_number, z=option_number], "move a shape"),
            bind!(rotate, shapes::rotate[shape=shape, x=option_number, y=option_number, z=option_number], "rotate a shape"),
            bind!(scale, shapes::scale[shape=shape, scale=number], "scale a shape")
        ];

        Self::from_signatures(signatures)
    }

    fn from_signatures(signatures: Vec<Signature>) -> Self {
        let lookup = Self::build_lookup(&signatures);
        Library{
            signatures,
            lookup
        }
    }

    pub fn find(&self, name: &str, arguments: &HashMap<&str, Type>) -> Option<&Function> {
        if let Some(indices) = self.lookup.get(name) {
            'index: for index in indices {
                let signature = &self.signatures[*index];
                for (name, access) in signature.arguments.iter() {
                    match access {
                        Access::Required(t) => {
                            if !arguments.contains_key(name) || !arguments.get(name).unwrap().eq(t) {
                                continue 'index;
                            }
                        }
                        Access::Optional(t) => {
                            if arguments.contains_key(name) && !arguments.get(name).unwrap().eq(t) {
                                continue 'index;
                            }
                        }
                    }
                }
                return Some(signature.function);
            }
        }
        None
    }

    fn build_lookup<'a>(signatures: &'a Vec<Signature>) -> HashMap<&'static str, Vec<usize>> {
        let mut lookup: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, sig) in signatures.iter().enumerate() {
            if lookup.contains_key(sig.name) {
                lookup.get_mut(sig.name).unwrap().push(i);
            } else {
                lookup.insert(sig.name, vec![i]);
            }
        }
        lookup
    }
}

#[cfg(test)]
pub mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use opencascade::Point;
    use super::*;

    #[test]
    fn it_can_create_library() {
        let _lib = Library::new();
    }

    #[test]
    fn it_can_find_single_signature() {
        let lib = Library::from_signatures(vec![
           bind!(test, one[a=number, b=number], "")
        ]);
        lib.find("test", &HashMap::from([("a", Type::Number), ("b", Type::Number)])).expect("couldnt find method");
    }

    #[test]
    fn it_can_find_overload_signature() {
        let lib = Library::from_signatures(vec![
            bind!(test, one[a=number, b=number], ""),
            bind!(test, two[a=point, b=number], "")
        ]);
        let call = lib.find("test", &HashMap::from([("a", Type::Point), ("b", Type::Number)])).expect("couldnt find method");
        call(&HashMap::from([("a".to_string(), Value::Point(Rc::new(RefCell::new(Point::default())))), ("b".to_string(), Value::Number(0.0))]))
            .expect("called wrong handler");
    }

    fn one(_a: f64, _b: f64) -> Result<Value, RuntimeError> {
        Ok(Value::Number(0.0))
    }

    fn two(_a: Rc<RefCell<Point>>, _b: f64) -> Result<Value, RuntimeError> {
        Ok(Value::Number(0.0))
    }
}