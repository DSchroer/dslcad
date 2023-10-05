mod editor;
mod settings;

use crate::settings::Settings;
use persistence::protocol::Render;
use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender};

enum PreviewEvent {
    Render(Render),
    Error(String),
}

#[derive(Clone)]
pub struct PreviewHandle {
    tx: Sender<PreviewEvent>,
}

impl PreviewHandle {
    pub fn show_render(&self, render: Render) -> Result<(), Box<dyn Error>> {
        Ok(self.tx.send(PreviewEvent::Render(render))?)
    }

    pub fn show_error(&self, error: String) -> Result<(), Box<dyn Error>> {
        Ok(self.tx.send(PreviewEvent::Error(error))?)
    }
}

pub struct Preview {
    rx: Receiver<PreviewEvent>,
}

impl Preview {
    pub fn new() -> (Self, PreviewHandle) {
        let (tx, rx) = channel();
        (Self { rx }, PreviewHandle { tx })
    }

    pub fn open(self, cheetsheet: String) {
        editor::main(cheetsheet, self.rx, Settings::default()).unwrap();
    }
}
