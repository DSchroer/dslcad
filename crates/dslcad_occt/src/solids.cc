// Sweeps a profile along a path. The profile may be a wire (a shell sweep) or
// a face (a solid sweep).
#include <Standard_Failure.hxx>
#include <BRepOffsetAPI_MakePipe.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Wire.hxx>

extern "C" void* dslcad_sweep_shape(const void* profile, const void* path) {
    if (profile == nullptr || path == nullptr) {
        return nullptr;
    }

    try {
        const TopoDS_Shape& profile_shape = *static_cast<const TopoDS_Shape*>(profile);
        const TopoDS_Wire& path_wire = TopoDS::Wire(*static_cast<const TopoDS_Shape*>(path));

        BRepOffsetAPI_MakePipe sweep(path_wire, profile_shape);
        if (!sweep.IsDone()) {
            return nullptr;
        }
        return new TopoDS_Shape(sweep.Shape());
    } catch (const Standard_Failure&) {
        return nullptr;
    }
}
