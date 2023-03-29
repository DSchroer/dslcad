use clap::Parser;
use crate::dslcad::server;
use dslcad_api::protocol::Message;
use dslcad_api::Client;
use std::error::Error;
use std::path::PathBuf;

/// model_script cad compiler
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source path to load
    source: String,

    /// Outdir
    #[arg(short, long, default_value = ".")]
    out: String,
}

pub(crate) fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let client: Client<Message> = Client::new(server);
    let output = client.send(Message::Render {
        path: args.source.clone(),
    }).busy_loop();

    let export = match output {
        Message::RenderResults(render, _) => {
            let render = render?;
            let file_name = PathBuf::from(&args.source);
            client.send(Message::Export {
                render,
                name: file_name.file_stem().unwrap().to_str().unwrap().to_string(),
                path: args.out,
            }).busy_loop()
        }
        _ => panic!("unexpected message {:?}", output),
    };

    match export {
        Message::ExportResults() => Ok(()),
        Message::Error(e) => Err(e.into()),
        _ => panic!("unexpected message {:?}", export),
    }
}
