mod boolean;
mod faces;
mod lists;
mod math;
mod shapes;
mod text;

use crate::runtime::{RuntimeError, Type, Value};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

type Function = dyn Fn(&HashMap<&str, Value>) -> Result<Value, RuntimeError>;

type Arguments<'a> = Vec<ArgValue<'a>>;

#[derive(Clone)]
pub struct CallSignature<'a> {
    name: &'a str,
    arguments: Arguments<'a>,
}

#[derive(Clone)]
pub enum ArgValue<'a> {
    Unnamed(Value),
    Named(&'a str, Value),
}

impl<'a> CallSignature<'a> {
    pub fn new(name: &'a str, arguments: Arguments<'a>) -> Self {
        CallSignature { name, arguments }
    }

    fn named(&self) -> impl Iterator<Item = (&'a str, &Value)> {
        self.arguments.iter().filter_map(|a| match a {
            ArgValue::Unnamed(_) => None,
            ArgValue::Named(n, v) => Some((*n, v)),
        })
    }

    fn unnamed(&self) -> impl Iterator<Item = &Value> {
        self.arguments.iter().filter_map(|a| match a {
            ArgValue::Unnamed(v) => Some(v),
            ArgValue::Named(_, _) => None,
        })
    }
}

#[derive(Clone)]
pub struct Signature {
    name: &'static str,
    arguments: IndexMap<&'static str, Access>,
    function: &'static Function,
    category: Category,
    description: &'static str,
    variadic: bool,
}

impl Signature {
    fn call_with<'b>(&self, call: &CallSignature<'b>) -> Option<HashMap<&'b str, Value>> {
        let mut map = HashMap::new();
        let mut to_cover = self.arguments.clone();

        for (name, value) in call.named() {
            if let Some((name, access)) = to_cover.swap_remove_entry(name) {
                match access {
                    Access::Required(t) => map.insert(name, value.to_type(t).ok()?),
                    Access::Optional(t) => map.insert(name, value.to_type(t).ok()?),
                    Access::RequiredAny() => map.insert(name, value.clone()),
                };
            } else if self.variadic {
                map.insert(name, value.clone());
            } else {
                return None;
            }
        }

        for value in call.unnamed() {
            let (name, access) = to_cover.shift_remove_index(0)?;
            match access {
                Access::Required(t) => map.insert(name, value.to_type(t).ok()?),
                Access::Optional(t) => map.insert(name, value.to_type(t).ok()?),
                Access::RequiredAny() => map.insert(name, value.clone()),
            };
        }

        if to_cover.values().any(|a| a.is_required()) {
            return None;
        }

        Some(map)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Access {
    Required(Type),
    Optional(Type),
    RequiredAny(),
}

impl Access {
    fn is_required(&self) -> bool {
        match self {
            Access::Required(_) => true,
            Access::Optional(_) => false,
            Access::RequiredAny() => true,
        }
    }
}

macro_rules! bind {
    ($name: ident, $func: path[$($arg_name:ident=$arg_value:ident), *], $cat: expr, $desc: literal) => {{
        Signature{
            name: stringify!($name),
            arguments: arguments!($($arg_name=$arg_value), *).into_iter().collect(),
            function: invoke!($func[$($arg_name=$arg_value), *]),
            category: $cat,
            description: $desc,
            variadic: false
        }
    }};
}

macro_rules! arguments {
    (number) => {Access::Required(Type::Number)};
    (option_number) => {Access::Optional(Type::Number)};
    (bool) => {Access::Required(Type::Bool)};
    (option_bool) => {Access::Optional(Type::Bool)};
    (text) => {Access::Required(Type::Text)};
    (any) => {Access::RequiredAny()};
    (point) => {Access::Required(Type::Point)};
    (edge) => {Access::Required(Type::Edge)};
    (shape) => {Access::Required(Type::Shape)};
    (list) => {Access::Required(Type::List)};
    ($($name: ident=$value: ident), *) => {vec![$((stringify!($name),arguments!($value))), *]};
}

macro_rules! invoke {
    ($map: ident, $name: ident=any) => {{
        $map.get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?
            .clone()
    }};
    ($map: ident, $name: ident=number) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_number()?
    }};
    ($map: ident, $name: ident=option_number) => {{
        match $map.get(stringify!($name)) {
            Some(value) => Some(value.to_number()?),
            None => None,
        }
    }};
    ($map: ident, $name: ident=bool) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_bool()?
    }};
    ($map: ident, $name: ident=option_bool) => {{
        match $map.get(stringify!($name)) {
            Some(value) => Some(value
                .to_bool()?),
            None => None,
        }
    }};
    ($map: ident, $name: ident=text) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        value
            .to_text()?
    }};
    ($map: ident, $name: ident=point) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        &value
            .to_point()?
    }};
    ($map: ident, $name: ident=shape) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        &value
            .to_shape()?
    }};
    ($map: ident, $name: ident=edge) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        &value
            .to_line()?
    }};
    ($map: ident, $name: ident=list) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        &value
            .to_list()?
    }};
    ($func: path[$($name: ident=$value: ident), *]) => {&|_a|{
        $(let $name = invoke!(_a, $name=$value);)*
        Ok($func($($name),*)?.into())
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
    Resources,
    Lists,
    Text,
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Hidden => panic!("can not display hidden category"),
            Category::Math => f.write_str("Math"),
            Category::TwoD => f.write_str("2D"),
            Category::ThreeD => f.write_str("3D"),
            Category::Resources => f.write_str("Resources"),
            Category::Lists => f.write_str("Lists"),
            Category::Text => f.write_str("Text"),
        }
    }
}

impl Library {
    pub fn contains(&self, name: &str) -> bool {
        self.lookup.contains_key(name)
    }

    fn from_signatures(signatures: Vec<Signature>) -> Self {
        let lookup = Self::build_lookup(&signatures);
        Library { signatures, lookup }
    }

    pub fn find<'b>(
        &self,
        to_call: CallSignature<'b>,
    ) -> Result<(&Function, HashMap<&'b str, Value>), RuntimeError> {
        if let Some(indices) = self.lookup.get(to_call.name) {
            let mut options = Vec::new();
            for i in indices {
                let signature = &self.signatures[*i];
                if let Some(full_args) = signature.call_with(&to_call) {
                    options.push((signature.function, full_args))
                }
            }

            if options.len() == 1 {
                return Ok(options.remove(0));
            }

            return Err(RuntimeError::CouldNotFindFunctionSignature {
                target: to_call.to_string(),
                options: indices
                    .iter()
                    .map(|i| self.signatures[*i].to_string())
                    .collect(),
            });
        } else {
            Err(RuntimeError::CouldNotFindFunction {
                name: to_call.name.to_string(),
            })
        }
    }

    fn build_lookup(signatures: &[Signature]) -> HashMap<&'static str, Vec<usize>> {
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
        let signatures = vec![
            // Math
            bind!(add, math::add[left=number, right=number], Category::Hidden, "addition"),
            bind!(subtract, math::subtract[left=number, right=number], Category::Hidden, "subtraction"),
            bind!(multiply, math::multiply[left=number, right=number], Category::Hidden, "multiplication"),
            bind!(divide, math::divide[left=number, right=number], Category::Hidden, "division"),
            bind!(modulo, math::modulo[left=number, right=number], Category::Hidden, "modulo"),
            bind!(power, math::power[left=number, right=number], Category::Hidden, "exponentiation"),
            bind!(pi, math::pi[], Category::Math, "constant pi"),
            bind!(
                rad_to_deg,
                math::rad_to_deg[radians = number],
                Category::Math,
                "convert radians to degrees"
            ),
            bind!(
                deg_to_rad,
                math::deg_to_rad[degrees = number],
                Category::Math,
                "convert degrees to radians"
            ),
            bind!(
                sin,
                math::sin_deg[degrees = number],
                Category::Math,
                "sin operation"
            ),
            bind!(
                sin,
                math::sin_rad[radians = number],
                Category::Math,
                "sin operation"
            ),
            bind!(
                cos,
                math::cos_deg[degrees = number],
                Category::Math,
                "cos operation"
            ),
            bind!(
                cos,
                math::cos_rad[radians = number],
                Category::Math,
                "cos operation"
            ),
            bind!(
                tan,
                math::tan_deg[degrees = number],
                Category::Math,
                "tan operation"
            ),
            bind!(
                tan,
                math::tan_rad[radians = number],
                Category::Math,
                "tan operation"
            ),
            bind!(less, math::less[left=number, right=number], Category::Hidden, "less than"),
            bind!(less_or_equal, math::less_or_equal[left=number, right=number], Category::Hidden, "less than or equal"),
            bind!(equals, math::equals[left=number, right=number], Category::Hidden, "equal"),
            bind!(not_equals, math::not_equals[left=number, right=number], Category::Hidden, "not equal"),
            bind!(greater, math::greater[left=number, right=number], Category::Hidden, "greater than"),
            bind!(greater_or_equal, math::greater_or_equal[left=number, right=number], Category::Hidden, "greater than or equal"),
            bind!(
                round,
                math::round[number = number],
                Category::Math,
                "round to the nearest whole number"
            ),
            bind!(
                ceil,
                math::ceil[number = number],
                Category::Math,
                "round up to a whole number"
            ),
            bind!(
                floor,
                math::floor[number = number],
                Category::Math,
                "round down to a whole number"
            ),
            bind!(
                sqrt,
                math::sqrt[number = number],
                Category::Math,
                "square root of a number"
            ),
            // Boolean
            bind!(and, boolean::and[left=bool, right=bool], Category::Hidden, "logical and"),
            bind!(or, boolean::or[left=bool, right=bool], Category::Hidden, "logical or"),
            bind!(
                not,
                boolean::not[value = bool],
                Category::Hidden,
                "logical not"
            ),
            // Text
            bind!(add, text::add[left=text, right=text], Category::Hidden, "add text"),
            bind!(
                string,
                text::string[item = any],
                Category::Text,
                "convert to text"
            ),
            Signature {
                name: "format",
                arguments: IndexMap::from([("message", Access::Required(Type::Text))]),
                function: &|args| Ok(text::format(args)?.into()),
                category: Category::Text,
                description: "format text using {my_arg} style formatting",
                variadic: true,
            },
            Signature {
                name: "formatln",
                arguments: IndexMap::from([("message", Access::Required(Type::Text))]),
                function: &|args| Ok(text::formatln(args)?.into()),
                category: Category::Text,
                description: "format text with newline",
                variadic: true,
            },
            // 2D
            bind!(point, faces::point[x=option_number, y=option_number, z=option_number], Category::TwoD, "create a new 2D point"),
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
            bind!(translate, faces::translate[shape=edge, x=option_number, y=option_number, z=option_number], Category::TwoD, "move an edge"),
            bind!(rotate, faces::rotate[shape=edge, angle=option_number], Category::TwoD, "rotate an edge"),
            bind!(rotate, faces::rotate_3d[shape=edge, x=option_number, y=option_number, z=option_number], Category::TwoD, "rotate an edge"),
            bind!(scale, faces::scale[shape=edge, scale=number], Category::TwoD, "scale an edge"),
            bind!(
                center,
                faces::center[shape = edge, x=option_bool, y=option_bool, z=option_bool],
                Category::TwoD,
                "center an edge"
            ),
            bind!(offset, faces::offset[shape=edge, distance=number], Category::TwoD, "offset an edge"),
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
            bind!(scale, shapes::scale_xyz[shape=shape, x=option_number, y=option_number, z=option_number], Category::ThreeD, "scale a shape"),
            bind!(
                center,
                shapes::center[shape = shape, x=option_bool, y=option_bool, z=option_bool],
                Category::ThreeD,
                "center a shape"
            ),
            bind!(
                slice,
                shapes::slice_2d[left = shape, right = edge],
                Category::ThreeD,
                "cut a slice out of a shape"
            ),
            bind!(
                slice,
                shapes::slice[left = shape, right = shape],
                Category::ThreeD,
                "cut a slice out of a shape"
            ),
            // Lists
            bind!(
                length,
                lists::length[list = list],
                Category::Lists,
                "get the length of a list"
            ),
            bind!(range, lists::range[start=option_number, end=number], Category::Lists, "create a list of numbers from a range"),
        ];

        Self::from_signatures(signatures)
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
- `model.data` access data of a model
- `list[5]` get the fifth item of a list
- `if a: something() else: something_else();` test a and follow one branch depending on the value

## Lists
- `[1,2,3]` make a list with three numbers
- `map MY_LIST as NAME: OPERATION` loop over every entry in MY_LIST
- `reduce MY_LIST as NAME1,NAME2: OPERATION` combine every item in MY_LIST
- `reduce MY_LIST from BASE as NAME1,NAME2: OPERATION` combine every item in MY_LIST starting from BASE

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

## Properties
- `point.{{x,y,z}}` properties of points
- `2d_value.{{center}}` properties of 2D objects
- `3d_value.{{center}}` properties of 3D objects
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
        for (i, arg) in self.arguments.iter().enumerate() {
            match arg {
                ArgValue::Unnamed(_) => write!(f, "?")?,
                ArgValue::Named(n, _) => write!(f, "{n}")?,
            }
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
            write!(f, "{name}=")?;
            match access {
                Access::Required(t) => write!(f, "{t}")?,
                Access::RequiredAny() => write!(f, "*")?,
                Access::Optional(t) => write!(f, "[{t}]")?,
            }
            if i != self.arguments.len() - 1 || self.variadic {
                write!(f, ", ")?;
            }
        }

        if self.variadic {
            write!(f, "...")?;
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
    use dslcad_occt::Point;
    use std::rc::Rc;

    #[test]
    fn it_can_create_library() {
        let _lib = Library::default();
    }

    #[test]
    fn it_can_print_library() {
        let lib = Library::default();
        println!("{lib}");
    }

    #[test]
    fn it_can_find_single_signature() {
        let lib = Library::from_signatures(vec![
            bind!(test, one[a=number, b=number], Category::Math, ""),
        ]);
        lib.find(CallSignature::new(
            "test",
            vec![
                ArgValue::Named("a", Value::Number(1.)),
                ArgValue::Named("b", Value::Number(1.)),
            ],
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
            vec![
                ArgValue::Named("a", Value::Number(1.)),
                ArgValue::Named("b", Value::Number(1.)),
                ArgValue::Named("c", Value::Number(1.)),
            ],
        ));
        assert!(res.is_err())
    }

    #[test]
    fn it_allows_unnamed_arguments() {
        let lib = Library::from_signatures(vec![
            bind!(test, one[a=number, b=number], Category::Math, ""),
        ]);
        let res = lib.find(CallSignature::new(
            "test",
            vec![
                ArgValue::Unnamed(Value::Number(1.)),
                ArgValue::Unnamed(Value::Number(1.)),
            ],
        ));
        assert!(res.is_ok())
    }

    #[test]
    fn it_allows_variadic_arguments() {
        let lib = Library::from_signatures(vec![Signature {
            name: "test",
            arguments: IndexMap::from([("a", Access::Required(Type::Number))]),
            function: &|args| Ok(text::formatln(args)?.into()),
            category: Category::Math,
            description: "format text with newline",
            variadic: true,
        }]);
        let res = lib.find(CallSignature::new(
            "test",
            vec![
                ArgValue::Named("a", Value::Number(1.)),
                ArgValue::Named("b", Value::Number(1.)),
            ],
        ));
        assert!(res.is_ok())
    }

    #[test]
    fn it_can_find_overload_signature() {
        let lib = Library::from_signatures(vec![
            bind!(test, one[a=number, b=number], Category::Math, ""),
            bind!(test, two[a=point, b=number], Category::Math, ""),
        ]);
        let (call, args) = lib
            .find(CallSignature::new(
                "test",
                vec![
                    ArgValue::Named("a", Value::Point(Rc::new(Point::default()))),
                    ArgValue::Named("b", Value::Number(1.)),
                ],
            ))
            .expect("couldnt find method");
        call(&args).expect("called wrong handler");
    }

    fn one(_a: f64, _b: f64) -> Result<Value, RuntimeError> {
        Ok(Value::Number(0.0))
    }

    fn two(_a: &Point, _b: f64) -> Result<Value, RuntimeError> {
        Ok(Value::Number(0.0))
    }
}
