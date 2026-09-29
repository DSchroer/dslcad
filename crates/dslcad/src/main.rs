use clap::{Parser, Subcommand, ValueEnum};
use dslcad::error_printer::ErrorPrinter;
use dslcad::library::Library;
use dslcad::parser::{Ast, DocumentParseError, ParseError};
use dslcad::reader::{FsReader, StdinReader};
use dslcad::runtime::{RuntimeError, WithStack};
use dslcad::{eval, parse, parse_arguments, parse_with, render};
#[cfg(feature = "preview")]
use dslcad::{eval_with_cache, render_with_cache, Cache};
use dslcad_storage::protocol;
use dslcad_storage::protocol::BincodeError;
#[cfg(feature = "preview")]
use dslcad_storage::protocol::Render;
use dslcad_storage::threemf::{ThreeMF, ThreeMFError};
#[cfg(feature = "preview")]
use dslcad_viewer::PreviewHandle;
use log::info;
use std::env;
use std::fs::File;
use std::io::{stderr, Write};
use std::path::Path;
use thiserror::Error;

#[cfg(feature = "preview")]
const EXAMPLE_HELP: &str = "\
Examples:
  dslcad ./part.ds                  render part.ds to part.3mf
  dslcad ./part.ds --preview        open part.ds in the interactive preview
  dslcad ./part.ds -o stl           render to an STL instead of a 3MF
  dslcad ./part.ds -o step          render to a STEP instead of a 3MF
  dslcad ./part.ds -o png           render every declared view to a png
  dslcad ./part.ds -a size=5        render with the `size` script argument set to 5
  dslcad cheatsheet                 print the full syntax and function reference

Run `dslcad cheatsheet` for the language reference, and see
https://github.com/DSchroer/dslcad/tree/master/examples for example models.";

#[cfg(not(feature = "preview"))]
const EXAMPLE_HELP: &str = "\
Examples:
  dslcad ./part.ds                  render part.ds to part.3mf
  dslcad ./part.ds -o stl           render to an STL instead of a 3MF
  dslcad ./part.ds -o step          render to a STEP instead of a 3MF
  dslcad ./part.ds -a size=5        render with the `size` script argument set to 5
  dslcad cheatsheet                 print the full syntax and function reference

Run `dslcad cheatsheet` for the language reference, and see
https://github.com/DSchroer/dslcad/tree/master/examples for example models.";

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "Parametric CAD from code",
    long_about = "DSLCAD is a parametric CAD package with a scripting language and an \
interactive 3D preview. Run a model file to render it to a 3D-printable mesh. Models are \
written in .ds files; run `dslcad cheatsheet` to learn the language.",
    after_help = EXAMPLE_HELP,
    arg_required_else_help = true
)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// Source path to load, or `-` to read from stdin
    source: Option<String>,

    #[cfg(feature = "preview")]
    #[arg(short, long)]
    /// Display dslcad_viewer window for editing
    preview: bool,

    #[cfg(feature = "preview")]
    #[arg(long)]
    /// Named view to render with `-o png` instead of every view
    view: Option<String>,

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
    /// Log filter (examples: "info", "debug")
    log: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Print the full syntax and function cheat sheet
    Cheatsheet,
}

#[derive(Debug, Clone, Default, ValueEnum)]
enum Output {
    #[default]
    #[value(name = "3mf")]
    ThreeMf,
    Raw,
    Stl,
    Step,
    /// Render every declared view to a png (preview feature only)
    Png,
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
    #[cfg(feature = "preview")]
    Notify(#[from] notify::Error),
    #[error(transparent)]
    Stl(#[from] protocol::StlError),
    #[cfg(not(feature = "preview"))]
    #[error("{0}")]
    Unsupported(String),
    #[cfg(feature = "preview")]
    #[error("unknown view '{0}'")]
    UnknownView(String),
    #[cfg(feature = "preview")]
    #[error("screenshot failed: {0}")]
    Screenshot(String),
}

fn main() {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) => e.exit(),
    };

    if let Some(log) = &args.log {
        env_logger::builder().parse_filters(log).init();
    }

    if let Some(Command::Cheatsheet) = args.command {
        let _ = writeln!(std::io::stdout(), "{}", Library::default());
        return;
    }

    let Some(source) = args.source else {
        eprintln!("error: no source file provided\n\nFor more information, try '--help'.");
        std::process::exit(2);
    };

    #[cfg(feature = "preview")]
    if args.preview {
        if let Err(e) = render_to_preview(&source, args.argument, args.deflection) {
            fail(e);
        }
        return;
    }

    if let Output::Png = &args.output {
        #[cfg(feature = "preview")]
        {
            if let Err(e) = render_to_screenshot(&source, args.argument, args.deflection, args.view)
            {
                fail(e);
            }
            return;
        }
        #[cfg(not(feature = "preview"))]
        fail(CliError::Unsupported(
            "png output requires the preview feature".to_string(),
        ));
    }

    if let Err(e) = render_to_file(&source, args.argument, args.deflection, args.output) {
        fail(e);
    }
}

fn fail(error: CliError) -> ! {
    let _ = handle_error(error, &mut stderr());
    std::process::exit(1);
}

fn handle_error(error: CliError, writer: &mut impl Write) -> Result<(), std::io::Error> {
    let printer = ErrorPrinter::new(FsReader);

    match error {
        CliError::Parse(parse_error) => printer.print_parse_error(writer, &parse_error),
        CliError::Runtime(runtime_error) => printer.print_runtime_error(writer, &runtime_error),
        _ => printer.print_error(writer, &error),
    }
}

fn load_ast(source: &str) -> Result<Ast, CliError> {
    if source == "-" {
        let reader = StdinReader::new()?;
        Ok(parse_with(reader, source.to_string())?)
    } else {
        Ok(parse(source.to_string())?)
    }
}

fn render_to_file(
    source: &String,
    arguments: Vec<String>,
    deflection: f64,
    output: Output,
) -> Result<(), CliError> {
    let arguments = parse_arguments(arguments.iter().map(|i| i.as_str()))?;
    let eval_result = eval(load_ast(source)?, arguments)?;

    let text_output = eval_result.to_text().unwrap_or_default();
    if !text_output.is_empty() {
        println!("{}", text_output);
    }

    let cwd = env::current_dir()?;
    let file = Path::new(source)
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .filter(|stem| stem != "-")
        .unwrap_or_else(|| "stdin".to_string());

    let outfile = match output {
        Output::ThreeMf => {
            let render = render(eval_result, deflection)?;

            let outpath = cwd.join(format!("{}.3mf", file));
            let threemf: ThreeMF = render.into();
            let out = File::create(&outpath)?;
            threemf.write_to_zip(out)?;
            outpath
        }
        Output::Raw => {
            let render = render(eval_result, deflection)?;

            let outpath = cwd.join(format!("{}.bin", file));
            let raw: Vec<u8> = render.try_into()?;
            let mut out = File::create(&outpath)?;
            out.write_all(&raw)?;
            outpath
        }
        Output::Stl => {
            let compressed = eval_result.to_shape()?.into();
            let render = render(compressed, deflection)?;

            let outpath = cwd.join(format!("{}.stl", file));
            let mut out = File::create(&outpath)?;
            render.to_stl(&mut out)?;
            outpath
        }
        Output::Step => {
            let shape = eval_result.to_shape()?;

            let outpath = cwd.join(format!("{}.step", file));
            shape.write_step(&outpath).map_err(RuntimeError::from)?;
            outpath
        }
        // `png` is rendered through the screenshot path in `main`, before this
        // function runs.
        Output::Png => unreachable!("png output is handled before rendering to a file"),
    };

    info!("output written to {}", outfile.to_string_lossy());

    Ok(())
}

#[cfg(feature = "preview")]
fn render_to_screenshot(
    source: &str,
    arguments: Vec<String>,
    deflection: f64,
    view: Option<String>,
) -> Result<(), CliError> {
    use dslcad_storage::protocol::Projection;
    use dslcad_viewer::AxisAngles;

    let arguments = parse_arguments(arguments.iter().map(|i| i.as_str()))?;
    let render = render(eval(load_ast(source)?, arguments)?, deflection)?;

    if !render.stdout.is_empty() {
        print!("{}", render.stdout);
    }

    let stem = Path::new(source)
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "screenshot".to_string());

    // `--view NAME` selects a single view. Without it every declared view is
    // rendered, falling back to the default framing when there are none.
    let selected: Vec<usize> = match &view {
        Some(name) => vec![render
            .views
            .iter()
            .position(|view| view.name.as_deref() == Some(name.as_str()))
            .ok_or_else(|| CliError::UnknownView(name.clone()))?],
        None => (0..render.views.len()).collect(),
    };

    if selected.is_empty() {
        let path = env::current_dir()?.join(format!("{stem}.png"));
        save_screenshot(
            render,
            path,
            AxisAngles::default(),
            None,
            Projection::Perspective,
        )?;
        return Ok(());
    }

    let named = selected.len() > 1;
    for index in selected {
        let selected = render.views[index].clone();

        // A view draws exactly what it includes, so replace the shared
        // annotations rather than adding to them.
        let mut view_render = render.clone();
        view_render.annotations = selected.annotations.clone();
        view_render.views.clear();

        let view_angle = AxisAngles {
            x: selected.angle.x,
            y: selected.angle.y,
            z: selected.angle.z,
        };
        let view_zoom = selected.zoom.map(f64::from);

        let name = if named {
            let name = selected
                .name
                .clone()
                .unwrap_or_else(|| format!("view{}", index + 1));
            format!("{stem}_{name}.png")
        } else {
            format!("{stem}.png")
        };
        let path = env::current_dir()?.join(name);

        save_screenshot(
            view_render,
            path,
            view_angle,
            view_zoom,
            selected.projection,
        )?;
    }

    Ok(())
}

#[cfg(feature = "preview")]
fn save_screenshot(
    render: Render,
    path: std::path::PathBuf,
    angle: dslcad_viewer::AxisAngles,
    zoom: Option<f64>,
    projection: dslcad_storage::protocol::Projection,
) -> Result<(), CliError> {
    use dslcad_viewer::{Preview, ScreenshotOptions};

    let _ = std::fs::remove_file(&path);

    let (preview, handle) = Preview::new();
    handle.show_render(render);
    preview
        .screenshot(ScreenshotOptions {
            path: path.clone(),
            angle,
            zoom: zoom.map(|zoom| zoom as f32),
            projection,
        })
        .map_err(|e| CliError::Screenshot(e.to_string()))?;

    info!("screenshot written to {}", path.to_string_lossy());

    Ok(())
}

#[cfg(feature = "preview")]
fn render_to_preview(
    source: &str,
    arguments: Vec<String>,
    deflection: f64,
) -> Result<(), CliError> {
    use dslcad::parser::DocId;
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
        arguments: &Arc<Mutex<Vec<String>>>,
        deflection: f64,
        watch: Arc<Mutex<Option<RecommendedWatcher>>>,
        cache: Arc<Mutex<Cache>>,
    ) -> Result<Render, CliError> {
        let ast = load_ast(source)?;
        add_files_to_watch(watch, &ast);

        let arguments = arguments.lock().unwrap().clone();
        let arguments = parse_arguments(arguments.iter().map(|i| i.as_str()))?;

        let mut cache = cache.lock().unwrap();
        let value = eval_with_cache(ast, arguments, Some(&mut cache))?;
        let render = render_with_cache(value, deflection, Some(&mut cache))?;

        log::debug!(
            "preview cache: {} hits, {} misses",
            cache.hits(),
            cache.misses()
        );

        Ok(render)
    }

    fn render_to_handle(
        handle: PreviewHandle,
        source: &str,
        arguments: Arc<Mutex<Vec<String>>>,
        deflection: f64,
        watch: Arc<Mutex<Option<RecommendedWatcher>>>,
        cache: Arc<Mutex<Cache>>,
    ) {
        handle.show_rendering();
        match render_with_watcher(source, &arguments, deflection, watch, cache) {
            Ok(render) => handle.show_render(render),
            Err(err) => {
                let mut buffer = Vec::new();
                handle_error(err, &mut buffer).unwrap();
                handle.show_error(String::from_utf8(buffer).unwrap());
            }
        }
    }

    let (preview, handle) = Preview::new();
    *handle.arguments().lock().unwrap() = arguments;
    let watch = Arc::new(Mutex::new(None));
    let cache = Arc::new(Mutex::new(Cache::new()));

    let watcher = {
        let (source, watch, handle, cache) = (
            source.to_string(),
            watch.clone(),
            handle.clone(),
            cache.clone(),
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
                    handle.arguments(),
                    deflection,
                    watch.clone(),
                    cache.clone(),
                );
            }
        })?
    };

    {
        let mut g = watch.lock().unwrap();
        g.replace(watcher);
    }

    let source = source.to_string();

    {
        let (handle, watch, cache, source) =
            (handle.clone(), watch.clone(), cache.clone(), source.clone());
        let arguments = handle.arguments();
        std::thread::spawn(move || {
            render_to_handle(handle, &source, arguments, deflection, watch, cache)
        });
    }

    // Parameter edits from the editor re-render with the updated arguments,
    // without waiting for the file to change.
    {
        let (handle, watch, cache, source) =
            (handle.clone(), watch.clone(), cache.clone(), source.clone());
        let arguments = handle.arguments();
        let rerender = handle.rerender();
        std::thread::spawn(move || loop {
            if rerender.lock().unwrap().recv().is_err() {
                break;
            }
            render_to_handle(
                handle.clone(),
                &source,
                arguments.clone(),
                deflection,
                watch.clone(),
                cache.clone(),
            );
        });
    }

    preview.open(Library::default().to_string());
    Ok(())
}
