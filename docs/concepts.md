# Concepts

DSLCAD has some main concepts to understand when modeling. This page does over them at a high level and explains their intended use.
For a more detailed list of operations check out the [Syntax Reference](reference.md) page.

## Shapes

When building anything in DSLCAD you need to start somewhere. Normally this is done by starting with a 2D or 3D shape. 

### 2D Shapes

The basic 2D shapes are made using the `square` and `circle` functions.

<div class="tryme">

```
square(x=5, y=7);
```

</div>

<div class="tryme">

```
circle(radius=7);
```

</div>

### 3D Shapes

The basic 3D shapes are made using the `cube`, `sphere` and `cylinder` functions.

<div class="tryme">

```
cube(x=1, y=3, z=5);
```

</div>

<div class="tryme">

```
sphere(radius=4);
```

</div>

<div class="tryme">

```
cylinder(radius=2, height=4);
```

</div>

## Operations

### Moving Objects

Often you need to position objects in 3D space. 
The `translate` function lets you move an object.
The `rotate` function lets you rotate an object around the x, y or z axis.
The `scale` function lets scale an object up or down.


<div class="tryme">

```
// Cube moved up the z axix by 2 units
cube() -> translate(z=2);

// Cube rotated 225 degrees
cube() -> rotate(z=180 + 45);

// Cube doubled in size
cube() -> scale(scale=2);
```

</div>

The `center` function lets you center any object on any axis you want.

<div class="tryme">

```
// Cube centered on the x and y axis.
cube() -> center(z=false);
```

</div>


### Boolean Operations (CSG)

To manipulate 3D objects DSLCAD uses boolean operations (AKA [Constructive Solid Geometry](https://en.wikipedia.org/wiki/Constructive_solid_geometry)).

You can combine two objects using the `union` function. This joins both object into a single object.

<div class="tryme">

```
// Cube combined with a sphere
cube() -> union(sphere(radius=1));
```

</div>

You can cut away at an object using the `difference` function. This removes the second object from the first object.

<div class="tryme">

```
// Cube with a sphere cut out
cube() -> difference(sphere(radius=1));
```

</div>

You can get the overlap using the `intersect` function. This leaves the overlapping parts of both objects.

<div class="tryme">

```
// Overlap of a cube and sphere
cube() -> intersect(sphere(radius=1));
```

</div>

### 2D to 3D

2D objects can be converted to 3D objects using the `extrude` or `revolve` functions.

<div class="tryme">

```
// Extrude a square into a rectangular cube
square() -> extrude(z=2);
```

</div>

<div class="tryme">

```
// Revolve a square around the y axis to make a half moon
square() -> revolve(y=180);
```

</div>

### 3D to 2D

Sometimes you need a 2D outline of a 3D part, use the `slice` function to cut a cross-section.

<div class="tryme">

```
// Cut the outline of a complex shape
cube() 
-> union(sphere() -> translate(y=0.9)) 
-> center(x=false, y=false)
-> slice(square(x=10, y=10));
```

</div>

## Parts

Simple projects can get away with single 3D models. For more complex projects
it is often useful to break things into multiple parts.

You can make multiple parts in the same script. Just separate each part into its
own statement like so:
```
./part1();

./part2();
```

If you wanted them to be a single part you can join them with a `union` to form 
a single object.

```
./part1() ->left union(right=./part2());
```

## Workflow

DSLCAD is designed with an opinionated workflow in mind. Following this will 
help you build high quality parts quickly. 

For every part that you want to build:

1. Start with 2D sections of it. Currently, the `face` function is the simplest
way to create any 2D sketch. 

2. Extrude your face into a 3D section using `extrude` or `revolve` to bring your
face into the 3D world. 

3. Combine 3D sections using `union`, `difference` and `intersect` to cut out the 
final dimensions of your part. 

4. [OPTIONAL] Use `chamfer` or `fillet` to add a polished look to the part. 

