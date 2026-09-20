mod boolean;
mod faces;
mod lists;
mod math;
mod shapes;
mod text;
mod utils;

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
                    Access::Required2d() => map.insert(name, value.to_2d().ok()?),
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
                Access::Required2d() => map.insert(name, value.to_2d().ok()?),
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
    Required2d(),
    RequiredAny(),
}

impl Access {
    fn is_required(&self) -> bool {
        match self {
            Access::Required(_) => true,
            Access::Optional(_) => false,
            Access::Required2d() => true,
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
    (line) => {Access::Required(Type::Line)};
    (plane) => {Access::Required(Type::Plane)};
    (shape2d) => {Access::Required2d()};
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
    ($map: ident, $name: ident=line) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        &value.to_line()?
    }};
    ($map: ident, $name: ident=plane) => {{
        let value = $map
            .get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?;
        &value.to_plane()?
    }};
    ($map: ident, $name: ident=shape2d) => {{
        $map.get(stringify!($name))
            .ok_or(RuntimeError::UnsetParameter(String::from(stringify!($name))))?
            .clone()
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

            Err(RuntimeError::CouldNotFindFunctionSignature {
                target: to_call.to_string(),
                options: indices
                    .iter()
                    .map(|i| self.signatures[*i].to_string())
                    .collect(),
            })
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
                "sine of an angle given in degrees"
            ),
            bind!(
                sin,
                math::sin_rad[radians = number],
                Category::Math,
                "sine of an angle given in radians"
            ),
            bind!(
                cos,
                math::cos_deg[degrees = number],
                Category::Math,
                "cosine of an angle given in degrees"
            ),
            bind!(
                cos,
                math::cos_rad[radians = number],
                Category::Math,
                "cosine of an angle given in radians"
            ),
            bind!(
                tan,
                math::tan_deg[degrees = number],
                Category::Math,
                "tangent of an angle given in degrees"
            ),
            bind!(
                tan,
                math::tan_rad[radians = number],
                Category::Math,
                "tangent of an angle given in radians"
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
                "convert a number, boolean or text to text"
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
            bind!(
                error,
                utils::error[message = text],
                Category::Text,
                "generate an error"
            ),
            // 2D
            bind!(point, faces::point[x=option_number, y=option_number, z=option_number], Category::TwoD, "create a point in 2D or 3D space (x, y and z default to 0)"),
            bind!(line, faces::line[start=point, end=point], Category::TwoD, "create a line between two points"),
            bind!(square, faces::square[x=option_number, y=option_number], Category::TwoD, "create a rectangle (x and y default to 1)"),
            bind!(
                circle,
                faces::circle[radius = option_number],
                Category::TwoD,
                "create a circle (radius defaults to 0.5)"
            ),
            bind!(arc, faces::arc[start=point, center=point, end=point], Category::TwoD, "create an arcing line between three points"),
            bind!(union, faces::union_edge[left=shape2d, right=shape2d], Category::TwoD, "combine two 2D shapes"),
            bind!(
                face,
                faces::face[parts = list],
                Category::TwoD,
                "make a closed face from a list of points, lines and arcs"
            ),
            bind!(translate, faces::translate[shape=shape2d, x=option_number, y=option_number, z=option_number], Category::TwoD, "move a 2D shape"),
            bind!(rotate, faces::rotate[shape=shape2d, angle=option_number], Category::TwoD, "rotate a 2D shape around the z axis by angle in degrees"),
            bind!(rotate, faces::rotate_3d[shape=shape2d, x=option_number, y=option_number, z=option_number], Category::TwoD, "rotate a 2D shape around the x, y and z axes by degrees"),
            bind!(scale, faces::scale[shape=shape2d, scale=number], Category::TwoD, "scale a 2D shape"),
            bind!(
                normalize,
                faces::normalize[shape = shape2d],
                Category::TwoD,
                "scale a 2D shape so its largest side is 1 unit long"
            ),
            bind!(
                center,
                faces::center[shape = shape2d, x=option_bool, y=option_bool, z=option_bool],
                Category::TwoD,
                "center a 2D shape on the given axes (each axis defaults to true; pass false to leave it in place)"
            ),
            bind!(offset, faces::offset[shape=plane, distance=number], Category::TwoD, "expand a closed 2D shape outward by distance"),
            bind!(simplify, faces::simplify[shape=shape2d, tolerance=option_number], Category::TwoD, "remove detail from a line or plane so it stays within tolerance of the original"),
            bind!(thicken, faces::thicken[shape=line, distance=option_number, x=option_number, y=option_number, z=option_number], Category::TwoD, "turn a line into a face by thickening it"),
            // 3D
            bind!(extrude, faces::extrude[shape=plane, x=option_number, y=option_number, z=option_number], Category::ThreeD, "extrude a face into a 3D shape"),
            bind!(revolve, faces::revolve[shape=plane, x=option_number, y=option_number, z=option_number], Category::ThreeD, "revolve a face around the x, y or z axis (the value is the angle in degrees)"),
            bind!(bend, shapes::bend[shape=shape, x=option_number, y=option_number, z=option_number], Category::ThreeD, "bend a shape around the x, y and z axes (each value is an angle in degrees)"),
            bind!(
                simplify,
                shapes::simplify[shape = shape],
                Category::ThreeD,
                "merge same-domain faces and edges of a shape to reduce its complexity"
            ),
            bind!(cube, shapes::cube[x=option_number, y=option_number, z=option_number], Category::ThreeD, "create a cube or box (x, y and z default to 1)"),
            bind!(
                sphere,
                shapes::sphere[radius = option_number],
                Category::ThreeD,
                "create a sphere (radius defaults to 0.5)"
            ),
            bind!(cylinder, shapes::cylinder[radius=option_number, height=option_number], Category::ThreeD, "create a cylinder (radius defaults to 0.5, height to 1)"),
            bind!(union, shapes::union_shape[left=shape, right=shape], Category::ThreeD, "combine two shapes"),
            bind!(chamfer, shapes::chamfer[shape=shape, radius=number], Category::ThreeD, "chamfer edges"),
            bind!(fillet, shapes::fillet[shape=shape, radius=number], Category::ThreeD, "fillet edges"),
            bind!(difference, shapes::difference[left=shape, right=shape], Category::ThreeD, "cut one shape out of another"),
            bind!(intersect, shapes::intersect[left=shape, right=shape], Category::ThreeD, "intersection between two shapes"),
            bind!(translate, shapes::translate[shape=shape, x=option_number, y=option_number, z=option_number], Category::ThreeD, "move a shape"),
            bind!(rotate, shapes::rotate[shape=shape, x=option_number, y=option_number, z=option_number], Category::ThreeD, "rotate a shape around the x, y and z axes by degrees"),
            bind!(scale, shapes::scale[shape=shape, scale=number], Category::ThreeD, "scale a shape uniformly"),
            bind!(scale, shapes::scale_xyz[shape=shape, x=option_number, y=option_number, z=option_number], Category::ThreeD, "scale a shape independently on the x, y and z axes"),
            bind!(
                normalize,
                shapes::normalize[shape = shape],
                Category::ThreeD,
                "scale a shape so its largest side is 1 unit long"
            ),
            bind!(
                center,
                shapes::center[shape = shape, x=option_bool, y=option_bool, z=option_bool],
                Category::ThreeD,
                "center a shape on the given axes (each axis defaults to true; pass false to leave it in place)"
            ),
            bind!(
                slice,
                shapes::slice_2d[left = shape, right = shape2d],
                Category::ThreeD,
                "cut a 2D cross-section out of a shape"
            ),
            bind!(
                slice,
                shapes::slice[left = shape, right = shape],
                Category::ThreeD,
                "cut one shape out of another"
            ),
            // Lists
            bind!(
                length,
                lists::length[list = list],
                Category::Lists,
                "get the length of a list"
            ),
            bind!(range, lists::range[start=option_number, end=number], Category::Lists, "create a list of whole numbers from start (default 0) up to but not including end"),
        ];

        Self::from_signatures(signatures)
    }
}

impl Display for Library {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "# Cheat Sheet")?;

        write!(
            f,
            r#"
## Getting Started
- `cube();` draw a 1x1x1 cube
- `square(x=10, y=5) -> extrude(z=2);` extrude a 2D shape into 3D
- `cube() -> translate(z=2) -> difference(sphere());` move a shape and cut a sphere out of it
- run `dslcad ./part.ds` to write `part.3mf`, or `dslcad ./part.ds --preview` to edit live

Coordinates are in millimetres.

## Syntax
- `// text` a comment to the end of the line
- `var name = value;` declare a variable called name that stores value
- `name = value;` reassign an existing variable
- `var name;` declare a parameter, set from the CLI with `--argument name=value`
- `value;` draw the value; each top-level value is a separate part
- `123`, `1.5` numbers; `true` / `false` booleans; `"text"` strings (escapes `\n`, `\t`, `"`, `\`)
- `b(name=a)` pass the value of `a` as the named argument `name` of function `b`
- `b(a)` pass `a` as the first argument of `b` (positional arguments are matched in order)
- `a ->name b()` pipe `a` into the named argument `name` of function `b`
- `a -> b()` pipe `a` into the first argument of `b`
- `a.b` access property `b` of `a` (for example `point.x`, `shape.center`, `shape.volume`)
- `list[0]` get an item of a list (the index is zero-based)
- `{{ ... }}` a scope; `func {{ ... }}` a function that takes its own arguments
- `if a: b() else: c()` branch on condition `a` (the `else` branch is required)
- statements may end at a newline instead of a semicolon, like JavaScript automatic semicolon insertion

## Collections and Loops
- `[1, 2, 3]` make a list with three numbers
- `map LIST as NAME: OPERATION` loop over every entry in LIST, collecting the results
- `reduce LIST as NAME1,NAME2: OPERATION` combine every item in LIST
- `reduce LIST from BASE as NAME1,NAME2: OPERATION` combine every item in LIST starting from BASE

## Resources
- `./part.ds(name=a)` run another script as if it were a function
- `./model.stl()` import an STL mesh as a 3D shape
- `./drawing.svg()` import an SVG drawing as a 2D line or plane
- `./data.ini()` import an INI file as an object of text values

## Operators
- `a + b` addition; concatenates text when both sides are text
- `a - b` subtraction
- `a * b` multiplication
- `a / b` division
- `a % b` modulo
- `a ^ b` power
- `-a` negate a number

## Logic
- `a < b` less than (numbers only)
- `a <= b` less than or equal (numbers only)
- `a == b` equal (numbers only)
- `a != b` not equal (numbers only)
- `a > b` greater than (numbers only)
- `a >= b` greater than or equal (numbers only)
- `a and b` logical and
- `a or b` logical or
- `not a` logical not

## Properties
- `point.x`, `point.y`, `point.z` coordinates of a point
- `2d_value.center` center of a 2D object
- `3d_value.center` center of a 3D object
- `3d_value.volume` volume of a 3D object

## Signature Notation
- `name(type)` required argument of the given type
- `name=[type]` optional argument (values in the description are the defaults)
- `name=*` argument of any type
- `...` any number of additional named arguments
- `line|plane` accepts either type
"#
        )?;

        let mut to_print = self.signatures.clone();
        to_print.sort_by_key(|a| a.category);

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

impl Display for CallSignature<'_> {
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
                Access::Required2d() => write!(f, "line|plane")?,
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
