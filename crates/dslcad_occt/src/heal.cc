// Repairs a shape, for example after reading an IGES file where faces can be
// disconnected or oriented inconsistently.
#include <Standard_Failure.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepLib.hxx>
#include <ShapeFix_Shape.hxx>
#include <ShapeFix_Shell.hxx>
#include <ShapeFix_Solid.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>

extern "C" void* dslcad_heal_shape(const void* shape) {
    if (shape == nullptr) {
        return nullptr;
    }

    try {
        const TopoDS_Shape& input = *static_cast<const TopoDS_Shape*>(shape);

        // Stitch the faces back into shells, then fix whatever is still wrong
        // (orientation, missing p-curves, small gaps).
        BRepBuilderAPI_Sewing sewing(1e-6);
        sewing.Add(input);
        sewing.Perform();

        ShapeFix_Shape fixer(sewing.SewedShape());
        fixer.SetPrecision(1e-6);
        fixer.Perform();

        const TopoDS_Shape& fixed = fixer.Shape();

        // Rebuild a single-shell shape as a solid: orient the faces of the
        // shell first, then make a solid with a finite volume. This gives
        // imported shells (IGES in particular) the same orientation a native
        // solid has.
        int shells = 0;
        TopoDS_Shell shell;
        for (TopExp_Explorer explorer(fixed, TopAbs_SHELL); explorer.More(); explorer.Next()) {
            if (shells == 0) {
                shell = TopoDS::Shell(explorer.Current());
            }
            ++shells;
        }

        if (shells == 1) {
            ShapeFix_Shell shell_fixer;
            shell_fixer.FixFaceOrientation(shell, Standard_True);

            ShapeFix_Solid solid_fixer;
            TopoDS_Solid solid = solid_fixer.SolidFromShell(shell);
            if (!solid.IsNull()) {
                return new TopoDS_Shape(solid);
            }
        }

        return new TopoDS_Shape(fixed);
    } catch (const Standard_Failure&) {
        return nullptr;
    }
}
