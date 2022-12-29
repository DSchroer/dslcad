mod boolean;
mod faces;
mod math;
mod shapes;

use crate::runtime::{RuntimeError, Type, Value};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

type Function = dyn Fn(&HashMap<String, Value>) -> Result<Value, RuntimeError>;

pub struct CallSignature<'a> {
    name: &'a str,
    arguments: &'a HashMap<&'a str, Type>,
}

impl<'a> CallSignature<'a> {
    pub fn new(name: &'a str, arguments: &'a HashMap<&'a str, Type>) -> Self {
        CallSignature { name, arguments }
    }
}

#[derive(Clone)]
pub struct Signature {
    name: &'static str,
    arguments: IndexMap<&'static str, Access>,
    function: &'static Function,
    category: Category,
    description: &'static str,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Access {
    Required(Type),
    Optional(Type),
}

macro_rules! from_cascade {
    ($source: expr) => {
        $source.map_err(|e| RuntimeError::Opencascade(e))
    };
}
pub(crate) use from_cascade;

macro_rules! bind {
    ($name: ident, $func: path[$($arg_name:ident=$arg_value:ident), *], $cat: expr, $desc: literal) => {{
        Signature{
            name: stringify!($name),
            arguments: arguments!($($arg_name=$arg_value), *).into_iter().collect(),
            function: invoke!($func[$($arg_name=$arg_value), *]),
            category: $cat,
            description: $desc,
        }
    }};
}

macro_rules! arguments {
    (number) => {Access::Required(Type::Number)};
    (option_number) => {Access::Optional(Type::Number)};
    (bool) => {Access::Required(Type::Bool)};
    (point) => {Access::Required(Type::Point)};
    (edge) => {Access::Required(Type::Edge)};
    (shape) => {Access::Required(Type::Shape)};
    (list) => {Access::Required(Type::List)};
    ($($name: ident=$value: ident), *) => {vec![$((stringify!($name),arguments!($value))), *]};
}

macro_rules! invoke {
    ($map: ident, $name: ident=number) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_number()
            .ok_or(RuntimeError::UnexpectedType(value.get_type()))?
    }};
    ($map: ident, $name: ident=option_number) => {{
        match $map.get(stringify!($name)) {
            Some(value) => Some(value
                .to_number()
                .ok_or(RuntimeError::UnexpectedType(value.get_type()))?),
            None => None,
        }
    }};
    ($map: ident, $name: ident=bool) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_bool()
            .ok_or(RuntimeError::UnexpectedType(value.get_type()))?
    }};
    ($map: ident, $name: ident=point) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_point()
            .ok_or(RuntimeError::UnexpectedType(value.get_type()))?
    }};
    ($map: ident, $name: ident=shape) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_shape()
            .ok_or(RuntimeError::UnexpectedType(value.get_type()))?
    }};
    ($map: ident, $name: ident=edge) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_line()
            .ok_or(RuntimeError::UnexpectedType(value.get_type()))?
    }};
    ($map: ident, $name: ident=list) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_list()
            .ok_or(RuntimeError::UnexpectedType(value.get_type()))?
    }};
    ($func: path[$($name: ident=$value: ident), *]) => {&|_a|{
        $(let $name = invoke!(_a, $name=$value);)*
        $func($($name),*)
    }};
}

pub struct Library {
    signatures: Vec<Signature>,
    lookup: HashMap<&'static str, Vec<usize>>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum Category {
    Hidden,
    Math,
    TwoD,
    ThreeD,
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Hidden => panic!("can not display hidden category"),
            Category::Math => f.write_str("Math"),
            Category::TwoD => f.write_str("2D"),
            Category::ThreeD => f.write_str("3D"),
        }
    }
}

impl Library {
    pub fn new() -> Self {
        let signatures = vec![
            // Math
            bind!(add, math::add[left=number, right=number], Category::Hidden, "addition"),
            bind!(subtract, math::subtract[left=number, right=number], Category::Hidden, "subtraction"),
            bind!(multiply, math::multiply[left=number, right=number], Category::Hidden, "multiplication"),
            bind!(divide, math::divide[left=number, right=number], Category::Hidden, "division"),
            bind!(modulo, math::modulo[left=number, right=number], Category::Hidden, "modulo"),
            bind!(power, math::power[left=number, right=number], Category::Hidden, "exponentiation"),
            bind!(pi, math::pi[], Category::Math, "constant pi"),
            bind!(less, math::less[left=number, right=number], Category::Hidden, "less than"),
            bind!(less_or_equal, math::less_or_equal[left=number, right=number], Category::Hidden, "less than or equal"),
            bind!(equals, math::equals[left=number, right=number], Category::Hidden, "equal"),
            bind!(not_equals, math::not_equals[left=number, right=number], Category::Hidden, "not equal"),
            bind!(greater, math::greater[left=number, right=number], Category::Hidden, "greater than"),
            bind!(greater_or_equal, math::greater_or_equal[left=number, right=number], Category::Hidden, "greater than or equal"),
            // Boolean
            bind!(and, boolean::and[left=bool, right=bool], Category::Hidden, "logical and"),
            bind!(or, boolean::or[left=bool, right=bool], Category::Hidden, "logical or"),
            bind!(
                not,
                boolean::not[value = bool],
                Category::Hidden,
                "logical not"
            ),
            // 2D
            bind!(point, faces::point[x=option_number, y=option_number], Category::TwoD, "create a new 2D point"),
            bind!(line, faces::line[start=point, end=point], Category::TwoD, "create a line between two points"),
            bind!(square, faces::square[x=option_number, y=option_number], Category::TwoD, "create a square"),
            bind!(
                circle,
                faces::circle[radius = option_number],
                Category::TwoD,
                "create a circle"
            ),
            bind!(arc, faces::arc[start=point, center=point, end=point], Category::TwoD, "create an arcing line between three points"),
            bind!(union, faces::union_edge[left=edge, right=edge], Category::TwoD, "combine two edges"),
            bind!(
                face,
                faces::face[parts = list],
                Category::TwoD,
                "make a closed face from a list of points, lines and arcs"
            ),
            // 3D
            bind!(extrude, faces::extrude[shape=edge, x=option_number, y=option_number, z=option_number], Category::ThreeD, "extrude a face into a 3D shape"),
            bind!(revolve, faces::revolve[shape=edge, x=option_number, y=option_number, z=option_number], Category::ThreeD, "extrude a face into a 3D shape around an axis"),
            bind!(cube, shapes::cube[x=option_number, y=option_number, z=option_number], Category::ThreeD, "create a cube"),
            bind!(
                sphere,
                shapes::sphere[radius = option_number],
                Category::ThreeD,
                "create a sphere"
            ),
            bind!(cylinder, shapes::cylinder[radius=option_number, height=option_number], Category::ThreeD, "create a cylinder"),
            bind!(union, shapes::union_shape[left=shape, right=shape], Category::ThreeD, "combine two shapes"),
            bind!(chamfer, shapes::chamfer[shape=shape, radius=number], Category::ThreeD, "chamfer edges"),
            bind!(fillet, shapes::fillet[shape=shape, radius=number], Category::ThreeD, "fillet edges"),
            bind!(difference, shapes::difference[left=shape, right=shape], Category::ThreeD, "cut one shape out of another"),
            bind!(intersect, shapes::intersect[left=shape, right=shape], Category::ThreeD, "intersection between two shapes"),
            bind!(translate, shapes::translate[shape=shape, x=option_number, y=option_number, z=option_number], Category::ThreeD, "move a shape"),
            bind!(rotate, shapes::rotate[shape=shape, x=option_number, y=option_number, z=option_number], Category::ThreeD, "rotate a shape"),
            bind!(scale, shapes::scale[shape=shape, scale=number], Category::ThreeD, "scale a shape"),
        ];

        Self::from_signatures(signatures)
    }

    fn from_signatures(signatures: Vec<Signature>) -> Self {
        let lookup = Self::build_lookup(&signatures);
        Library { signatures, lookup }
    }

    pub fn find(&self, to_call: CallSignature) -> Result<&Function, RuntimeError> {
        if let Some(indices) = self.lookup.get(to_call.name) {
            'index: for index in indices {
                let signature = &self.signatures[*index];

                for name in to_call.arguments.keys() {
                    if !signature.arguments.contains_key(name) {
                        continue 'index;
                    }
                }

                for (name, access) in signature.arguments.iter() {
                    match access {
                        Access::Required(t) => {
                            if !to_call.arguments.contains_key(name)
                                || !to_call.arguments.get(name).unwrap().eq(t)
                            {
                                continue 'index;
                            }
                        }
                        Access::Optional(t) => {
                            if to_call.arguments.contains_key(name)
                                && !to_call.arguments.get(name).unwrap().eq(t)
                            {
                                continue 'index;
                            }
                        }
                    }
                }
                return Ok(signature.function);
            }
            return Err(RuntimeError::CouldNotFindFunctionSignature {
                target: format!("{}", to_call),
                options: indices
                    .iter()
                    .map(|i| format!("{}", self.signatures[*i]))
                    .collect(),
            });
        } else {
            Err(RuntimeError::CouldNotFindFunction {
                name: to_call.name.to_string(),
            })
        }
    }

    fn build_lookup<'a>(signatures: &'a [Signature]) -> HashMap<&'static str, Vec<usize>> {
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

impl Default for Library {
    fn default() -> Self {
        Library::new()
    }
}

impl Display for Library {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "# Cheat Sheet")?;

        write!(
            f,
            r"
## Syntax
- `var name = value;` create a variable called name that stores value
- `value;` draw the value, each script can only draw one thing
- `b(name=a)` pass a into the name parameter of function b
- `a ->name b()` pipe a into the name parameter of function b
- `./file(name=a)` run a file as if it were a function

## Operators
- `a + b` addition
- `a - b` subtraction
- `a * b` multiplication
- `a / b` division
- `a % b` modulo
- `a ^ b` power

## Logic
- `a < b` less than
- `a <= b` less than or equal
- `a == b` equal
- `a != b` not equal
- `a > b` greater than
- `a >= b` greater than or equal
- `a and b` logical and
- `a or b` logical or
- `not a` logical not
"
        )?;

        let mut to_print = self.signatures.clone();
        to_print.sort_by(|a, b| a.category.cmp(&b.category));

        let mut category: Option<Category> = None;

        for signature in &to_print {
            if signature.category == Category::Hidden {
                continue;
            }

            if category.is_none() || category.unwrap() != signature.category {
                category = Some(signature.category);
                writeln!(f)?;
                writeln!(f, "## {}", signature.category)?;
            }

            writeln!(f, "- `{}` {}", signature, signature.description)?;
        }

        Ok(())
    }
}

impl<'a> Display for CallSignature<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.name)?;
        for (i, (name, arg_type)) in self.arguments.iter().enumerate() {
            write!(f, "{}={}", name, arg_type)?;
            if i != self.arguments.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, ")")
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.name)?;
        for (i, (name, access)) in self.arguments.iter().enumerate() {
            write!(f, "{}=", name)?;
            match access {
                Access::Required(t) => write!(f, "{}", t)?,
                Access::Optional(t) => write!(f, "[{}]", t)?,
            }
            if i != self.arguments.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, ")")
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use opencascade::Point;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn it_can_create_library() {
        let _lib = Library::new();
    }

    #[test]
    fn it_can_print_library() {
        let lib = Library::new();
        println!("{}", lib);
    }

    #[test]
    fn it_can_find_single_signature() {
        let lib = Library::from_signatures(vec![
            bind!(test, one[a=number, b=number], Category::Math, ""),
        ]);
        lib.find(CallSignature::new(
            "test",
            &HashMap::from([("a", Type::Number), ("b", Type::Number)]),
        ))
        .expect("couldnt find method");
    }

    #[test]
    fn it_rejects_extra_arguments() {
        let lib = Library::from_signatures(vec![
            bind!(test, one[a=number, b=number], Category::Math, ""),
        ]);
        let res = lib.find(CallSignature::new(
            "test",
            &HashMap::from([
                ("a", Type::Number),
                ("b", Type::Number),
                ("c", Type::Number),
            ]),
        ));
        assert!(matches!(res, Err(_)))
    }

    #[test]
    fn it_can_find_overload_signature() {
        let lib = Library::from_signatures(vec![
            bind!(test, one[a=number, b=number], Category::Math, ""),
            bind!(test, two[a=point, b=number], Category::Math, ""),
        ]);
        let call = lib
            .find(CallSignature::new(
                "test",
                &HashMap::from([("a", Type::Point), ("b", Type::Number)]),
            ))
            .expect("couldnt find method");
        call(&HashMap::from([
            (
                "a".to_string(),
                Value::Point(Rc::new(RefCell::new(Point::default()))),
            ),
            ("b".to_string(), Value::Number(0.0)),
        ]))
        .expect("called wrong handler");
    }

    fn one(_a: f64, _b: f64) -> Result<Value, RuntimeError> {
        Ok(Value::Number(0.0))
    }

    fn two(_a: Rc<RefCell<Point>>, _b: f64) -> Result<Value, RuntimeError> {
        Ok(Value::Number(0.0))
    }
}
