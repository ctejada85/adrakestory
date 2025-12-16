//! Dialog events and resources.

use bevy::prelude::*;
use std::path::PathBuf;
use std::sync::{
    mpsc::Receiver,
    Arc, Mutex,
};

/// Event sent when a file is selected from the file dialog
#[derive(Event)]
pub struct FileSelectedEvent {
    pub path: PathBuf,
}

/// Event sent when map data changes (needs to be public for map_editor.rs)
#[derive(Event)]
pub struct MapDataChangedEvent;

/// Event sent when the app should exit
#[derive(Event)]
pub struct AppExitEvent;

/// Resource to track the file dialog receiver
#[derive(Resource, Default)]
pub struct FileDialogReceiver {
    pub receiver: Option<Arc<Mutex<Receiver<Option<PathBuf>>>>>,
}
