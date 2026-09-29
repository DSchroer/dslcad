#include "point_map.hxx"

#include <cmath>
#include <algorithm>

#include <BRep_Builder.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <Geom2dAPI_Interpolate.hxx>
#include <Geom2dAPI_PointsToBSpline.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <GeomAPI_PointsToBSpline.hxx>
#include <GeomAPI_PointsToBSplineSurface.hxx>
#include <GeomAPI_ProjectPointOnCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <ShapeAnalysis_Surface.hxx>
#include <Standard_Failure.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TColgp_Array1OfPnt2d.hxx>
#include <TColgp_HArray1OfPnt2d.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TColStd_HArray1OfReal.hxx>
#include <TopAbs_Orientation.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Vec2d.hxx>

namespace dslcad {
namespace {

const int SURFACE_SAMPLES = 15;
const int MAX_SURFACE_SAMPLES = 256;
const int CURVE_SAMPLES = 33;

// Whether every control point of a surface stays within the bounding box of the
// points it was fitted to. A B-spline lies inside the convex hull of its poles,
// so poles far outside the samples mean the fit oscillated.
bool poles_within(const Handle(Geom_BSplineSurface)& surface, const double lower[3],
                  const double upper[3], double margin) {
    for (int i = 1; i <= surface->NbUPoles(); ++i) {
        for (int j = 1; j <= surface->NbVPoles(); ++j) {
            const gp_Pnt pole = surface->Pole(i, j);
            const double coordinates[3] = {pole.X(), pole.Y(), pole.Z()};
            for (int k = 0; k < 3; ++k) {
                if (coordinates[k] < lower[k] - margin || coordinates[k] > upper[k] + margin) {
                    return false;
                }
            }
        }
    }
    return true;
}

} // namespace

PointMapModification::PointMapModification(Map map, Standard_Real tolerance, Standard_Real scale)
    : myPoint(std::move(map)), myTolerance(tolerance), myScale(scale), myFailed(false) {
    mySurface = [this](const TopoDS_Face&, const gp_Pnt& point) { return myPoint(point); };
    myCurve = [this](const TopoDS_Edge&, const gp_Pnt& point) { return myPoint(point); };
    myVertex = [this](const TopoDS_Vertex&, const gp_Pnt& point) { return myPoint(point); };
}

PointMapModification::PointMapModification(Map map, SurfaceMap surface, CurveMap curve,
                                           VertexMap vertex, Standard_Real tolerance,
                                           Standard_Real scale)
    : myPoint(std::move(map)),
      mySurface(std::move(surface)),
      myCurve(std::move(curve)),
      myVertex(std::move(vertex)),
      myTolerance(tolerance),
      myScale(scale),
      myFailed(false) {}

Standard_Boolean PointMapModification::NewSurface(const TopoDS_Face& face,
                                                  Handle(Geom_Surface)& surface,
                                                  TopLoc_Location& location,
                                                  Standard_Real& tolerance,
                                                  Standard_Boolean& reverse_wires,
                                                  Standard_Boolean& reverse_face) {
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

    // A face can cover a large part of the shape. Fitting such a face from a
    // fixed, coarse grid makes the B-spline oscillate, which shows up as stray
    // geometry. Start coarse and refine until the fit stops overshooting.
    Handle(Geom_BSplineSurface) fitted;
    for (int samples = SURFACE_SAMPLES;; samples = std::min(samples * 2, MAX_SURFACE_SAMPLES)) {
        const int nu = samples;
        const int nv = samples;
        const double du = u_closed ? (u2 - u1) / nu : (u2 - u1) / (nu - 1);
        const double dv = v_closed ? (v2 - v1) / nv : (v2 - v1) / (nv - 1);

        TColgp_Array2OfPnt points(1, nu, 1, nv);
        double lower[3] = {1e300, 1e300, 1e300};
        double upper[3] = {-1e300, -1e300, -1e300};
        for (int i = 0; i < nu; ++i) {
            for (int j = 0; j < nv; ++j) {
                const gp_Pnt point = original->Value(u1 + du * i, v1 + dv * j)
                                         .Transformed(face_location.Transformation());
                const gp_Pnt moved = mySurface(face, point);

                const double coordinates[3] = {moved.X(), moved.Y(), moved.Z()};
                for (int k = 0; k < 3; ++k) {
                    lower[k] = std::min(lower[k], coordinates[k]);
                    upper[k] = std::max(upper[k], coordinates[k]);
                }

                // When only V is closed, transpose the grid so that the
                // periodic fit can close it, and swap the fitted surface back
                // below.
                if (v_closed && !u_closed) {
                    points.SetValue(j + 1, i + 1, moved);
                } else {
                    points.SetValue(i + 1, j + 1, moved);
                }
            }
        }

        GeomAPI_PointsToBSplineSurface fit;
        try {
            if (u_closed || (v_closed && !u_closed)) {
                fit.Interpolate(points, Approx_ChordLength, Standard_True);
            } else {
                fit.Init(points, 3, 8, GeomAbs_C2, myTolerance);
                if (!fit.IsDone()) {
                    fit.Interpolate(points);
                }
            }
        } catch (const Standard_Failure&) {
            // Handled by the retry / failure logic below.
        }

        if (!fit.IsDone()) {
            if (samples >= MAX_SURFACE_SAMPLES) {
                myFailed = true;
                return Standard_False;
            }
            continue;
        }

        fitted = fit.Surface();

        const double diagonal =
            std::sqrt((upper[0] - lower[0]) * (upper[0] - lower[0]) +
                      (upper[1] - lower[1]) * (upper[1] - lower[1]) +
                      (upper[2] - lower[2]) * (upper[2] - lower[2]));
        if (samples < MAX_SURFACE_SAMPLES &&
            !poles_within(fitted, lower, upper, diagonal * 0.1)) {
            continue;
        }
        break;
    }

    if (v_closed && !u_closed) {
        fitted->ExchangeUV();
    }

    surface = fitted;
    location = TopLoc_Location();
    tolerance = myTolerance;
    reverse_wires = Standard_False;
    reverse_face = Standard_False;
    return Standard_True;
}

Standard_Boolean PointMapModification::NewCurve(const TopoDS_Edge& edge,
                                                Handle(Geom_Curve)& curve,
                                                TopLoc_Location& location,
                                                Standard_Real& tolerance) {
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
        points.SetValue(i + 1, myCurve(edge, point));
    }

    GeomAPI_PointsToBSpline fit;
    fit.Init(points, 3, 8, GeomAbs_C2, myTolerance);
    if (!fit.IsDone()) {
        myFailed = true;
        return Standard_False;
    }

    curve = fit.Curve();
    location = TopLoc_Location();
    tolerance = myTolerance;
    myCurves.Bind(forward(edge), curve);
    return Standard_True;
}

Standard_Boolean PointMapModification::NewPoint(const TopoDS_Vertex& vertex, gp_Pnt& point,
                                                Standard_Real& tolerance) {
    point = myVertex(vertex, BRep_Tool::Pnt(vertex));
    tolerance = myTolerance;
    return Standard_True;
}

Standard_Boolean PointMapModification::NewParameter(const TopoDS_Vertex& vertex,
                                                    const TopoDS_Edge& edge,
                                                    Standard_Real& parameter,
                                                    Standard_Real& tolerance) {
    const TopoDS_Shape key = forward(edge);
    if (!myCurves.IsBound(key)) {
        return Standard_False;
    }

    const gp_Pnt point = myVertex(vertex, BRep_Tool::Pnt(vertex));
    GeomAPI_ProjectPointOnCurve projection(point, myCurves.Find(key));
    if (projection.NbPoints() < 1) {
        return Standard_False;
    }

    parameter = projection.LowerDistanceParameter();
    tolerance = myTolerance;
    return Standard_True;
}

Standard_Boolean PointMapModification::NewCurve2d(const TopoDS_Edge& edge,
                                                  const TopoDS_Face& face,
                                                  const TopoDS_Edge& new_edge,
                                                  const TopoDS_Face& new_face,
                                                  Handle(Geom2d_Curve)& curve,
                                                  Standard_Real& tolerance) {
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

    // Degenerate edges (for example the poles of a sphere) have no 3D curve,
    // their pcurve is the projection of their single vertex.
    if (curve3d.IsNull()) {
        if (!BRep_Tool::Degenerated(new_edge)) {
            return Standard_False;
        }

        const TopoDS_Vertex vertex = TopExp::FirstVertex(edge);
        const gp_Pnt point = myVertex(vertex, BRep_Tool::Pnt(vertex));
        ShapeAnalysis_Surface analysis(surface);
        const gp_Pnt2d uv = analysis.ValueOfUV(point, myTolerance);

        TColgp_Array1OfPnt2d poles(1, 2);
        poles.SetValue(1, uv);
        poles.SetValue(2, uv);
        curve = new Geom2d_BezierCurve(poles);
        tolerance = myTolerance;
        return Standard_True;
    }

    // The modifier does not always set the range of the new edges, fall back
    // to the range of the curve itself.
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
    Handle(TColStd_HArray1OfReal) parameters = new TColStd_HArray1OfReal(1, CURVE_SAMPLES);
    gp_Pnt2d previous;
    for (int i = 0; i < CURVE_SAMPLES; ++i) {
        const Standard_Real t = first + (last - first) * i / (CURVE_SAMPLES - 1);
        parameters->SetValue(i + 1, t);
        const gp_Pnt point = curve3d->Value(t).Transformed(edge_location.Transformation());

        // Consecutive points are close together, so the previous parameter is
        // a good starting point for a local projection. This is much faster
        // than a full projection on the fitted surface, but fall back to one
        // when the local solution drifts too far from the point.
        gp_Pnt2d uv =
            i == 0 ? analysis.ValueOfUV(point, myTolerance)
                   : analysis.NextValueOfUV(previous, point, myTolerance, myTolerance * 10.0);

        // Unwrap periodic parameters so the fitted pcurve does not jump across
        // the seam of the surface.
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

    // The fitted surface is normalized to [0, 1] in both directions, so a 3D
    // tolerance is divided by the size of the shape to get the equivalent
    // tolerance in parameter space.
    Handle(Geom2d_Curve) result;
    Handle(Geom_Surface) original_surface = BRep_Tool::Surface(face);
    const bool closed =
        (!original_surface.IsNull() &&
         (original_surface->IsUClosed() || original_surface->IsVClosed())) ||
        BRep_Tool::IsClosed(edge);

    if (closed) {
        // Closed surfaces and closed edges contain seams, so keep using an
        // approximation that stays within a single period. The pcurve is
        // parameterized with the edge parameters so it stays in sync with the
        // 3D curve.
        Geom2dAPI_PointsToBSpline fit;
        fit.Init(points, parameters->Array1(), 3, 8, GeomAbs_C2, myTolerance / myScale);
        if (!fit.IsDone()) {
            myFailed = true;
            return Standard_False;
        }
        result = fit.Curve();
    } else {
        // Interpolating the projected points keeps short pcurves simple.
        // Approximating them with the tight tolerance above can instead
        // oscillate between samples with an overshoot far larger than the
        // pcurve itself, which leaves the shape invalid. Interpolating with
        // the edge parameters keeps the pcurve in sync with the 3D curve.
        bool interpolated = false;
        try {
            Handle(TColgp_HArray1OfPnt2d) interpolated_points =
                new TColgp_HArray1OfPnt2d(1, CURVE_SAMPLES);
            for (int i = 1; i <= CURVE_SAMPLES; ++i) {
                interpolated_points->SetValue(i, points.Value(i));
            }

            Geom2dAPI_Interpolate interpolate(interpolated_points, parameters, Standard_False,
                                              myTolerance / myScale);
            interpolate.Perform();
            if (interpolate.IsDone()) {
                result = interpolate.Curve();
                interpolated = true;
            }
        } catch (const Standard_Failure&) {
            // The interpolator rejects coincident samples.
            interpolated = false;
        }

        if (!interpolated) {
            // Interpolation fails when the samples collapse, so fall back to
            // an approximation that may deviate by the 3D tolerance.
            Geom2dAPI_PointsToBSpline fit;
            fit.Init(points, parameters->Array1(), 3, 8, GeomAbs_C2, myTolerance);
            if (!fit.IsDone()) {
                myFailed = true;
                return Standard_False;
            }
            result = fit.Curve();
        }
    }

    // Both occurrences of a seam edge need a pcurve on opposite sides of the
    // seam, otherwise the face collapses to a line in parameter space.
    if (BRep_Tool::IsClosed(edge, face) && BRepTools::IsReallyClosed(edge, face) &&
        edge.Orientation() == TopAbs_REVERSED) {
        if (u_periodic) {
            result = Handle(Geom2d_Curve)::DownCast(result->Translated(gp_Vec2d(u_period, 0.0)));
        } else if (v_periodic) {
            result = Handle(Geom2d_Curve)::DownCast(result->Translated(gp_Vec2d(0.0, v_period)));
        }
    }

    curve = result;
    tolerance = myTolerance;

    // The modifier leaves the range of the edges it rebuilds empty, restore it
    // so the edges can be evaluated and meshed.
    BRep_Builder builder;
    builder.Range(new_edge, first, last);

    return Standard_True;
}

GeomAbs_Shape PointMapModification::Continuity(const TopoDS_Edge&, const TopoDS_Face&,
                                               const TopoDS_Face&, const TopoDS_Edge&,
                                               const TopoDS_Face&, const TopoDS_Face&) {
    return GeomAbs_C0;
}

TopoDS_Shape PointMapModification::forward(const TopoDS_Edge& edge) {
    return TopoDS::Edge(edge).Oriented(TopAbs_FORWARD);
}

} // namespace dslcad
