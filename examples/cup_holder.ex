
var baseHeight = 8;
var baseRadius = 5;
var topHeight = 10;
var topRadius = 8;

var thickness = 1;

var base = cylinder(radius=baseRadius, height=baseHeight);
var top = cylinder(radius=topRadius, height=topHeight)
            ->shape translate(z=baseHeight);

var cutout = cube(width=baseRadius, height=topHeight, length=baseRadius*2)
            ->shape translate(y=-baseRadius/2)
            ->left union(right=cylinder(radius=baseRadius/2, height=baseRadius*2)
                ->shape rotate(y=90))
            ->shape translate(z=baseHeight + 6);

base
    ->left union(right=top)
    ->left difference(right=cylinder(radius=baseRadius - thickness, height=topHeight + baseHeight))
    ->left difference(right=cylinder(radius=topRadius - thickness, height=topHeight - thickness)
        ->shape translate(z=baseHeight + thickness))
    ->left difference(right=cutout)
    ->left difference(right=cutout ->shape rotate(z=90))
    ->left difference(right=cutout ->shape rotate(z=180))
    ->left difference(right=cutout ->shape rotate(z=270))
;
