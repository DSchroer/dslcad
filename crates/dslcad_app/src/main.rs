#![windows_subsystem = "windows"]

extern crate core;

#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "editor")]
mod editor;

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    if env::args().len() > 1 {
        #[cfg(feature = "cli")]
        cli::main()
    } else {
        #[cfg(feature = "editor")]
        editor::main()
    }
}
