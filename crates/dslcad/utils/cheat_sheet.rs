use dslcad_api::protocol::Message;
use dslcad_api::Client;

fn main() {
    let client = Client::new(dslcad::server);
    let result = client.send(Message::CheatSheet()).busy_loop();
    match result {
        Message::CheatSheetResults { cheatsheet } => {
            println!("{}", cheatsheet)
        }
        _ => panic!("unexpected response from server"),
    }
}
