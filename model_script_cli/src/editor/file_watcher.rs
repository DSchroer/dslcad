use crate::editor::{State, UiEvent};
use bevy::prelude::ResMut;
use bevy::prelude::*;
use notify::event::{AccessKind, AccessMode};
use notify::{Error, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct FileWatcherPlugin;

impl Plugin for FileWatcherPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_watcher)
            .add_system(notify_watch);
    }
}

pub struct FileWatcher {
    status: Arc<AtomicBool>,
    files: Vec<PathBuf>,
    watcher: RecommendedWatcher,
}

impl FileWatcher {
    pub fn new() -> Self {
        let status = Arc::new(AtomicBool::new(false));

        let watch_status = status.clone();
        let watcher = notify::recommended_watcher(move |res: Result<Event, Error>| match res {
            Ok(e) => {
                if let EventKind::Access(AccessKind::Close(AccessMode::Write)) = e.kind {
                    watch_status.swap(true, Ordering::Relaxed);
                }
            }
            Err(e) => {
                println!("watch error: {:?}", e)
            }
        })
        .unwrap();

        FileWatcher {
            status,
            files: Vec::new(),
            watcher,
        }
    }

    pub fn add(&mut self, path: PathBuf) -> Result<(), Error> {
        self.files.push(path);
        self.watcher.watch(
            &self.files[self.files.len() - 1],
            RecursiveMode::NonRecursive,
        )
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        for path in &self.files {
            self.watcher.unwatch(path)?;
        }
        self.files.clear();
        Ok(())
    }

    pub fn dirty(&mut self) -> bool {
        self.status.swap(false, Ordering::Relaxed)
    }
}

fn setup_watcher(mut state: ResMut<State>) {
    state.watcher = Some(FileWatcher::new())
}

fn notify_watch(mut state: ResMut<State>, mut events: EventWriter<UiEvent>) {
    if state.autowatch && state.watcher.as_mut().unwrap().dirty() {
        events.send(UiEvent::Render())
    }
}
