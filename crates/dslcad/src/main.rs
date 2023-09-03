use clap::Parser;
use dslcad::library::Library;
use dslcad::parser::{Ast, DocId};
use dslcad::{parse, render};
use std::error::Error;
use std::path::Path;
use std::{env, fs};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source path to load
    source: String,
    #[cfg(feature = "preview")]
    #[arg(short, long)]
    preview: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    #[cfg(feature = "preview")]
    if args.preview {
        return render_to_preview(args);
    }

    render_to_file(args)
}

fn render_to_file(args: Args) -> Result<(), Box<dyn Error>> {
    let render = render(parse(args.source.clone())?)?;

    let cwd = env::current_dir()?;
    let file = Path::new(&args.source).file_stem().unwrap();
    let outpath = cwd.join(format!("{}.parts", file.to_string_lossy()));

    fs::write(outpath, render.to_bytes())?;

    Ok(())
}

#[cfg(feature = "preview")]
fn render_to_preview(args: Args) -> Result<(), Box<dyn Error>> {
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
        args: Args,
        handle: PreviewHandle,
        watch: Arc<Mutex<Option<RecommendedWatcher>>>,
    ) -> Result<(), Box<dyn Error>> {
        match parse(args.source.clone()) {
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
        let (args, watch, handle) = (args.clone(), watch.clone(), handle.clone());
        recommended_watcher(move |event| {
            if let Ok(notify::Event {
                kind: EventKind::Modify(_),
                ..
            }) = event
            {
                render_with_watcher(args.clone(), handle.clone(), watch.clone()).unwrap()
            }
        })?
    };

    {
        let mut g = watch.lock().unwrap();
        g.replace(watcher);
    }

    render_with_watcher(args.clone(), handle.clone(), watch.clone())?;

    preview.open(Library::new().to_string());
    Ok(())
}
