use dslcad::library::Library;
use std::fs;

#[test]
fn cheatsheet_docs_up_to_date() {
    let cheetsheet = Library::default().to_string();

    let reference =
        fs::read_to_string("../../docs/reference.md").expect("could not load reference docs");

    assert!(
        reference.contains(&cheetsheet),
        "reference docs out of date"
    )
}
