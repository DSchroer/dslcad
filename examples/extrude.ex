var start = point(x=0, y=0);
var arc = point(x=5, y=5);
var end = point(x=0, y=10);

var face = line(start=start, end=end)
    ->left union(right=line(start=start, end=arc))
    ->left union(right=line(start=arc, end=end));

face ->shape revolve(y=360) ->shape fillet(radius=2);
