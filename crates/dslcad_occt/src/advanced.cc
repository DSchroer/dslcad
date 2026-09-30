// Advanced modelling queries and operations: distance and containment,
// splitting solids, removing drilled holes and drilling holes.
#include <cmath>

#include <BRepAdaptor_Surface.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Defeaturing.hxx>
#include <BRepAlgoAPI_Splitter.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepFeat_MakeCylindricalHole.hxx>
#include <BRepFeat_Status.hxx>
#include <BRepGProp.hxx>
#include <GeomAbs_SurfaceType.hxx>
#include <GProp_GProps.hxx>
#include <Precision.hxx>
#include <Standard_Failure.hxx>
#include <TopAbs_State.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Ax1.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>

// Minimum distance between two shapes.
extern "C" bool dslcad_distance(const void* left, const void* right, double* distance) {
    if (left == nullptr || right == nullptr || distance == nullptr) {
        return false;
    }

    try {
        const TopoDS_Shape& a = *static_cast<const TopoDS_Shape*>(left);
        const TopoDS_Shape& b = *static_cast<const TopoDS_Shape*>(right);

        // Detect touching or overlapping shapes first. Asking BRepExtrema for
        // a zero distance crashes, so the second shape is scaled up minutely
        // and the two are intersected instead.
        GProp_GProps props;
        BRepGProp::VolumeProperties(b, props);

        gp_Trsf scale;
        scale.SetScale(props.CentreOfMass(), 1.0 + 1e-6);
        BRepBuilderAPI_Transform scaling(b, scale, true);

        BRepAlgoAPI_Common common(a, scaling.Shape());
        common.SetFuzzyValue(Precision::Confusion());
        common.Build();
        if (!common.HasErrors() && !common.Shape().IsNull()) {
            TopExp_Explorer shared(common.Shape(), TopAbs_VERTEX);
            if (shared.More()) {
                *distance = 0.0;
                return true;
            }
        }

        BRepExtrema_DistShapeShape extrema(a, b);
        extrema.Perform();
        if (!extrema.IsDone() || extrema.NbSolution() < 1) {
            return false;
        }

        *distance = extrema.Value();
        return true;
    } catch (const Standard_Failure&) {
        return false;
    }
}

// Whether a point is inside (1), on (2) or outside (0) a solid.
extern "C" int dslcad_contains(const void* shape, double x, double y, double z) {
    if (shape == nullptr) {
        return 0;
    }

    try {
        const TopoDS_Shape& solid = *static_cast<const TopoDS_Shape*>(shape);

        BRepClass3d_SolidClassifier classifier(solid);
        classifier.Perform(gp_Pnt(x, y, z), Precision::Confusion());

        switch (classifier.State()) {
            case TopAbs_IN:
                return 1;
            case TopAbs_ON:
                return 2;
            default:
                return 0;
        }
    } catch (const Standard_Failure&) {
        return 0;
    }
}

// Splits a shape with a tool shape, returning a compound of the pieces.
extern "C" void* dslcad_split(const void* shape, const void* tool) {
    if (shape == nullptr || tool == nullptr) {
        return nullptr;
    }

    try {
        TopTools_ListOfShape arguments;
        arguments.Append(*static_cast<const TopoDS_Shape*>(shape));

        TopTools_ListOfShape tools;
        tools.Append(*static_cast<const TopoDS_Shape*>(tool));

        BRepAlgoAPI_Splitter splitter;
        splitter.SetArguments(arguments);
        splitter.SetTools(tools);
        splitter.Build();
        if (splitter.HasErrors()) {
            return nullptr;
        }

        return new TopoDS_Shape(splitter.Shape());
    } catch (const Standard_Failure&) {
        return nullptr;
    }
}

// Removes every cylindrical face with the given radius, for example to get rid
// of drilled holes.
extern "C" void* dslcad_defeature(const void* shape, double radius) {
    if (shape == nullptr || radius <= 0.0) {
        return nullptr;
    }

    try {
        const TopoDS_Shape& input = *static_cast<const TopoDS_Shape*>(shape);

        BRepAlgoAPI_Defeaturing defeaturing;
        defeaturing.SetShape(input);

        int count = 0;
        for (TopExp_Explorer explorer(input, TopAbs_FACE); explorer.More(); explorer.Next()) {
            const TopoDS_Face& face = TopoDS::Face(explorer.Current());

            BRepAdaptor_Surface surface(face);
            if (surface.GetType() != GeomAbs_Cylinder) {
                continue;
            }

            if (std::fabs(surface.Cylinder().Radius() - radius) > 1e-6) {
                continue;
            }

            defeaturing.AddFaceToRemove(face);
            ++count;
        }

        if (count == 0) {
            return nullptr;
        }

        defeaturing.Build();
        if (defeaturing.HasErrors()) {
            return nullptr;
        }

        return new TopoDS_Shape(defeaturing.Shape());
    } catch (const Standard_Failure&) {
        return nullptr;
    }
}

// Drills a cylindrical hole along an axis. A negative depth drills through the
// whole shape.
extern "C" void* dslcad_hole(const void* shape, double x, double y, double z, double dx, double dy,
                             double dz, double radius, double depth) {
    if (shape == nullptr || radius <= 0.0 || (dx == 0.0 && dy == 0.0 && dz == 0.0)) {
        return nullptr;
    }

    try {
        const TopoDS_Shape& input = *static_cast<const TopoDS_Shape*>(shape);

        BRepFeat_MakeCylindricalHole hole;
        hole.Init(input, gp_Ax1(gp_Pnt(x, y, z), gp_Dir(dx, dy, dz)));

        if (depth > 0.0) {
            hole.PerformBlind(radius, depth);
        } else {
            hole.Perform(radius);
        }

        if (hole.Status() != BRepFeat_NoError) {
            return nullptr;
        }

        TopoDS_Shape result = hole.Shape();
        if (result.IsNull()) {
            return nullptr;
        }

        return new TopoDS_Shape(result);
    } catch (const Standard_Failure&) {
        return nullptr;
    }
}
