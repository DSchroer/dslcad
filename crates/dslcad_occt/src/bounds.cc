// Computes the axis aligned bounding box of a shape. Writes the minimum and
// maximum corner into the provided arrays and returns true on success.
#include <Bnd_Box.hxx>
#include <BRepBndLib.hxx>
#include <TopoDS_Shape.hxx>

extern "C" bool dslcad_shape_bounds(const void* shape, double* minimum, double* maximum) {
    if (shape == nullptr || minimum == nullptr || maximum == nullptr) {
        return false;
    }

    const TopoDS_Shape& input = *static_cast<const TopoDS_Shape*>(shape);

    Bnd_Box box;
    BRepBndLib::Add(input, box, Standard_False);
    if (box.IsVoid()) {
        return false;
    }

    // Bnd_Box adds a small tolerance to every corner, drop it so the reported
    // bounds reflect the actual geometry.
    box.SetGap(0.0);
    box.Get(minimum[0], minimum[1], minimum[2], maximum[0], maximum[1], maximum[2]);
    return true;
}
