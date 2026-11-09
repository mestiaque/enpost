//! Shared state handed to commands: explicit services, built once at start-up.

use crate::database::Database;
use crate::modules::settings::SettingsService;
use std::path::PathBuf;

pub struct AppState {
    pub db: Database,
    pub settings: SettingsService,
    pub data_dir: PathBuf,
}
