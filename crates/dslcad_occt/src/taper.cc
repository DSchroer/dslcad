// Tapers the walls of a shape that run along one axis so they lean inward along
// one or more other axes. Every wall facing a selected direction is rotated
// about its intersection with the neutral plane at the base of the shape, which
// translates its top edge inward by height * tan(angle) without scaling the
// cross-section. BRepOffsetAPI_DraftAngle rebuilds the walls and the faces next
// to them, so the shape stays closed.
#include <cmath>

#include <Bnd_Box.hxx>
#include <BRepBndLib.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepLib.hxx>
#include <BRepOffsetAPI_DraftAngle.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <GeomLProp_SLProps.hxx>
#include <Geom_Surface.hxx>
#include <ShapeFix_Shape.hxx>
#include <Standard_Failure.hxx>
#include <TopExp_Explorer.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Dir.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>

namespace {

// A wall counts as running along the up axis when its normal is nearly
// perpendicular to it, and as facing a movement axis when its normal points
// mostly along that axis.
const double MAX_UP_COMPONENT = 0.1;
const double MIN_MOVE_COMPONENT = 0.5;

gp_Dir axis_direction(int axis) {
    return gp_Dir(axis == 0 ? 1.0 : 0.0, axis == 1 ? 1.0 : 0.0, axis == 2 ? 1.0 : 0.0);
}

// Normal of a face at the middle of its UV range. Returns false for faces
// whose normal is undefined, for example the apex of a cone.
bool face_normal(const TopoDS_Face& face, gp_Dir& normal) {
    Standard_Real u1, u2, v1, v2;
    BRepTools::UVBounds(face, u1, u2, v1, v2);

    TopLoc_Location location;
    Handle(Geom_Surface) surface = BRep_Tool::Surface(face, location);
    if (surface.IsNull()) {
        return false;
    }

    GeomLProp_SLProps properties(surface, (u1 + u2) / 2.0, (v1 + v2) / 2.0, 1, 1e-9);
    if (!properties.IsNormalDefined()) {
        return false;
    }

    normal = properties.Normal().Transformed(location.Transformation());
    return true;
}

} // namespace

extern "C" void* dslcad_taper_shape(const void* shape, int up_axis, int directions,
                                    double degrees) {
    if (shape == nullptr || up_axis < 0 || up_axis > 2 || (directions & 0x7) == 0 ||
        degrees == 0.0) {
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

        const double diagonal = std::sqrt(
            (xmax - xmin) * (xmax - xmin) + (ymax - ymin) * (ymax - ymin) +
            (zmax - zmin) * (zmax - zmin));

        const gp_Dir up = axis_direction(up_axis);
        const gp_Dir axes[3] = {axis_direction(0), axis_direction(1), axis_direction(2)};
        const double angle = degrees * M_PI / 180.0;

        // The neutral plane sits at the base of the shape, so the taper
        // translates the top of each wall and leaves the bottom in place.
        gp_Pnt neutral(0.0, 0.0, 0.0);
        neutral.SetCoord(up_axis + 1, minimum[up_axis]);
        const gp_Pln plane(neutral, up);

        BRepOffsetAPI_DraftAngle draft(input);

        int drafted = 0;
        for (TopExp_Explorer it(input, TopAbs_FACE); it.More(); it.Next()) {
            const TopoDS_Face face = TopoDS::Face(it.Current());

            gp_Dir normal;
            if (!face_normal(face, normal)) {
                continue;
            }
            if (std::fabs(normal.Dot(up)) > MAX_UP_COMPONENT) {
                continue;
            }

            bool facing = false;
            for (int axis = 0; axis < 3; ++axis) {
                if (axis == up_axis || (directions & (1 << axis)) == 0) {
                    continue;
                }
                if (std::fabs(normal.Dot(axes[axis])) >= MIN_MOVE_COMPONENT) {
                    facing = true;
                    break;
                }
            }
            if (!facing) {
                continue;
            }

            draft.Add(face, up, angle, plane);
            if (draft.AddDone()) {
                drafted++;
            }
        }

        // No wall faces the requested direction, so there is nothing to taper.
        if (drafted == 0) {
            return new TopoDS_Shape(input);
        }

        draft.Build();
        if (!draft.IsDone()) {
            return nullptr;
        }

        TopoDS_Shape result = draft.Shape();
        if (result.IsNull()) {
            return nullptr;
        }

        // Drafting leaves some edges with a parameterization that no longer
        // matches their curve. Recomputing those is cheaper than a full
        // ShapeFix pass, so only fall back to ShapeFix when that is not enough.
        const double tolerance = std::max(diagonal * 1e-6, 1e-9);
        if (!BRepCheck_Analyzer(result).IsValid()) {
            BRepLib::SameParameter(result, tolerance, Standard_True);
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
