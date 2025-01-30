use clap::{Parser, ValueEnum};
use dslcad::library::Library;
use dslcad::{eval, parse, parse_arguments, render};
use dslcad_storage::threemf::ThreeMF;
use log::info;
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
    /// Display dslcad_viewer window for editing
    preview: bool,

    #[arg(short, long)]
    /// Arguments for the script (examples: "foo=5", "name=\"bob\"")
    argument: Vec<String>,

    #[arg(short, long, default_value_t = 0.01)]
    /// Deflection used to calculate mesh (smaller = more detail)
    deflection: f64,

    #[arg(short, long, value_enum, default_value = "3mf")]
    /// Output file format
    output: Output,

    #[arg(short, long)]
    /// Log filter
    log: Option<String>,

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

fn main() {
    match Args::try_parse() {
        Ok(args) => {
            if let Some(log) = &args.log {
                env_logger::builder().parse_filters(log).init();
            }

            #[cfg(feature = "preview")]
            if args.preview {
                if let Err(e) = render_to_preview(&args.source, args.argument, args.deflection) {
                    eprintln!("{}", e);
                }
                return;
            }

            if let Err(e) =
                render_to_file(&args.source, args.argument, args.deflection, args.output)
            {
                eprintln!("{}", e);
            }
        }
        Err(e) => {
            if let Ok(Cheatsheet { cheatsheet: true }) = Cheatsheet::try_parse() {
                println!("{}", Library::default());
            } else {
                e.exit();
            }
        }
    }
}

fn render_to_file(
    source: &String,
    arguments: Vec<String>,
    deflection: f64,
    output: Output,
) -> Result<(), Box<dyn Error>> {
    let arguments = parse_arguments(arguments.iter().map(|i| i.as_str()))?;
    let eval_result = eval(parse(source.clone())?, arguments)?;

    let text_output = eval_result.to_text().unwrap_or_default();
    if !text_output.is_empty() {
        println!("{}", &text_output);
    }

    let cwd = env::current_dir()?;
    let file = Path::new(source).file_stem().unwrap();

    let outfile = match output {
        Output::ThreeMf => {
            let render = render(eval_result, deflection)?;

            let outpath = cwd.join(format!("{}.3mf", file.to_string_lossy()));
            let threemf: ThreeMF = render.into();
            let out = File::create(&outpath)?;
            threemf.write_to_zip(out)?;
            outpath
        }
        Output::Raw => {
            let render = render(eval_result, deflection)?;

            let outpath = cwd.join(format!("{}.bin", file.to_string_lossy()));
            let raw: Vec<u8> = render.try_into()?;
            let mut out = File::create(&outpath)?;
            out.write_all(&raw)?;
            outpath
        }
    };

    info!("output written to {}", outfile.to_string_lossy());

    Ok(())
}

#[cfg(feature = "preview")]
fn render_to_preview(
    source: &str,
    arguments: Vec<String>,
    deflection: f64,
) -> Result<(), Box<dyn Error>> {
    use dslcad::parser::{Ast, DocId};
    use dslcad_viewer::{Preview, PreviewHandle};
    use notify::{recommended_watcher, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
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
        arguments: &[String],
        deflection: f64,
        handle: PreviewHandle,
        watch: Arc<Mutex<Option<RecommendedWatcher>>>,
    ) -> Result<(), Box<dyn Error>> {
        handle.show_rendering()?;
        match parse(source.to_string()) {
            Ok(ast) => {
                add_files_to_watch(watch, &ast);
                let arguments = parse_arguments(arguments.iter().map(|i| i.as_str()))?;
                match render(eval(ast, arguments)?, deflection) {
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
        let (source, arguments, watch, handle) = (
            source.to_string(),
            arguments.clone(),
            watch.clone(),
            handle.clone(),
        );
        recommended_watcher(move |event| {
            if let Ok(notify::Event {
                kind: EventKind::Modify(_),
                ..
            }) = event
            {
                render_with_watcher(
                    &source,
                    &arguments,
                    deflection,
                    handle.clone(),
                    watch.clone(),
                )
                .unwrap()
            }
        })?
    };

    {
        let mut g = watch.lock().unwrap();
        g.replace(watcher);
    }

    let source = source.to_string();
    std::thread::spawn(move || {
        render_with_watcher(
            &source,
            &arguments,
            deflection,
            handle.clone(),
            watch.clone(),
        )
        .unwrap();
    });

    preview.open(Library::default().to_string());
    Ok(())
}
