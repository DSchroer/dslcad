use clap::{Parser, Subcommand, ValueEnum};
use dslcad::error_printer::ErrorPrinter;
use dslcad::library::Library;
use dslcad::parser::{Ast, DocumentParseError, ParseError};
use dslcad::reader::{FsReader, StdinReader};
use dslcad::runtime::{RuntimeError, WithStack};
use dslcad::{eval, parse, parse_arguments, parse_with, render};
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
  dslcad ./part.ds -a size=5        render with the `size` script argument set to 5
  dslcad ./part.ds -s x90y45 2      render a screenshot from an angle at 2x zoom
  dslcad cheatsheet                 print the full syntax and function reference

Run `dslcad cheatsheet` for the language reference, and see
https://github.com/DSchroer/dslcad/tree/master/examples for example models.";

#[cfg(not(feature = "preview"))]
const EXAMPLE_HELP: &str = "\
Examples:
  dslcad ./part.ds                  render part.ds to part.3mf
  dslcad ./part.ds -o stl           render to an STL instead of a 3MF
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
    #[arg(
        short,
        long,
        num_args = 0..=2,
        value_names = ["ANGLE", "ZOOM"],
        allow_negative_numbers = true,
        conflicts_with = "preview"
    )]
    /// Render a single view of the part to a png file. Angle is a sequence of
    /// axis rotations like `x90y45`, where x tilts from the top (x0 is a top
    /// view), y rotates around the vertical axis and z rolls the camera. A bare
    /// number is shorthand for a y rotation. Zoom is a magnification factor
    /// (defaults to 1)
    screenshot: Option<Vec<String>>,

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
    #[cfg(feature = "preview")]
    #[error("invalid screenshot argument: {0}")]
    InvalidScreenshot(String),
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
    if let Some(values) = &args.screenshot {
        let result = parse_screenshot_arguments(values);
        match result {
            Ok((angle, zoom)) => {
                if let Err(e) =
                    render_to_screenshot(&source, args.argument, args.deflection, angle, zoom)
                {
                    fail(e);
                }
            }
            Err(e) => fail(e),
        }
        return;
    }

    #[cfg(feature = "preview")]
    if args.preview {
        if let Err(e) = render_to_preview(&source, args.argument, args.deflection) {
            fail(e);
        }
        return;
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
    };

    info!("output written to {}", outfile.to_string_lossy());

    Ok(())
}

#[cfg(feature = "preview")]
fn parse_screenshot_arguments(
    values: &[String],
) -> Result<(dslcad_viewer::AxisAngles, Option<f64>), CliError> {
    let angle = match values.first() {
        Some(angle) => angle
            .parse::<dslcad_viewer::AxisAngles>()
            .map_err(CliError::InvalidScreenshot)?,
        None => dslcad_viewer::AxisAngles::default(),
    };

    let zoom = match values.get(1) {
        Some(zoom) => {
            let zoom: f64 = zoom
                .parse()
                .map_err(|_| CliError::InvalidScreenshot(format!("invalid zoom '{}'", zoom)))?;
            if zoom <= 0.0 {
                return Err(CliError::InvalidScreenshot(
                    "zoom must be greater than zero".to_string(),
                ));
            }
            Some(zoom)
        }
        None => None,
    };

    Ok((angle, zoom))
}

#[cfg(feature = "preview")]
fn render_to_screenshot(
    source: &str,
    arguments: Vec<String>,
    deflection: f64,
    angle: dslcad_viewer::AxisAngles,
    zoom: Option<f64>,
) -> Result<(), CliError> {
    use dslcad_viewer::{Preview, ScreenshotOptions};

    let arguments = parse_arguments(arguments.iter().map(|i| i.as_str()))?;
    let render = render(eval(load_ast(source)?, arguments)?, deflection)?;

    if !render.stdout.is_empty() {
        print!("{}", render.stdout);
    }

    let stem = Path::new(source)
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "screenshot".to_string());
    let path = env::current_dir()?.join(format!("{}.png", stem));
    let _ = std::fs::remove_file(&path);

    let (preview, handle) = Preview::new();
    handle.show_render(render);
    preview
        .screenshot(ScreenshotOptions {
            path: path.clone(),
            angle,
            zoom: zoom.map(|zoom| zoom as f32),
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
        arguments: &[String],
        deflection: f64,
        watch: Arc<Mutex<Option<RecommendedWatcher>>>,
    ) -> Result<Render, CliError> {
        let ast = load_ast(source)?;
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
