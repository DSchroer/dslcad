var thickness = 5;

var baseHeight = 55;
var baseRadius = 72/2;
var topHeight = 50;
var topRadius = (89/2) + thickness;

var base = cylinder(radius=baseRadius, height=baseHeight);
var top = cylinder(radius=topRadius, height=topHeight)
            ->shape translate(z=baseHeight);

var cutout = cube(y=baseRadius, z=topHeight, x=topRadius)
            ->shape translate(y=-baseRadius/2)
            ->left union(right=cylinder(radius=baseRadius/2, height=topRadius)
                ->shape rotate(y=90))
            ->shape translate(z=baseHeight + (topHeight * 0.7));

var a = point(x=baseRadius, y=baseRadius);
var b = point(x=baseRadius, y=topRadius);
var c = point(x=topRadius, y=baseRadius);
var face = line(start=a, end=b)
    ->left union(right=line(start=b, end=c))
    ->left union(right=line(start=a, end=c))
    ->shape revolve(y=360)
    ->shape rotate(x=270)
    ->shape translate(z=(baseHeight*2)-(topRadius-baseRadius-thickness));

var core = base
    ->left union(right=top)
    ->left difference(right=cylinder(radius=baseRadius - thickness, height=topHeight + baseHeight))
    ->left difference(right=cylinder(radius=topRadius - thickness, height=topHeight - thickness)
        ->shape translate(z=baseHeight + thickness))
    ->left difference(right=cutout)
    ->left difference(right=cutout ->shape rotate(z=90))
    ->left difference(right=cutout ->shape rotate(z=180))
    ->left difference(right=cutout ->shape rotate(z=270))
;

core ->left union(right=face);
