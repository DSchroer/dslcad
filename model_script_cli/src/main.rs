#![windows_subsystem = "windows"]

mod cli;
mod editor;

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    if env::args().len() > 1 {
        cli::main()
    } else {
        editor::main()
    }
}
