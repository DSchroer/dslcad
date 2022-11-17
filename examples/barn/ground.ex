// Setting x for later use
var x = 476;

// build on its side until extrude can be used on any axis
// this will be simpler in later versions
var base = line(start=point(), end=point(x=x))
->left union(right=line(start=point(), end=point(z=20)))
->left union(right=line(start=point(z=20), end=point(x=x)))
->shape extrude(y=278+195)
;

base
->left difference(right=cube(x=236, y=195, z=20) ->shape translate(x=240, y=278))
;
