cube(width=2, length=2, height=5)
    ->shape translate(x=10)
    //->shape rotate(x=90)
    //->shape scale(size=2)
    ->right union(left=cube(width=5, length=5, height=2))
    ->shape fillet(radius=0.4)
    ;
