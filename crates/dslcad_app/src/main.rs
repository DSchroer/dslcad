#![windows_subsystem = "windows"]

extern crate core;

mod cli;
mod dslcad;
mod editor;
mod reader;

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    if env::args().len() > 1 {
        cli::main()
    } else {
        editor::main()
    }
}
