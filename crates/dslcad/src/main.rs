use clap::{Parser, ValueEnum};
use dslcad::library::Library;
use dslcad::parser::{Ast, DocId};
use dslcad::{parse, render};
use persistence::threemf::ThreeMF;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source path to load
    source: String,

    #[cfg(feature = "preview")]
    #[arg(short, long)]
    /// Display preview window for editing
    preview: bool,

    #[arg(short, long, default_value_t = 0.01)]
    /// Deflection used to calculate mesh. Smaller numbers are more detailed.
    deflection: f64,

    #[arg(short, long, value_enum)]
    /// Deflection used to calculate mesh. Smaller numbers are more detailed.
    output: Output,

    #[command(flatten)]
    cheatsheet: Cheatsheet,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cheatsheet {
    #[arg(long)]
    /// Print the cheatsheet
    cheatsheet: bool,
}

#[derive(Debug, Clone, Default, ValueEnum)]
enum Output {
    #[default]
    #[value(name = "3mf")]
    ThreeMf,
    Raw,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    match Args::try_parse() {
        Ok(Args {
            preview,
            source,
            deflection,
            output,
            ..
        }) => {
            #[cfg(feature = "preview")]
            if preview {
                return render_to_preview(&source, deflection);
            }

            render_to_file(&source, deflection, output)
        }
        Err(e) => {
            if let Ok(Cheatsheet { cheatsheet: true }) = Cheatsheet::try_parse() {
                print_cheatsheet()
            } else {
                e.exit();
            }
        }
    }
}

fn print_cheatsheet() -> Result<(), Box<dyn Error>> {
    let lib = Library::default();
    println!("{}", lib);
    Ok(())
}

fn render_to_file(source: &String, deflection: f64, output: Output) -> Result<(), Box<dyn Error>> {
    let render = render(parse(source.clone())?, deflection)?;

    if !render.stdout.is_empty() {
        println!("{}", &render.stdout);
    }

    let cwd = env::current_dir()?;
    let file = Path::new(source).file_stem().unwrap();

    match output {
        Output::ThreeMf => {
            let outpath = cwd.join(format!("{}.3mf", file.to_string_lossy()));
            let threemf: ThreeMF = render.into();
            let out = File::create(outpath)?;
            threemf.write_to_zip(out)?;
        }
        Output::Raw => {
            let outpath = cwd.join(format!("{}.bin", file.to_string_lossy()));
            let raw: Vec<u8> = render.try_into()?;
            let mut out = File::create(outpath)?;
            out.write_all(&raw)?
        }
    }

    Ok(())
}

#[cfg(feature = "preview")]
fn render_to_preview(source: &str, deflection: f64) -> Result<(), Box<dyn Error>> {
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
        deflection: f64,
        handle: PreviewHandle,
        watch: Arc<Mutex<Option<RecommendedWatcher>>>,
    ) -> Result<(), Box<dyn Error>> {
        handle.show_rendering()?;
        match parse(source.to_string()) {
            Ok(ast) => {
                add_files_to_watch(watch, &ast);
                match render(ast, deflection) {
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
                render_with_watcher(&source, deflection, handle.clone(), watch.clone()).unwrap()
            }
        })?
    };

    {
        let mut g = watch.lock().unwrap();
        g.replace(watcher);
    }

    let source = source.to_string();
    std::thread::spawn(move || {
        render_with_watcher(&source, deflection, handle.clone(), watch.clone()).unwrap();
    });

    preview.open(Library::default().to_string());
    Ok(())
}
