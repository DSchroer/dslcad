// Angle allows main to rotate the barn
var angle = 0;

var row = ./pillar()
->left union(right=./pillar() ->shape translate(x=9))
->left union(right=./pillar() ->shape translate(x=9*2))
->left union(right=./pillar() ->shape translate(x=9*3))
;

var all = row
->left union(right=row ->shape translate(y=10))
->left union(right=row ->shape translate(y=10*2))
->left union(right=row ->shape translate(y=10*3))
->left union(right=row ->shape translate(y=10*4))
;

all ->shape rotate(z=angle);
