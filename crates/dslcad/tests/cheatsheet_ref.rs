use dslcad_api::protocol::Message;
use dslcad_api::Client;
use std::fs;

#[test]
fn cheatsheet_docs_up_to_date() {
    let client = Client::new(dslcad::server);
    let result = client.send(Message::CheatSheet()).busy_loop();
    let cheetsheet = match result {
        Message::CheatSheetResults { cheatsheet } => cheatsheet,
        _ => panic!("unexpected response from server"),
    };

    let reference =
        fs::read_to_string("../../docs/reference.md").expect("could not load reference docs");

    assert!(
        reference.contains(&cheetsheet),
        "reference docs out of date"
    )
}
