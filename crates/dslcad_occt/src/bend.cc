// Bends a shape around an axis by wrapping one of its extents into a circular
// arc. The deformation is applied to the B-Rep geometry with BRepTools_Modifier,
// so curves and surfaces are rebuilt as B-splines (no triangulation involved).
#include <cmath>
#include <memory>
#include <utility>
#include <vector>

#include "point_map.hxx"

#include <Bnd_Box.hxx>
#include <BRepBndLib.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepLib.hxx>
#include <BRepTools_Modifier.hxx>
#include <GProp_GProps.hxx>
#include <ShapeFix_Shape.hxx>
#include <Standard_Failure.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Solid.hxx>
#include <gp_Pnt.hxx>

namespace {

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


// A full 360 degree bend maps the ends of the shape onto each other, leaving
// two coincident end faces inside the result as an internal wall. Drop those
// faces and stitch the rest back together, so the bend is a single continuous
// solid instead of two halves touching along a face.
TopoDS_Shape merge_wrapped_ends(const TopoDS_Shape& shape, double tolerance, double diagonal) {
    std::vector<TopoDS_Face> faces;
    std::vector<double> areas;
    std::vector<gp_Pnt> centers;
    std::vector<int> edge_counts;
    for (TopExp_Explorer it(shape, TopAbs_FACE); it.More(); it.Next()) {
        const TopoDS_Face face = TopoDS::Face(it.Current());
        GProp_GProps props;
        BRepGProp::SurfaceProperties(face, props);
        int edge_count = 0;
        for (TopExp_Explorer edge_it(face, TopAbs_EDGE); edge_it.More(); edge_it.Next()) {
            edge_count++;
        }
        faces.push_back(face);
        areas.push_back(props.Mass());
        centers.push_back(props.CentreOfMass());
        edge_counts.push_back(edge_count);
    }

    // Coincident end faces have the same geometry, so they match on area,
    // centroid and number of edges. The tolerance scales with the shape since
    // the two ends are fitted independently.
    const double center_tolerance = std::max(diagonal * 1e-4, tolerance * 10.0);
    std::vector<bool> drop(faces.size(), false);
    for (size_t i = 0; i < faces.size(); ++i) {
        for (size_t j = i + 1; j < faces.size(); ++j) {
            if (drop[i] || drop[j] || edge_counts[i] != edge_counts[j]) {
                continue;
            }
            const double largest = std::max(std::fabs(areas[i]), std::fabs(areas[j]));
            if (centers[i].Distance(centers[j]) < center_tolerance &&
                std::fabs(areas[i] - areas[j]) < std::max(largest * 1e-6, tolerance * tolerance)) {
                drop[i] = drop[j] = true;
            }
        }
    }

    bool found = false;
    for (bool dropped : drop) {
        found = found || dropped;
    }
    if (!found) {
        return shape;
    }

    BRepBuilderAPI_Sewing sewing(center_tolerance);
    for (size_t i = 0; i < faces.size(); ++i) {
        if (!drop[i]) {
            sewing.Add(faces[i]);
        }
    }
    sewing.Perform();
    if (sewing.NbFreeEdges() != 0 || sewing.SewedShape().IsNull()) {
        return shape;
    }

    TopoDS_Shape sewn = sewing.SewedShape();
    int shell_count = 0;
    TopoDS_Shell shell;
    for (TopExp_Explorer shell_it(sewn, TopAbs_SHELL); shell_it.More(); shell_it.Next()) {
        shell = TopoDS::Shell(shell_it.Current());
        shell_count++;
    }
    if (shell_count != 1) {
        return shape;
    }
    BRepBuilderAPI_MakeSolid make_solid(shell);
    if (!make_solid.IsDone()) {
        return shape;
    }

    TopoDS_Shape solid = make_solid.Solid();
    if (!BRepCheck_Analyzer(solid).IsValid()) {
        return shape;
    }

    return solid;
}

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

        dslcad::PointMapModification::Map map = [&bend](const gp_Pnt& point) {
            return bend_point(bend, point);
        };
        Handle(dslcad::PointMapModification) modification =
            new dslcad::PointMapModification(map, bend.tolerance, bend.scale);
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

        // A full 360 degree bend wraps the shape back onto itself, so the two
        // ends touch. The end faces then sit on top of each other inside the
        // result as internal walls. Merge them away and stitch the shape back
        // together so the bend forms a single continuous solid.
        if (std::fabs(std::fabs(degrees) - 360.0) < 1e-9) {
            try {
                result = merge_wrapped_ends(result, bend.tolerance, diagonal);
            } catch (const Standard_Failure&) {
                // Keep the unmerged shape if the seam could not be stitched.
            }
        }

        return new TopoDS_Shape(result);
    } catch (const Standard_Failure&) {
        return nullptr;
    } catch (...) {
        return nullptr;
    }
}
