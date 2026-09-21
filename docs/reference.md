# Syntax Reference

The following is a cheat sheet style reference for all operators in DSLCAD.
Please refer to the [examples](https://github.com/DSchroer/dslcad/tree/master/examples) folder for even more reference on how
to build parts.

# Cheat Sheet

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
- `{ ... }` a scope; `func { ... }` a function that takes its own arguments
- `if a: b() else: c()` branch on condition `a` (the `else` branch is required)
- statements may end at a newline instead of a semicolon, like JavaScript automatic semicolon insertion

## Collections and Loops
- `[1, 2, 3]` make a list with three numbers
- `map LIST as NAME: OPERATION` loop over every entry in LIST, collecting the results
- `reduce LIST as NAME1,NAME2: OPERATION` combine every item in LIST
- `reduce LIST from BASE as NAME1,NAME2: OPERATION` combine every item in LIST starting from BASE

## Resources
- `./part.ds(name=a)` run another script as if it were a function
- `@lib/part.ds(name=a)` run a shared script from the nearest `modules` directory (searched upward from the current file)
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

## Math
- `pi()` constant pi
- `rad_to_deg(radians=number)` convert radians to degrees
- `deg_to_rad(degrees=number)` convert degrees to radians
- `sin(degrees=number)` sine of an angle given in degrees
- `sin(radians=number)` sine of an angle given in radians
- `cos(degrees=number)` cosine of an angle given in degrees
- `cos(radians=number)` cosine of an angle given in radians
- `tan(degrees=number)` tangent of an angle given in degrees
- `tan(radians=number)` tangent of an angle given in radians
- `round(number=number)` round to the nearest whole number
- `ceil(number=number)` round up to a whole number
- `floor(number=number)` round down to a whole number
- `sqrt(number=number)` square root of a number

## 2D
- `point(x=[number], y=[number], z=[number])` create a point in 2D or 3D space (x, y and z default to 0)
- `line(start=point, end=point)` create a line between two points
- `square(x=[number], y=[number])` create a rectangle (x and y default to 1)
- `circle(radius=[number])` create a circle (radius defaults to 0.5)
- `arc(start=point, center=point, end=point)` create an arcing line between three points
- `union(left=line|plane, right=line|plane)` combine two 2D shapes
- `face(parts=list)` make a closed face from a list of points, lines and arcs
- `translate(shape=line|plane, x=[number], y=[number], z=[number])` move a 2D shape
- `rotate(shape=line|plane, angle=[number])` rotate a 2D shape around the z axis by angle in degrees
- `rotate(shape=line|plane, x=[number], y=[number], z=[number])` rotate a 2D shape around the x, y and z axes by degrees
- `scale(shape=line|plane, scale=number)` scale a 2D shape
- `normalize(shape=line|plane)` scale a 2D shape so its largest side is 1 unit long
- `center(shape=line|plane, x=[bool], y=[bool], z=[bool])` center a 2D shape on the given axes (each axis defaults to true; pass false to leave it in place)
- `offset(shape=plane, distance=number)` expand a closed 2D shape outward by distance
- `simplify(shape=line|plane, tolerance=[number])` remove detail from a line or plane so it stays within tolerance of the original
- `thicken(shape=line, distance=[number], x=[number], y=[number], z=[number])` turn a line into a face by thickening it

## 3D
- `extrude(shape=plane, x=[number], y=[number], z=[number])` extrude a face into a 3D shape
- `revolve(shape=plane, x=[number], y=[number], z=[number])` revolve a face around the x, y or z axis (the value is the angle in degrees)
- `bend(shape=shape, x=[number], y=[number], z=[number])` bend a shape around the x, y and z axes (each value is an angle in degrees)
- `simplify(shape=shape)` merge same-domain faces and edges of a shape to reduce its complexity
- `cube(x=[number], y=[number], z=[number])` create a cube or box (x, y and z default to 1)
- `sphere(radius=[number])` create a sphere (radius defaults to 0.5)
- `cylinder(radius=[number], height=[number])` create a cylinder (radius defaults to 0.5, height to 1)
- `union(left=shape, right=shape)` combine two shapes
- `chamfer(shape=shape, radius=number)` chamfer edges
- `fillet(shape=shape, radius=number)` fillet edges
- `difference(left=shape, right=shape)` cut one shape out of another
- `intersect(left=shape, right=shape)` intersection between two shapes
- `translate(shape=shape, x=[number], y=[number], z=[number])` move a shape
- `rotate(shape=shape, x=[number], y=[number], z=[number])` rotate a shape around the x, y and z axes by degrees
- `scale(shape=shape, scale=number)` scale a shape uniformly
- `scale(shape=shape, x=[number], y=[number], z=[number])` scale a shape independently on the x, y and z axes
- `normalize(shape=shape)` scale a shape so its largest side is 1 unit long
- `center(shape=shape, x=[bool], y=[bool], z=[bool])` center a shape on the given axes (each axis defaults to true; pass false to leave it in place)
- `slice(left=shape, right=line|plane)` cut a 2D cross-section out of a shape
- `slice(left=shape, right=shape)` cut one shape out of another

## Lists
- `length(list=list)` get the length of a list
- `range(start=[number], end=number)` create a list of whole numbers from start (default 0) up to but not including end

## Text
- `string(item=*)` convert a number, boolean or text to text
- `format(message=text, ...)` format text using {my_arg} style formatting
- `formatln(message=text, ...)` format text with newline
- `error(message=text)` generate an error

