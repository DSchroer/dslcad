var barn = ./barn(angle=25);
var ground = ./ground();

ground
->left union(right=barn ->shape translate(x=ground.x - 50, y=20))
->left union(right=barn ->shape translate(x=ground.x - 50, y=130))
->left union(right=barn ->shape translate(x=ground.x - 150, y=20, z=5))
->left union(right=barn ->shape translate(x=50, y=130, z=17))
;
