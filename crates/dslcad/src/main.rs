use clap::{Parser, ValueEnum};
use dslcad::error_printer::ErrorPrinter;
use dslcad::library::Library;
use dslcad::parser::{DocumentParseError, ParseError};
use dslcad::reader::FsReader;
use dslcad::runtime::{RuntimeError, WithStack};
use dslcad::{eval, parse, parse_arguments, render};
use dslcad_storage::protocol::{BincodeError, Render};
use dslcad_storage::threemf::{ThreeMF, ThreeMFError};
use dslcad_viewer::PreviewHandle;
use log::info;
use std::env;
use std::fs::File;
use std::io::{stderr, Write};
use std::path::Path;
use thiserror::Error;

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

#[derive(Debug, Error)]
enum CliError {
    #[error(transparent)]
    ArgParse(#[from] DocumentParseError),
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Runtime(#[from] WithStack<RuntimeError>),
    #[error(transparent)]
    Render(#[from] RuntimeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ThreeMf(#[from] ThreeMFError),
    #[error(transparent)]
    Bincode(#[from] BincodeError),
    #[error(transparent)]
    Notify(#[from] notify::Error),
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
                    handle_error(e, &mut stderr()).unwrap();
                }
                return;
            }

            if let Err(e) =
                render_to_file(&args.source, args.argument, args.deflection, args.output)
            {
                handle_error(e, &mut stderr()).unwrap();
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

fn handle_error(error: CliError, writer: &mut impl Write) -> Result<(), std::io::Error> {
    let printer = ErrorPrinter::new(FsReader);

    match error {
        CliError::Parse(parse_error) => printer.print_parse_error(writer, &parse_error),
        CliError::Runtime(runtime_error) => printer.print_runtime_error(writer, &runtime_error),
        _ => printer.print_error(writer, &error),
    }
}

fn render_to_file(
    source: &String,
    arguments: Vec<String>,
    deflection: f64,
    output: Output,
) -> Result<(), CliError> {
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
) -> Result<(), CliError> {
    use dslcad::parser::{Ast, DocId};
    use dslcad_viewer::Preview;
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
        watch: Arc<Mutex<Option<RecommendedWatcher>>>,
    ) -> Result<Render, CliError> {
        let ast = parse(source.to_string())?;
        add_files_to_watch(watch, &ast);
        let arguments = parse_arguments(arguments.iter().map(|i| i.as_str()))?;
        let render = render(eval(ast, arguments)?, deflection)?;
        Ok(render)
    }

    fn render_to_handle(
        handle: PreviewHandle,
        source: &str,
        arguments: &[String],
        deflection: f64,
        watch: Arc<Mutex<Option<RecommendedWatcher>>>,
    ) {
        handle.show_rendering();
        match render_with_watcher(source, arguments, deflection, watch) {
            Ok(render) => handle.show_render(render),
            Err(err) => {
                let mut buffer = Vec::new();
                handle_error(err, &mut buffer).unwrap();
                handle.show_error(String::from_utf8(buffer).unwrap());
            }
        }
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
                render_to_handle(
                    handle.clone(),
                    &source,
                    &arguments,
                    deflection,
                    watch.clone(),
                );
            }
        })?
    };

    {
        let mut g = watch.lock().unwrap();
        g.replace(watcher);
    }

    let source = source.to_string();
    std::thread::spawn(move || {
        render_to_handle(handle, &source, &arguments, deflection, watch.clone())
    });

    preview.open(Library::default().to_string());
    Ok(())
}
