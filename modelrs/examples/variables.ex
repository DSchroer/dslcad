var width = 10;

var leg = cube(width=2, length=2, height=5);
var base = cube(width=width, length=10, height=1);

base
    ->right union(left=leg)
    ->right union(left=leg ->shape translate(y=width-2))
    ->right union(left=leg ->shape translate(x=8, y=width-2))
    ->right union(left=leg ->shape translate(x=8))
    ->shape rotate(x=180)
    ->shape translate(x=-5, y=width/2, z=5)
    ->shape fillet(radius=0.4)
    ;
