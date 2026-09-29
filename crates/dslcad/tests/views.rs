//! Views are empty by default, compose with `model()`, call each other, and
//! travel with imported documents.

use dslcad::{eval, parse, render};
use std::collections::HashMap;

fn render_source(path: &str) -> dslcad_storage::protocol::Render {
    let ast = parse(path.to_owned()).expect("parse");
    render(eval(ast, HashMap::new()).expect("eval"), 0.1).expect("render")
}

#[test]
fn imported_documents_export_their_views() {
    let render = render_source("../../examples/view_export/main.ds");

    let names: Vec<_> = render
        .views
        .iter()
        .filter_map(|view| view.name.clone())
        .collect();

    // The importer's own views plus the imported document's views.
    for expected in ["Assembly", "PlateElevation", "Front", "Top"] {
        assert!(names.contains(&expected.to_string()), "missing {expected}");
    }

    let elevation = render
        .views
        .iter()
        .find(|view| view.name.as_deref() == Some("PlateElevation"))
        .unwrap();

    // The imported `Front` view contributes geometry but not its camera: this
    // view declares its own framing.
    assert!(!elevation.parts.is_empty());
    assert_eq!(Some(90.0), elevation.angle.x);
}
