use dslcad::Dslcad;
use std::fs;

#[test]
fn cheatsheet_docs_up_to_date() {
    let cheetsheet = Dslcad::default().cheat_sheet();

    let reference =
        fs::read_to_string("../../docs/reference.md").expect("could not load reference docs");

    assert!(
        reference.contains(&cheetsheet),
        "reference docs out of date"
    )
}
