var table = cube(width=10, length=10, height=1)
    ->left union(right=cube(width=2, length=2, height=5))
    ->shape fillet(radius=0.4);

table;
