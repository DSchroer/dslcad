// Fillets and chamfers edges, optionally filtered by direction.
#include <cmath>

#include <BRepAdaptor_Curve.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <Standard_Failure.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Vec.hxx>

namespace {

// axis_mask bits: 1 = x, 2 = y, 4 = z. A mask of 0 matches every edge.
bool matches_axis(const TopoDS_Edge& edge, int axis_mask) {
    if (axis_mask == 0) {
        return true;
    }

    try {
        BRepAdaptor_Curve curve(edge);
        const double parameter = (curve.FirstParameter() + curve.LastParameter()) / 2.0;

        gp_Pnt point;
        gp_Vec direction;
        curve.D1(parameter, point, direction);

        const double length = direction.Magnitude();
        if (length < 1e-12) {
            return false;
        }
        direction /= length;

        if ((axis_mask & 1) != 0 && std::fabs(direction.X()) > 0.999) {
            return true;
        }
        if ((axis_mask & 2) != 0 && std::fabs(direction.Y()) > 0.999) {
            return true;
        }
        if ((axis_mask & 4) != 0 && std::fabs(direction.Z()) > 0.999) {
            return true;
        }
        return false;
    } catch (const Standard_Failure&) {
        return false;
    }
}

}  // namespace

// Fillets the selected edges.
extern "C" void* dslcad_fillet(const void* shape, double radius, int axis_mask) {
    if (shape == nullptr || radius <= 0.0) {
        return nullptr;
    }

    try {
        const TopoDS_Shape& input = *static_cast<const TopoDS_Shape*>(shape);
        BRepFilletAPI_MakeFillet fillet(input);

        int added = 0;
        for (TopExp_Explorer explorer(input, TopAbs_EDGE); explorer.More(); explorer.Next()) {
            const TopoDS_Edge& edge = TopoDS::Edge(explorer.Current());
            if (!matches_axis(edge, axis_mask)) {
                continue;
            }

            fillet.Add(radius, edge);
            ++added;
        }

        if (added == 0) {
            return nullptr;
        }

        fillet.Build();
        if (!fillet.IsDone()) {
            return nullptr;
        }
        return new TopoDS_Shape(fillet.Shape());
    } catch (const Standard_Failure&) {
        return nullptr;
    }
}

// Chamfers the selected edges.
extern "C" void* dslcad_chamfer(const void* shape, double distance, int axis_mask) {
    if (shape == nullptr || distance <= 0.0) {
        return nullptr;
    }

    try {
        const TopoDS_Shape& input = *static_cast<const TopoDS_Shape*>(shape);
        BRepFilletAPI_MakeChamfer chamfer(input);

        int added = 0;
        for (TopExp_Explorer explorer(input, TopAbs_EDGE); explorer.More(); explorer.Next()) {
            const TopoDS_Edge& edge = TopoDS::Edge(explorer.Current());
            if (!matches_axis(edge, axis_mask)) {
                continue;
            }

            chamfer.Add(distance, edge);
            ++added;
        }

        if (added == 0) {
            return nullptr;
        }

        chamfer.Build();
        if (!chamfer.IsDone()) {
            return nullptr;
        }
        return new TopoDS_Shape(chamfer.Shape());
    } catch (const Standard_Failure&) {
        return nullptr;
    }
}
