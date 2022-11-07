var width = 10;
var length = 10;
var height = 5;

var leg = cube(width=2, length=2, height=height);
var base = cube(width=width, length=length, height=1);

base
    ->right union(left=leg)
    ->right union(left=leg ->shape translate(x=0, y=width-2))
    ->right union(left=leg ->shape translate(x=length-2, y=width-2))
    ->right union(left=leg ->shape translate(x=length-2, y=0))
    ->shape rotate(x=180)
    ->shape translate(x=-length/2, y=width/2, z=height)
    ->shape fillet(radius=0.4)
    ;
