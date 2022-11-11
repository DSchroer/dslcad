var a = point(x=0.00, y=5.49);
var b = point(x=14.02, y=5.49);
var c = point(x=8.00, y=7.98);

line(start=a, end=b)
->left union(right=line(start=b, end=c))
->left union(right=line(start=c, end=a))
->shape extrude(height=15.84)
->shape translate(y=-5.49)
->shape rotate(x=90)
->shape translate(y=15.84);
