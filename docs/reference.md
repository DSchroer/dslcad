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
- `var name(type, min=0, max=100, step=1) = value;` declare a parameter with metadata for the editor panel
- `value;` draw the value; each top-level value is a separate part
- `123`, `1.5` numbers; `true` / `false` booleans; `"text"` strings (escapes `\n`, `\t`, `"`, `\`)
- `b(name=a)` pass the value of `a` as the named argument `name` of function `b`
- `b(a)` pass `a` as the first argument of `b` (positional arguments are matched in order)
- `a ->name b()` pipe `a` into the named argument `name` of function `b`
- `a -> b()` pipe `a` into the first argument of `b`
- `a.b` access property `b` of `a` (for example `point.x`, `shape.center`, `shape.volume`)
- `list[0]` get an item of a list (the index is zero-based)
- `view name(angle="front", projection="orthographic") { ... }` a named view: a camera plus the geometry and annotations it draws; views are empty by default, so call `model()` to draw the document's default model
- `model()` inside a view, the document's default model (its top level geometry and annotations); reserved only inside views
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
- `./model.step()` import a STEP file as a 3D shape
- `./model.iges()` import an IGES file as a 3D shape
- `./drawing.svg()` import an SVG drawing as a 2D line or plane
- `./font.ttf(message="hi")` import text from a TrueType/OpenType font as a 2D plane
- `./data.ini()` import an INI file as an object of text values

Resource arguments can be any expression, so `./font.ttf(message=name)` works with a variable.

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
- `2d_value.length` length of a line or the perimeter of a 2D object
- `2d_value.area` area enclosed by a 2D object
- `3d_value.center` center of a 3D object
- `3d_value.volume` volume of a 3D object
- `3d_value.area` surface area of a 3D object

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
- `ellipse(x=[number], y=[number])` create an ellipse (x and y are the semi-axes, they default to 1 and 0.5)
- `arc(start=point, center=point, end=point)` create an arcing line between three points
- `bezier(points=list)` create a bezier curve from a list of control points
- `spline(points=list)` create a spline that passes through a list of points
- `union(left=line|plane, right=line|plane)` combine two 2D shapes
- `face(parts=list)` make a closed face from a list of points, lines and arcs
- `translate(shape=line|plane, x=[number], y=[number], z=[number])` move a 2D shape
- `rotate(shape=line|plane, angle=[number])` rotate a 2D shape around the z axis by angle in degrees
- `rotate(shape=line|plane, x=[number], y=[number], z=[number])` rotate a 2D shape around the x, y and z axes by degrees
- `scale(shape=line|plane, scale=number)` scale a 2D shape
- `mirror(shape=line|plane, x=[bool], y=[bool], z=[bool])` mirror a 2D shape across the plane perpendicular to the given axis
- `normalize(shape=line|plane)` scale a 2D shape so its largest side is 1 unit long
- `center(shape=line|plane, x=[bool], y=[bool], z=[bool])` center a 2D shape on the given axes (each axis defaults to true; pass false to leave it in place)
- `offset(shape=plane, distance=number, join=[text])` expand a closed 2D shape outward by distance (join accepts arc, tangent or intersection)
- `transform(shape=line|plane, matrix=list)` transform a 2D shape with a 3x4 matrix given as a list of 12 numbers
- `fillet(shape=plane, radius=number)` round the corners of a 2D shape
- `chamfer(shape=plane, radius=number)` chamfer the corners of a 2D shape
- `simplify(shape=line|plane, tolerance=[number])` remove detail from a line or plane so it stays within tolerance of the original
- `thicken(shape=line, distance=[number], x=[number], y=[number], z=[number])` turn a line into a face by thickening it

## 3D
- `extrude(shape=plane, x=[number], y=[number], z=[number])` extrude a face into a 3D shape
- `revolve(shape=plane, x=[number], y=[number], z=[number])` revolve a face around the x, y or z axis (the value is the angle in degrees)
- `loft(sections=list)` loft through a list of 2D sections
- `sweep(profile=plane, path=line|plane)` sweep a 2D profile along a line or plane path
- `shell(shape=shape, thickness=number)` hollow a shape leaving walls of the given thickness
- `bend(shape=shape, x=[number], y=[number], z=[number])` bend a shape around the x, y and z axes (each value is an angle in degrees)
- `taper(shape=shape, x=[number], y=[number], z=[number], axis=[text])` taper the walls running along the x, y or z axis inward along the given axis (the value is the angle in degrees; the axis selects the sides and defaults to every other axis, accepting combinations like "xy")
- `simplify(shape=shape)` merge same-domain faces and edges of a shape to reduce its complexity
- `cube(x=[number], y=[number], z=[number])` create a cube or box (x, y and z default to 1)
- `sphere(radius=[number])` create a sphere (radius defaults to 0.5)
- `cylinder(radius=[number], height=[number])` create a cylinder (radius defaults to 0.5, height to 1)
- `cone(radius1=[number], radius2=[number], height=[number])` create a cone or truncated cone (radius1 defaults to 0, radius2 to 0.5, height to 1)
- `torus(radius=[number], tube=[number])` create a torus (radius defaults to 0.5, tube to 0.25)
- `mirror(shape=shape, x=[bool], y=[bool], z=[bool])` mirror a shape across the plane perpendicular to the given axis
- `union(left=shape, right=shape, glue=[text], fuzzy=[number])` combine two shapes (glue accepts off, shift or full)
- `chamfer(shape=shape, radius=number, axis=[text])` chamfer edges (pass an axis to only chamfer edges running along it)
- `fillet(shape=shape, radius=number, axis=[text])` fillet edges (pass an axis to only fillet edges running along it)
- `difference(left=shape, right=shape, glue=[text], fuzzy=[number])` cut one shape out of another (glue accepts off, shift or full)
- `intersect(left=shape, right=shape, glue=[text], fuzzy=[number])` intersection between two shapes (glue accepts off, shift or full)
- `translate(shape=shape, x=[number], y=[number], z=[number])` move a shape
- `rotate(shape=shape, x=[number], y=[number], z=[number])` rotate a shape around the x, y and z axes by degrees
- `scale(shape=shape, scale=number)` scale a shape uniformly
- `scale(shape=shape, x=[number], y=[number], z=[number])` scale a shape independently on the x, y and z axes
- `transform(shape=shape, matrix=list)` transform a shape with a 3x4 matrix given as a list of 12 numbers
- `normalize(shape=shape)` scale a shape so its largest side is 1 unit long
- `center(shape=shape, x=[bool], y=[bool], z=[bool])` center a shape on the given axes (each axis defaults to true; pass false to leave it in place)
- `slice(left=shape, right=line|plane)` cut a 2D cross-section out of a shape
- `slice(left=shape, right=shape)` cut one shape out of another
- `distance(left=shape, right=shape)` minimum distance between two shapes
- `contains(shape=shape, at=point)` whether a point is inside a shape
- `split(left=shape, right=plane)` split a shape with a plane into a list of pieces
- `defeature(shape=shape, radius=number)` remove cylindrical features with the given radius
- `hole(shape=shape, radius=number, at=point, axis=[text], depth=[number])` drill a hole through a shape at a point (depth defaults to through all)

## Drawing
- `measure(start=point, end=point)` distance between two points, in mm
- `measure(shape=shape)` the x, y and z extents of a shape's bounding box
- `convert(value=number, source=text, target=text)` convert a value between units (mm, cm, m, in, ft)
- `dimension(start=point, end=point, offset=[number], axis=[text], text=[text], units=[text], precision=[number], arrow=[text], plane=[text])` a linear dimension between two points
- `dimension(radius=number, center=point, at=[point], text=[text], units=[text], precision=[number])` a radial dimension of a radius around a center
- `dimension(angle=[number], center=point, start=point, end=point, units=[text], precision=[number])` an angular dimension between two directions (angle defaults to the measured angle)
- `label(text=text, at=point, anchor=[point], size=[number], plane=[text])` a text label with an optional leader
- `leader(text=text, at=point, to=point)` a leader line with a note
- `level(z=number, text=[text], at=[point])` an elevation datum marker
- `north(angle=[number], at=[point])` a plan north arrow
- `centerline(start=point, end=point)` a dashed center line
- `title(text=text, subtitle=[text], scale=[text])` a drawing title with an optional subtitle and scale

## Lists
- `length(list=list)` get the length of a list
- `range(start=[number], end=number)` create a list of whole numbers from start (default 0) up to but not including end

## Text
- `string(item=*)` convert a number, boolean or text to text
- `format(message=text, ...)` format text using {my_arg} style formatting
- `formatln(message=text, ...)` format text with newline
- `error(message=text)` generate an error

