# Syntax Reference

The following is a cheat sheet style reference for all operators in DSLCAD.
Please refer to the [examples](https://github.com/DSchroer/dslcad/tree/master/examples) folder for even more reference on how
to build parts.

# Cheat Sheet

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
- `point.{x,y,z}` properties of points
- `2d_value.{center}` properties of 2D objects
- `3d_value.{center}` properties of 3D objects

## Math
- `pi()` constant pi
- `rad_to_deg([radians]=number)` convert radians to degrees
- `deg_to_rad([degrees]=number)` convert degrees to radians
- `sin([degrees]=number)` sin operation
- `sin([radians]=number)` sin operation
- `cos([degrees]=number)` cos operation
- `cos([radians]=number)` cos operation
- `tan([degrees]=number)` tan operation
- `tan([radians]=number)` tan operation
- `round([number]=number)` round to the nearest whole number
- `ceil([number]=number)` round up to a whole number
- `floor([number]=number)` round down to a whole number
- `sqrt([number]=number)` square root of a number

## 2D
- `point([x]=[number], y=[number], z=[number])` create a new 2D point
- `line([start]=point, end=point)` create a line between two points
- `square([x]=[number], y=[number])` create a square
- `circle([radius]=[number])` create a circle
- `arc([start]=point, center=point, end=point)` create an arcing line between three points
- `union([left]=edge, right=edge)` combine two edges
- `face([parts]=list)` make a closed face from a list of points, lines and arcs
- `translate([shape]=edge, x=[number], y=[number], z=[number])` move an edge
- `rotate([shape]=edge, angle=[number])` rotate an edge
- `rotate([shape]=edge, x=[number], y=[number], z=[number])` rotate an edge
- `scale([shape]=edge, scale=number)` scale an edge
- `center([shape]=edge, x=[bool], y=[bool], z=[bool])` center an edge
- `offset([shape]=edge, distance=number)` offset an edge

## 3D
- `extrude([shape]=edge, x=[number], y=[number], z=[number])` extrude a face into a 3D shape
- `revolve([shape]=edge, x=[number], y=[number], z=[number])` extrude a face into a 3D shape around an axis
- `cube([x]=[number], y=[number], z=[number])` create a cube
- `sphere([radius]=[number])` create a sphere
- `cylinder([radius]=[number], height=[number])` create a cylinder
- `union([left]=shape, right=shape)` combine two shapes
- `chamfer([shape]=shape, radius=number)` chamfer edges
- `fillet([shape]=shape, radius=number)` fillet edges
- `difference([left]=shape, right=shape)` cut one shape out of another
- `intersect([left]=shape, right=shape)` intersection between two shapes
- `translate([shape]=shape, x=[number], y=[number], z=[number])` move a shape
- `rotate([shape]=shape, x=[number], y=[number], z=[number])` rotate a shape
- `scale([shape]=shape, scale=number)` scale a shape
- `center([shape]=shape, x=[bool], y=[bool], z=[bool])` center a shape
- `slice([left]=shape, right=edge)` cut a slice out of a shape
- `slice([left]=shape, right=shape)` cut a slice out of a shape

## Lists
- `length([list]=list)` get the length of a list
- `range([start]=[number], end=number)` create a list of numbers from a range

## Text
- `string([item]=*)` convert to text
- `format([message]=text, ...)` format text using {my_arg} style formatting
- `formatln([message]=text, ...)` format text with newline
