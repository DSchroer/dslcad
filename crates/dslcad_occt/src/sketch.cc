// 2D sketch helpers: rounds or chamfers the corners of a planar face, and
// builds an ellipse edge.
#include <memory>

#include <BRepAdaptor_Curve.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepFilletAPI_MakeFillet2d.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Standard_Failure.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_ListIteratorOfListOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_Elips.hxx>
#include <gp_Pnt.hxx>
#include <gp_Vec.hxx>

namespace {

// Whether two edges meet smoothly at the vertex, for example the two arcs of
// a circle. Such corners can not be rounded or chamfered.
bool is_smooth(const TopoDS_Edge& edge1, const TopoDS_Edge& edge2, const TopoDS_Vertex& vertex) {
    try {
        BRepAdaptor_Curve curve1(edge1);
        BRepAdaptor_Curve curve2(edge2);

        gp_Pnt point;
        gp_Vec direction1;
        gp_Vec direction2;
        curve1.D1(BRep_Tool::Parameter(vertex, edge1), point, direction1);
        curve2.D1(BRep_Tool::Parameter(vertex, edge2), point, direction2);

        return direction1.IsParallel(direction2, 1e-6) || direction1.IsOpposite(direction2, 1e-6);
    } catch (const Standard_Failure&) {
        return false;
    }
}

// Rebuild a face's wires as a wire or compound of wires so the result can be
// used as a sketch again.
TopoDS_Shape wires_as_shape(const TopoDS_Shape& face) {
    int count = 0;
    TopoDS_Wire first;
    for (TopExp_Explorer explorer(face, TopAbs_WIRE); explorer.More(); explorer.Next()) {
        if (count == 0) {
            first = TopoDS::Wire(explorer.Current());
        }
        ++count;
    }

    if (count == 1) {
        return first;
    }

    TopoDS_Compound compound;
    BRep_Builder builder;
    builder.MakeCompound(compound);
    for (TopExp_Explorer explorer(face, TopAbs_WIRE); explorer.More(); explorer.Next()) {
        builder.Add(compound, explorer.Current());
    }
    return compound;
}

}  // namespace

// Rounds (chamfer = false) or chamfers (chamfer = true) every corner of the
// planar face with the given size.
extern "C" void* dslcad_fillet_2d(const void* face, double size, bool chamfer) {
    if (face == nullptr || size <= 0.0) {
        return nullptr;
    }

    const TopoDS_Shape& input = *static_cast<const TopoDS_Shape*>(face);

    try {
        BRepFilletAPI_MakeFillet2d make(TopoDS::Face(input));

        // Map each vertex to the edges that meet there so a vertex can be
        // rounded and a pair of edges can be chamfered.
        TopTools_IndexedDataMapOfShapeListOfShape map;
        TopExp::MapShapesAndAncestors(input, TopAbs_VERTEX, TopAbs_EDGE, map);

        int added = 0;
        for (int i = 1; i <= map.Extent(); ++i) {
            const TopoDS_Vertex& vertex = TopoDS::Vertex(map.FindKey(i));
            const TopTools_ListOfShape& edges = map.FindFromIndex(i);
            if (edges.Extent() != 2) {
                continue;
            }

            try {
                if (chamfer) {
                    TopTools_ListIteratorOfListOfShape it(edges);
                    const TopoDS_Edge& edge1 = TopoDS::Edge(it.Value());
                    it.Next();
                    const TopoDS_Edge& edge2 = TopoDS::Edge(it.Value());
                    if (is_smooth(edge1, edge2, vertex)) {
                        continue;
                    }
                    make.AddChamfer(edge1, edge2, size, size);
                } else {
                    const TopoDS_Edge edge1 = TopoDS::Edge(edges.First());
                    const TopoDS_Edge edge2 = TopoDS::Edge(edges.Last());
                    if (is_smooth(edge1, edge2, vertex)) {
                        continue;
                    }
                    make.AddFillet(vertex, size);
                }
                ++added;
            } catch (const Standard_Failure&) {
                // A corner that can not be rounded, leave it as it is.
            }
        }

        // Smooth outlines like a circle have no corners to round, return them
        // unchanged.
        if (added == 0) {
            return new TopoDS_Shape(wires_as_shape(input));
        }

        make.Build();
        if (!make.IsDone()) {
            return nullptr;
        }

        return new TopoDS_Shape(wires_as_shape(make.Shape()));
    } catch (const Standard_Failure&) {
        return nullptr;
    }
}

// A full ellipse edge centered on the origin in the xy plane.
extern "C" void* dslcad_ellipse_edge(double major, double minor) {
    if (major <= 0.0 || minor <= 0.0) {
        return nullptr;
    }

    try {
        gp_Elips ellipse(gp_Ax2(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), major, minor);
        BRepBuilderAPI_MakeEdge make(ellipse);
        if (!make.IsDone()) {
            return nullptr;
        }
        return new TopoDS_Shape(make.Edge());
    } catch (const Standard_Failure&) {
        return nullptr;
    }
}
