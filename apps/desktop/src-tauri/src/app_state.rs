use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use blinker_core_render::AnyRenderer;

pub struct ReaderSession {
    pub renderer: AnyRenderer,
    pub item_id: String,
}

pub struct AppState {
    pub db_path: PathBuf,
    pub sessions: Arc<Mutex<HashMap<String, ReaderSession>>>,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

