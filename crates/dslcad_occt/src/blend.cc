// Fillets and chamfers edges, optionally filtered by direction and with a
// variable radius along each edge.
#include <cmath>

#include <BRepAdaptor_Curve.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <Standard_Failure.hxx>
#include <TColgp_Array1OfPnt2d.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Pnt2d.hxx>
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

// Fillets the selected edges. count == 0 uses a uniform radius, otherwise the
// arrays hold `count` (position along the edge, radius) pairs for a variable
// radius.
extern "C" void* dslcad_fillet(const void* shape, double radius, const double* positions,
                               const double* radii, int count, int axis_mask) {
    if (shape == nullptr || radius < 0.0 || (count > 0 && (positions == nullptr || radii == nullptr))) {
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

            if (count == 0) {
                fillet.Add(radius, edge);
            } else {
                // Map the positions from the [0, 1] range to the edge's own
                // parameter range.
                BRepAdaptor_Curve curve(edge);
                const double first = curve.FirstParameter();
                const double last = curve.LastParameter();

                TColgp_Array1OfPnt2d values(1, count);
                for (int i = 0; i < count; ++i) {
                    values.SetValue(i + 1, gp_Pnt2d(first + positions[i] * (last - first), radii[i]));
                }
                fillet.Add(values, edge);
            }
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
