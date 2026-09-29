#pragma once
//
// Rebuilds every curve and surface of a shape from points mapped through an
// arbitrary point map, using B-splines fitted within a tolerance relative to
// the shape's size. Used by deformations such as bend and taper that cannot be
// expressed as a rigid transformation.

#include <functional>

#include <BRepTools_Modification.hxx>
#include <Geom2d_Curve.hxx>
#include <Geom_Curve.hxx>
#include <Geom_Surface.hxx>
#include <NCollection_DataMap.hxx>
#include <Standard_Real.hxx>
#include <Standard_Boolean.hxx>
#include <GeomAbs_Shape.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopLoc_Location.hxx>
#include <gp_Pnt.hxx>

namespace dslcad {

class PointMapModification : public BRepTools_Modification {
public:
    using Map = std::function<gp_Pnt(const gp_Pnt&)>;
    using SurfaceMap = std::function<gp_Pnt(const TopoDS_Face&, const gp_Pnt&)>;
    using CurveMap = std::function<gp_Pnt(const TopoDS_Edge&, const gp_Pnt&)>;
    using VertexMap = std::function<gp_Pnt(const TopoDS_Vertex&, const gp_Pnt&)>;

    PointMapModification(Map map, Standard_Real tolerance, Standard_Real scale);

    PointMapModification(Map map, SurfaceMap surface, CurveMap curve, VertexMap vertex,
                         Standard_Real tolerance, Standard_Real scale);

    Standard_Boolean NewSurface(const TopoDS_Face& face, Handle(Geom_Surface)& surface,
                                TopLoc_Location& location, Standard_Real& tolerance,
                                Standard_Boolean& reverse_wires,
                                Standard_Boolean& reverse_face) override;

    Standard_Boolean NewCurve(const TopoDS_Edge& edge, Handle(Geom_Curve)& curve,
                              TopLoc_Location& location, Standard_Real& tolerance) override;

    Standard_Boolean NewPoint(const TopoDS_Vertex& vertex, gp_Pnt& point,
                              Standard_Real& tolerance) override;

    Standard_Boolean NewParameter(const TopoDS_Vertex& vertex, const TopoDS_Edge& edge,
                                  Standard_Real& parameter, Standard_Real& tolerance) override;

    Standard_Boolean NewCurve2d(const TopoDS_Edge& edge, const TopoDS_Face& face,
                                const TopoDS_Edge& new_edge, const TopoDS_Face& new_face,
                                Handle(Geom2d_Curve)& curve,
                                Standard_Real& tolerance) override;

    GeomAbs_Shape Continuity(const TopoDS_Edge&, const TopoDS_Face&, const TopoDS_Face&,
                             const TopoDS_Edge&, const TopoDS_Face&, const TopoDS_Face&) override;

    bool Failed() const { return myFailed; }

private:
    static TopoDS_Shape forward(const TopoDS_Edge& edge);

    Map myPoint;
    SurfaceMap mySurface;
    CurveMap myCurve;
    VertexMap myVertex;
    Standard_Real myTolerance;
    Standard_Real myScale;
    bool myFailed;
    NCollection_DataMap<TopoDS_Shape, Handle(Geom_Curve)> myCurves;
};

} // namespace dslcad
