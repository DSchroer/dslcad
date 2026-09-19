// Bends a shape around an axis by wrapping one of its extents into a circular
// arc. The deformation is applied to the B-Rep geometry with BRepTools_Modifier,
// so curves and surfaces are rebuilt as B-splines (no triangulation involved).
#include <cmath>
#include <memory>
#include <utility>

#include <Bnd_Box.hxx>
#include <BRepBndLib.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepLib.hxx>
#include <BRepTools.hxx>
#include <BRepTools_Modification.hxx>
#include <BRepTools_Modifier.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom2dAPI_Interpolate.hxx>
#include <Geom2dAPI_PointsToBSpline.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_Curve.hxx>
#include <GeomAPI_PointsToBSpline.hxx>
#include <GeomAPI_PointsToBSplineSurface.hxx>
#include <GeomAPI_ProjectPointOnCurve.hxx>
#include <GeomAbs_Shape.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_Curve.hxx>
#include <Geom_Surface.hxx>
#include <NCollection_DataMap.hxx>
#include <ShapeAnalysis_Surface.hxx>
#include <ShapeFix_Shape.hxx>
#include <Standard_Failure.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TColgp_Array1OfPnt2d.hxx>
#include <TColgp_HArray1OfPnt2d.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TopAbs_Orientation.hxx>
#include <TopExp.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Vec2d.hxx>

namespace {

const int SURFACE_SAMPLES = 15;
const int CURVE_SAMPLES = 33;

struct Bend {
    int axis;
    int wrap;
    int height;
    double center[3];
    double radius;
    double sign;
    double tolerance;
    // Size of the shape, used to convert the 3D tolerance into the normalized
    // parameter space of the fitted surfaces.
    double scale;
};

gp_Pnt bend_point(const Bend& bend, const gp_Pnt& point) {
    const double coordinates[3] = {point.X(), point.Y(), point.Z()};
    const double along = coordinates[bend.wrap] - bend.center[bend.wrap];
    const double offset = bend.sign * (coordinates[bend.height] - bend.center[bend.height]);

    const double angle = along / bend.radius;
    const double radius = bend.radius - offset;

    double result[3];
    result[bend.axis] = coordinates[bend.axis];
    result[bend.wrap] = bend.center[bend.wrap] + radius * std::sin(angle);
    result[bend.height] =
        bend.center[bend.height] + bend.sign * (bend.radius - radius * std::cos(angle));
    return gp_Pnt(result[0], result[1], result[2]);
}

// Rebuilds every curve and surface of a shape from points mapped through the
// bend. The new geometry is approximated with B-splines within a tolerance
// relative to the shape's size.
class BendModification : public BRepTools_Modification {
public:
    explicit BendModification(const Bend& bend) : myBend(bend), myFailed(false) {}

    Standard_Boolean NewSurface(const TopoDS_Face& face, Handle(Geom_Surface)& surface,
                                TopLoc_Location& location, Standard_Real& tolerance,
                                Standard_Boolean& reverse_wires,
                                Standard_Boolean& reverse_face) override {
        Standard_Real u1, u2, v1, v2;
        BRepTools::UVBounds(face, u1, u2, v1, v2);

        TopLoc_Location face_location;
        Handle(Geom_Surface) original = BRep_Tool::Surface(face, face_location);
        if (original.IsNull()) {
            return Standard_False;
        }

        // Closed surfaces are sampled without their duplicated end point and
        // fitted as a periodic surface, otherwise the seam of the rebuilt
        // surface overshoots and its projections become ambiguous.
        const bool u_closed = original->IsUClosed();
        const bool v_closed = original->IsVClosed();
        const int nu = SURFACE_SAMPLES;
        const int nv = SURFACE_SAMPLES;
        const double du = u_closed ? (u2 - u1) / nu : (u2 - u1) / (nu - 1);
        const double dv = v_closed ? (v2 - v1) / nv : (v2 - v1) / (nv - 1);

        TColgp_Array2OfPnt points(1, nu, 1, nv);
        for (int i = 0; i < nu; ++i) {
            for (int j = 0; j < nv; ++j) {
                const gp_Pnt point = original->Value(u1 + du * i, v1 + dv * j)
                                         .Transformed(face_location.Transformation());
                const gp_Pnt bent = bend_point(myBend, point);

                // When only V is closed, transpose the grid so that the periodic
                // fit can close it, and swap the fitted surface back below.
                if (v_closed && !u_closed) {
                    points.SetValue(j + 1, i + 1, bent);
                } else {
                    points.SetValue(i + 1, j + 1, bent);
                }
            }
        }

        GeomAPI_PointsToBSplineSurface fit;
        if (u_closed || (v_closed && !u_closed)) {
            fit.Interpolate(points, Approx_ChordLength, Standard_True);
        } else {
            fit.Init(points, 3, 8, GeomAbs_C2, myBend.tolerance);
            if (!fit.IsDone()) {
                fit.Interpolate(points);
            }
        }
        if (!fit.IsDone()) {
            myFailed = true;
            return Standard_False;
        }

        Handle(Geom_BSplineSurface) fitted = fit.Surface();
        if (v_closed && !u_closed) {
            fitted->ExchangeUV();
        }

        surface = fitted;
        location = TopLoc_Location();
        tolerance = myBend.tolerance;
        reverse_wires = Standard_False;
        reverse_face = Standard_False;
        return Standard_True;
    }

    Standard_Boolean NewCurve(const TopoDS_Edge& edge, Handle(Geom_Curve)& curve,
                              TopLoc_Location& location, Standard_Real& tolerance) override {
        Standard_Real first, last;
        TopLoc_Location edge_location;
        Handle(Geom_Curve) original = BRep_Tool::Curve(edge, edge_location, first, last);
        if (original.IsNull()) {
            return Standard_False;
        }

        TColgp_Array1OfPnt points(1, CURVE_SAMPLES);
        for (int i = 0; i < CURVE_SAMPLES; ++i) {
            const Standard_Real t = first + (last - first) * i / (CURVE_SAMPLES - 1);
            const gp_Pnt point = original->Value(t).Transformed(edge_location.Transformation());
            points.SetValue(i + 1, bend_point(myBend, point));
        }

        GeomAPI_PointsToBSpline fit;
        fit.Init(points, 3, 8, GeomAbs_C2, myBend.tolerance);
        if (!fit.IsDone()) {
            myFailed = true;
            return Standard_False;
        }

        curve = fit.Curve();
        location = TopLoc_Location();
        tolerance = myBend.tolerance;
        myCurves.Bind(forward(edge), curve);
        return Standard_True;
    }

    Standard_Boolean NewPoint(const TopoDS_Vertex& vertex, gp_Pnt& point,
                              Standard_Real& tolerance) override {
        point = bend_point(myBend, BRep_Tool::Pnt(vertex));
        tolerance = myBend.tolerance;
        return Standard_True;
    }

    Standard_Boolean NewCurve2d(const TopoDS_Edge& edge, const TopoDS_Face& face,
                                const TopoDS_Edge& new_edge, const TopoDS_Face& new_face,
                                Handle(Geom2d_Curve)& curve, Standard_Real& tolerance) override {
        if (new_edge.IsNull() || new_face.IsNull()) {
            return Standard_False;
        }

        Standard_Real first, last;
        TopLoc_Location edge_location;
        Handle(Geom_Curve) curve3d = BRep_Tool::Curve(new_edge, edge_location, first, last);
        Handle(Geom_Surface) surface = BRep_Tool::Surface(new_face);
        if (surface.IsNull()) {
            return Standard_False;
        }

        // Degenerate edges (for example the poles of a sphere) have no 3D
        // curve, their pcurve is the projection of their single vertex.
        if (curve3d.IsNull()) {
            if (!BRep_Tool::Degenerated(new_edge)) {
                return Standard_False;
            }

            const TopoDS_Vertex vertex = TopExp::FirstVertex(edge);
            const gp_Pnt point = bend_point(myBend, BRep_Tool::Pnt(vertex));
            ShapeAnalysis_Surface analysis(surface);
            const gp_Pnt2d uv = analysis.ValueOfUV(point, myBend.tolerance);

            TColgp_Array1OfPnt2d poles(1, 2);
            poles.SetValue(1, uv);
            poles.SetValue(2, uv);
            curve = new Geom2d_BezierCurve(poles);
            tolerance = myBend.tolerance;
            return Standard_True;
        }

        // The modifier does not always set the range of the new edges, fall
        // back to the range of the curve itself.
        if (first >= last) {
            first = curve3d->FirstParameter();
            last = curve3d->LastParameter();
        }

        ShapeAnalysis_Surface analysis(surface);
        const bool u_periodic = surface->IsUPeriodic();
        const bool v_periodic = surface->IsVPeriodic();
        const Standard_Real u_period = u_periodic ? surface->UPeriod() : 0.0;
        const Standard_Real v_period = v_periodic ? surface->VPeriod() : 0.0;

        TColgp_Array1OfPnt2d points(1, CURVE_SAMPLES);
        gp_Pnt2d previous;
        for (int i = 0; i < CURVE_SAMPLES; ++i) {
            const Standard_Real t = first + (last - first) * i / (CURVE_SAMPLES - 1);
            const gp_Pnt point = curve3d->Value(t).Transformed(edge_location.Transformation());

            // Consecutive points are close together, so the previous parameter
            // is a good starting point for a local projection. This is much
            // faster than a full projection on the fitted surface, but fall
            // back to one when the local solution drifts too far from the point.
            gp_Pnt2d uv =
                i == 0
                    ? analysis.ValueOfUV(point, myBend.tolerance)
                    : analysis.NextValueOfUV(previous, point, myBend.tolerance,
                                             myBend.tolerance * 10.0);

            // Unwrap periodic parameters so the fitted pcurve does not jump
            // across the seam of the surface.
            if (i > 0) {
                if (u_periodic) {
                    while (uv.X() - previous.X() > u_period / 2.0) {
                        uv.SetX(uv.X() - u_period);
                    }
                    while (previous.X() - uv.X() > u_period / 2.0) {
                        uv.SetX(uv.X() + u_period);
                    }
                }
                if (v_periodic) {
                    while (uv.Y() - previous.Y() > v_period / 2.0) {
                        uv.SetY(uv.Y() - v_period);
                    }
                    while (previous.Y() - uv.Y() > v_period / 2.0) {
                        uv.SetY(uv.Y() + v_period);
                    }
                }
            }

            previous = uv;
            points.SetValue(i + 1, uv);
        }

        // The fitted surface is normalized to [0, 1] in both directions, so a
        // 3D tolerance is divided by the size of the shape to get the
        // equivalent tolerance in parameter space.
        Handle(Geom2d_Curve) result;
        Handle(Geom_Surface) original_surface = BRep_Tool::Surface(face);
        const bool closed =
            (!original_surface.IsNull() &&
             (original_surface->IsUClosed() || original_surface->IsVClosed())) ||
            BRep_Tool::IsClosed(edge);

        if (closed) {
            // Closed surfaces and closed edges contain seams, so keep using an
            // approximation that stays within a single period.
            Geom2dAPI_PointsToBSpline fit;
            fit.Init(points, 3, 8, GeomAbs_C2, myBend.tolerance / myBend.scale);
            if (!fit.IsDone()) {
                myFailed = true;
                return Standard_False;
            }
            result = fit.Curve();
        } else {
            // Interpolating the projected points keeps short pcurves simple.
            // Approximating them with the tight tolerance above can instead
            // oscillate between samples with an overshoot far larger than the
            // pcurve itself, which leaves the shape invalid.
            Handle(TColgp_HArray1OfPnt2d) interpolated =
                new TColgp_HArray1OfPnt2d(1, CURVE_SAMPLES);
            for (int i = 1; i <= CURVE_SAMPLES; ++i) {
                interpolated->SetValue(i, points.Value(i));
            }

            Geom2dAPI_Interpolate interpolate(interpolated, Standard_False,
                                              myBend.tolerance / myBend.scale);
            interpolate.Perform();
            if (interpolate.IsDone()) {
                result = interpolate.Curve();
            } else {
                // Interpolation fails when the samples collapse, so fall back
                // to an approximation that may deviate by the 3D tolerance.
                Geom2dAPI_PointsToBSpline fit;
                fit.Init(points, 3, 8, GeomAbs_C2, myBend.tolerance);
                if (!fit.IsDone()) {
                    myFailed = true;
                    return Standard_False;
                }
                result = fit.Curve();
            }
        }

        // Both occurrences of a seam edge need a pcurve on opposite sides of
        // the seam, otherwise the face collapses to a line in parameter space.
        if (BRep_Tool::IsClosed(edge, face) && BRepTools::IsReallyClosed(edge, face) &&
            edge.Orientation() == TopAbs_REVERSED) {
            if (u_periodic) {
                result = Handle(Geom2d_Curve)::DownCast(
                    result->Translated(gp_Vec2d(u_period, 0.0)));
            } else if (v_periodic) {
                result = Handle(Geom2d_Curve)::DownCast(
                    result->Translated(gp_Vec2d(0.0, v_period)));
            }
        }

        curve = result;
        tolerance = myBend.tolerance;

        // The modifier leaves the range of the edges it rebuilds empty, restore
        // it so the edges can be evaluated and meshed.
        BRep_Builder builder;
        builder.Range(new_edge, first, last);

        return Standard_True;
    }

    Standard_Boolean NewParameter(const TopoDS_Vertex& vertex, const TopoDS_Edge& edge,
                                  Standard_Real& parameter, Standard_Real& tolerance) override {
        const TopoDS_Shape key = forward(edge);
        if (!myCurves.IsBound(key)) {
            return Standard_False;
        }

        const gp_Pnt point = bend_point(myBend, BRep_Tool::Pnt(vertex));
        GeomAPI_ProjectPointOnCurve projection(point, myCurves.Find(key));
        if (projection.NbPoints() < 1) {
            return Standard_False;
        }

        parameter = projection.LowerDistanceParameter();
        tolerance = myBend.tolerance;
        return Standard_True;
    }

    GeomAbs_Shape Continuity(const TopoDS_Edge&, const TopoDS_Face&, const TopoDS_Face&,
                             const TopoDS_Edge&, const TopoDS_Face&, const TopoDS_Face&) override {
        return GeomAbs_C0;
    }

    bool Failed() const { return myFailed; }

private:
    static TopoDS_Shape forward(const TopoDS_Edge& edge) {
        return TopoDS::Edge(edge).Oriented(TopAbs_FORWARD);
    }

    Bend myBend;
    bool myFailed;
    NCollection_DataMap<TopoDS_Shape, Handle(Geom_Curve)> myCurves;
};

} // namespace

extern "C" void* dslcad_bend_shape(const void* shape, int axis, double degrees) {
    if (shape == nullptr || axis < 0 || axis > 2 || degrees == 0.0) {
        return nullptr;
    }

    try {
        const TopoDS_Shape& input = *static_cast<const TopoDS_Shape*>(shape);

        Bnd_Box box;
        BRepBndLib::Add(input, box, Standard_False);
        if (box.IsVoid()) {
            return nullptr;
        }

        Standard_Real xmin, ymin, zmin, xmax, ymax, zmax;
        box.Get(xmin, ymin, zmin, xmax, ymax, zmax);
        const double minimum[3] = {xmin, ymin, zmin};
        const double maximum[3] = {xmax, ymax, zmax};

        // Wrap the larger of the two extents perpendicular to the bend axis, the
        // other one becomes the direction the shape bends towards.
        int wrap = (axis + 1) % 3;
        int height = (axis + 2) % 3;
        if ((maximum[wrap] - minimum[wrap]) < (maximum[height] - minimum[height])) {
            std::swap(wrap, height);
        }

        const double length = maximum[wrap] - minimum[wrap];
        if (length < 1e-9) {
            return nullptr;
        }

        const double angle = degrees * M_PI / 180.0;

        Bend bend;
        bend.axis = axis;
        bend.wrap = wrap;
        bend.height = height;
        bend.center[0] = 0.0;
        bend.center[1] = 0.0;
        bend.center[2] = 0.0;
        bend.center[wrap] = (minimum[wrap] + maximum[wrap]) / 2.0;
        bend.center[height] = (minimum[height] + maximum[height]) / 2.0;
        bend.radius = length / std::fabs(angle);
        bend.sign = angle >= 0.0 ? 1.0 : -1.0;

        const double diagonal = std::sqrt(
            (xmax - xmin) * (xmax - xmin) + (ymax - ymin) * (ymax - ymin) +
            (zmax - zmin) * (zmax - zmin));
        bend.tolerance = std::max(diagonal * 1e-6, 1e-9);
        bend.scale = std::max(diagonal, 1e-9);

        Handle(BendModification) modification = new BendModification(bend);
        BRepTools_Modifier modifier(input, modification);
        if (!modifier.IsDone() || modification->Failed()) {
            return nullptr;
        }

        TopoDS_Shape result = modifier.ModifiedShape(input);
        if (result.IsNull()) {
            return nullptr;
        }

        // Fitting surfaces and curves leaves the pcurves of some edges with a
        // parameterization that no longer matches their 3D curve. Recomputing
        // those pcurves is much cheaper than a full ShapeFix pass, so only fall
        // back to ShapeFix when the shape stays invalid.
        if (!BRepCheck_Analyzer(result).IsValid()) {
            BRepLib::SameParameter(result, bend.tolerance, Standard_True);
        }

        if (!BRepCheck_Analyzer(result).IsValid()) {
            Handle(ShapeFix_Shape) fixer = new ShapeFix_Shape(result);
            fixer->Perform();
            result = fixer->Shape();
            if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid()) {
                return nullptr;
            }
        }

        return new TopoDS_Shape(result);
    } catch (const Standard_Failure&) {
        return nullptr;
    } catch (...) {
        return nullptr;
    }
}
