// Boolean operations with glue and fuzzy options.
#include <memory>

#include <BOPAlgo_GlueEnum.hxx>
#include <BRepAlgoAPI_BuilderAlgo.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <Standard_Failure.hxx>
#include <TopoDS_Shape.hxx>

// operation: 0 fuse, 1 cut, 2 common. glue: 0 off, 1 shift, 2 full.
extern "C" void* dslcad_boolean(const void* left, const void* right, int operation, int glue,
                                double fuzzy) {
    if (left == nullptr || right == nullptr) {
        return nullptr;
    }

    try {
        const TopoDS_Shape& a = *static_cast<const TopoDS_Shape*>(left);
        const TopoDS_Shape& b = *static_cast<const TopoDS_Shape*>(right);

        std::unique_ptr<BRepAlgoAPI_BuilderAlgo> algorithm;
        switch (operation) {
            case 0:
                algorithm = std::make_unique<BRepAlgoAPI_Fuse>(a, b);
                break;
            case 1:
                algorithm = std::make_unique<BRepAlgoAPI_Cut>(a, b);
                break;
            default:
                algorithm = std::make_unique<BRepAlgoAPI_Common>(a, b);
                break;
        }

        switch (glue) {
            case 1:
                algorithm->SetGlue(BOPAlgo_GlueShift);
                break;
            case 2:
                algorithm->SetGlue(BOPAlgo_GlueFull);
                break;
            default:
                algorithm->SetGlue(BOPAlgo_GlueOff);
                break;
        }

        if (fuzzy > 0.0) {
            algorithm->SetFuzzyValue(fuzzy);
        }

        algorithm->Build();
        if (algorithm->HasErrors()) {
            return nullptr;
        }

        return new TopoDS_Shape(algorithm->Shape());
    } catch (const Standard_Failure&) {
        return nullptr;
    }
}
