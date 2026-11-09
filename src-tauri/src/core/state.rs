//! Shared state handed to commands: explicit services, built once at start-up.

use crate::database::Database;
use crate::modules::collections::CollectionsService;
use crate::modules::http::HttpEngine;
use crate::modules::settings::SettingsService;
use std::path::PathBuf;

pub struct AppState {
    pub db: Database,
    pub settings: SettingsService,
    pub collections: CollectionsService,
    pub http: HttpEngine,
    pub data_dir: PathBuf,
    /// Response bodies (`<data>/bodies`); nothing outside it is ever read back.
    pub bodies_dir: PathBuf,
}
