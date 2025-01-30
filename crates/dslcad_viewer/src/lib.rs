mod editor;
mod settings;

use crate::settings::Settings;
use dslcad_storage::protocol::Render;
use std::sync::mpsc::{channel, Receiver, Sender};

enum PreviewEvent {
    Rendering,
    Render(Render),
    Error(String),
}

#[derive(Clone)]
pub struct PreviewHandle {
    tx: Sender<PreviewEvent>,
}

impl PreviewHandle {
    pub fn show_rendering(&self) {
        self.tx.send(PreviewEvent::Rendering).unwrap()
    }

    pub fn show_render(&self, render: Render) {
        self.tx.send(PreviewEvent::Render(render)).unwrap()
    }

    pub fn show_error(&self, error: String) {
        self.tx.send(PreviewEvent::Error(error)).unwrap()
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
