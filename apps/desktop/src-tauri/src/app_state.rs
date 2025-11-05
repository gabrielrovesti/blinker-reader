use std::path::PathBuf;

pub struct AppState {
    pub db_path: PathBuf,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Self { Self { db_path } }
}

