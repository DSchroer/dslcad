// Tapers the walls of a shape that run along one axis so they lean inward along
// one or more other axes. Every wall facing a selected axis has its top edge
// translated inward along that axis by height * tan(angle), while the base of
// the shape stays in place. The displacement is along the selected axes only,
// so a taper along y never moves geometry along x. A shared edge or vertex
// blends the walls around it, so a corner between two tapered walls moves along
// both of their axes while the shape stays connected.
#include <algorithm>
#include <cmath>

#include <Bnd_Box.hxx>
#include <BRepBndLib.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepLib.hxx>
#include <BRepTools.hxx>
#include <BRepTools_Modifier.hxx>
#include <BRep_Tool.hxx>
#include <GeomLProp_SLProps.hxx>
#include <Geom_Surface.hxx>
#include <NCollection_DataMap.hxx>
#include <ShapeFix_Shape.hxx>
#include <Standard_Failure.hxx>
#include <TopAbs_Orientation.hxx>
#include <TopExp_Explorer.hxx>
#include <TopLoc_Location.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_XYZ.hxx>

#include "point_map.hxx"

namespace {

// A wall counts as running along the up axis when its normal is nearly
// perpendicular to it.
const double MAX_UP_COMPONENT = 0.1;

// The direction every wall around a shared edge or vertex pushes it in, added
// up per selected axis. The strongest wall around it breaks a tie when two
// opposite walls meet, so the corner still moves instead of standing vertical.
struct Blend {
    double sum[3];
    double weight[3];
    double sign[3];
};

// The outward normal of a face at the middle of its UV range. Returns false for
// faces whose normal is undefined, for example the apex of a cone.
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
    if (face.Orientation() == TopAbs_REVERSED) {
        normal.Reverse();
    }
    return true;
}

using DirectionMap = NCollection_DataMap<TopoDS_Shape, gp_XYZ, TopTools_ShapeMapHasher>;
using BlendMap = NCollection_DataMap<TopoDS_Shape, Blend, TopTools_ShapeMapHasher>;

void blend_direction(BlendMap& map, const TopoDS_Shape& key, const double direction[3],
                     const double weight[3]) {
    Blend blend;
    if (map.IsBound(key)) {
        blend = map.Find(key);
    } else {
        for (int axis = 0; axis < 3; ++axis) {
            blend.sum[axis] = 0.0;
            blend.weight[axis] = 0.0;
            blend.sign[axis] = 0.0;
        }
    }

    for (int axis = 0; axis < 3; ++axis) {
        if (direction[axis] == 0.0) {
            continue;
        }

        blend.sum[axis] += direction[axis] * weight[axis];
        if (weight[axis] > blend.weight[axis]) {
            blend.weight[axis] = weight[axis];
            blend.sign[axis] = direction[axis];
        }
    }

    map.Bind(key, blend);
}

gp_XYZ blend_result(const Blend& blend) {
    gp_XYZ direction(0.0, 0.0, 0.0);
    for (int axis = 0; axis < 3; ++axis) {
        if (blend.sum[axis] > 1e-12) {
            direction.SetCoord(axis + 1, 1.0);
        } else if (blend.sum[axis] < -1e-12) {
            direction.SetCoord(axis + 1, -1.0);
        } else if (blend.weight[axis] > 0.0) {
            direction.SetCoord(axis + 1, blend.sign[axis]);
        }
    }
    return direction;
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

        const int up = up_axis;
        const double base = minimum[up];
        const double tangent = std::tan(degrees * M_PI / 180.0);
        const gp_Dir up_direction(up == 0 ? 1.0 : 0.0, up == 1 ? 1.0 : 0.0,
                                  up == 2 ? 1.0 : 0.0);

        const double tolerance = std::max(diagonal * 1e-6, 1e-9);
        const double scale = std::max(diagonal, 1e-9);

        // The direction each wall's top edge moves along. A wall facing a
        // selected axis moves inward along it; a wall facing the opposite way
        // moves the other way. Walls running along the up axis only, and walls
        // that do not face a selected axis, stay in place.
        DirectionMap faces;
        BlendMap edges;
        BlendMap vertices;

        for (TopExp_Explorer it(input, TopAbs_FACE); it.More(); it.Next()) {
            const TopoDS_Face face = TopoDS::Face(it.Current());

            double direction[3] = {0.0, 0.0, 0.0};
            double weight[3] = {0.0, 0.0, 0.0};
            gp_Dir normal;
            if (face_normal(face, normal) &&
                std::fabs(normal.Dot(up_direction)) <= MAX_UP_COMPONENT) {
                const double components[3] = {normal.X(), normal.Y(), normal.Z()};
                for (int axis = 0; axis < 3; ++axis) {
                    if (axis == up || (directions & (1 << axis)) == 0) {
                        continue;
                    }
                    if (std::fabs(components[axis]) > 1e-9) {
                        direction[axis] = components[axis] > 0.0 ? -1.0 : 1.0;
                        weight[axis] = std::fabs(components[axis]);
                    }
                }
            }

            gp_XYZ face_direction(0.0, 0.0, 0.0);
            for (int axis = 0; axis < 3; ++axis) {
                face_direction.SetCoord(axis + 1, direction[axis]);
            }
            faces.Bind(face, face_direction);

            for (TopExp_Explorer edge(face, TopAbs_EDGE); edge.More(); edge.Next()) {
                blend_direction(edges, TopoDS::Edge(edge.Current()), direction, weight);
            }
            for (TopExp_Explorer vertex(face, TopAbs_VERTEX); vertex.More(); vertex.Next()) {
                blend_direction(vertices, TopoDS::Vertex(vertex.Current()), direction, weight);
            }
        }

        // Translate a point along the selected axes by the accumulated wall
        // direction. The amount grows with the height above the base, so the
        // base stays in place and every wall leans by the same angle.
        auto translate = [&](const gp_XYZ& direction, const gp_Pnt& point) -> gp_Pnt {
            const double coordinates[3] = {point.X(), point.Y(), point.Z()};
            const double amount = (coordinates[up] - base) * tangent;

            double moved[3] = {coordinates[0], coordinates[1], coordinates[2]};
            for (int axis = 0; axis < 3; ++axis) {
                moved[axis] += direction.Coord(axis + 1) * amount;
            }

            return gp_Pnt(moved[0], moved[1], moved[2]);
        };

        auto blended = [](const BlendMap& map, const TopoDS_Shape& key) {
            return blend_result(map.Find(key));
        };

        dslcad::PointMapModification::Map identity = [](const gp_Pnt& point) { return point; };
        Handle(dslcad::PointMapModification) modification = new dslcad::PointMapModification(
            identity,
            [&](const TopoDS_Face& face, const gp_Pnt& point) {
                return translate(faces.Find(face), point);
            },
            [&](const TopoDS_Edge& edge, const gp_Pnt& point) {
                return translate(blended(edges, edge), point);
            },
            [&](const TopoDS_Vertex& vertex, const gp_Pnt& point) {
                return translate(blended(vertices, vertex), point);
            },
            tolerance, scale);

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
        // those pcurves is cheaper than a full ShapeFix pass, so only fall back
        // to ShapeFix when the shape stays invalid.
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
