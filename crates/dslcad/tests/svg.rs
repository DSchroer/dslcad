use dslcad::{eval, parse, render};
use dslcad_storage::protocol::{Mesh, Part};
use std::collections::HashMap;

#[test]
fn it_can_extrude_svg_files() {
    let ast =
        parse("../../examples/svg_import/stamp-heat-mesh.ds".to_string()).expect("failed to parse");
    let value = eval(ast, HashMap::new()).expect("failed to evaluate");
    let output = render(value, 0.1).expect("failed to render");

    assert_eq!(1, output.parts.len());

    match &output.parts[0] {
        Part::Object { mesh, .. } => {
            assert!(!mesh.vertices.is_empty());
            assert!(!mesh.triangles.is_empty());

            let max_z = mesh
                .vertices
                .iter()
                .map(|vertex| vertex[2])
                .fold(f64::MIN, f64::max);
            assert!((max_z - 2.0).abs() < 0.01);

            let volume = mesh_volume(mesh);
            assert!((volume - 1285.3).abs() / 1285.3 < 0.05);
        }
        other => panic!("expected an object, got {other:?}"),
    }
}

#[test]
fn it_can_bend_a_small_traced_svg() {
    let ast = parse("../../examples/svg_ring/trace_ring.ds".to_string()).expect("failed to parse");
    let value = eval(ast, HashMap::new()).expect("failed to evaluate");
    let output = render(value, 0.1).expect("failed to render");

    assert_eq!(1, output.parts.len());
}

fn mesh_volume(mesh: &Mesh) -> f64 {
    let mut volume = 0.0;

    for triangle in &mesh.triangles {
        let a = mesh.vertices[triangle[0]];
        let b = mesh.vertices[triangle[1]];
        let c = mesh.vertices[triangle[2]];

        volume += (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
            + a[2] * (b[0] * c[1] - b[1] * c[0]))
            / 6.0;
    }

    volume.abs()
}
