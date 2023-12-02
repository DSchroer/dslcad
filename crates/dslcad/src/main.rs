use clap::{Parser, Subcommand};
use dslcad::library::Library;
use dslcad::parser::{Ast, DocId};
use dslcad::{parse, render};
use persistence::threemf::ThreeMF;
use std::env;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Render {
        /// Source path to load
        source: String,
        #[cfg(feature = "preview")]
        #[arg(short, long)]
        preview: bool,
    },
    Cheatsheet {},
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Render { source, preview } => {
            #[cfg(feature = "preview")]
            if preview {
                return render_to_preview(&source);
            }

            render_to_file(&source)
        }
        Commands::Cheatsheet {} => {
            println!("{}", Library::default());
            Ok(())
        }
    }
}

fn render_to_file(source: &String) -> Result<(), Box<dyn Error>> {
    let render = render(parse(source.clone())?)?;

    println!("{}", &render.stdout);

    let cwd = env::current_dir()?;
    let file = Path::new(source).file_stem().unwrap();
    let outpath = cwd.join(format!("{}.3mf", file.to_string_lossy()));

    let threemf: ThreeMF = render.into();
    let out = File::create(outpath)?;
    threemf.write_to_zip(out)?;

    Ok(())
}

#[cfg(feature = "preview")]
fn render_to_preview(source: &str) -> Result<(), Box<dyn Error>> {
    use notify::{recommended_watcher, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use preview::{Preview, PreviewHandle};
    use std::sync::{Arc, Mutex};

    fn add_files_to_watch(watch: Arc<Mutex<Option<RecommendedWatcher>>>, ast: &Ast) {
        let to_watch: Vec<DocId> = ast.documents.keys().cloned().collect();
        std::thread::spawn(move || {
            let mut guard = watch.lock().unwrap();
            let watcher = guard.as_mut().unwrap();
            for new_path in to_watch {
                let buf = new_path.to_path().to_path_buf();
                watcher.watch(&buf, RecursiveMode::NonRecursive).unwrap();
            }
        });
    }

    fn render_with_watcher(
        source: &str,
        handle: PreviewHandle,
        watch: Arc<Mutex<Option<RecommendedWatcher>>>,
    ) -> Result<(), Box<dyn Error>> {
        match parse(source.to_string()) {
            Ok(ast) => {
                add_files_to_watch(watch, &ast);
                match render(ast) {
                    Ok(r) => handle.show_render(r)?,
                    Err(e) => handle.show_error(e.to_string())?,
                }
            }
            Err(e) => handle.show_error(e.to_string())?,
        }
        Ok(())
    }

    let (preview, handle) = Preview::new();
    let watch = Arc::new(Mutex::new(None));

    let watcher = {
        let (source, watch, handle) = (source.to_string(), watch.clone(), handle.clone());
        recommended_watcher(move |event| {
            if let Ok(notify::Event {
                kind: EventKind::Modify(_),
                ..
            }) = event
            {
                render_with_watcher(&source, handle.clone(), watch.clone()).unwrap()
            }
        })?
    };

    {
        let mut g = watch.lock().unwrap();
        g.replace(watcher);
    }

    render_with_watcher(source, handle.clone(), watch.clone())?;

    preview.open(Library::default().to_string());
    Ok(())
}
