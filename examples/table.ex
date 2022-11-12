var width = 10;
var length = 10;
var height = 5;

var leg = cube(x=2, y=2, z=height);
var base = cube(x=width, y=length, z=1);

base
    ->right union(left=leg)
    ->right union(left=leg ->shape translate(x=0, y=width-2))
    ->right union(left=leg ->shape translate(x=length-2, y=width-2))
    ->right union(left=leg ->shape translate(x=length-2, y=0))
    ->shape rotate(x=180)
    ->shape translate(x=-length/2, y=width/2, z=height)
    ->shape fillet(radius=0.4)
    ;
