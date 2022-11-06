var leg = cube(width=2, length=2, height=5);

var t = 5 + 5;

var table =
    cube(width=10, length=10, height=1)
        //->shape translate(x=10)
        //->shape rotate(x=90)
        //->shape scale(value=10)
        ->left difference(right=leg)
        ->shape chamfer(radius=0.1
    ;

table;
