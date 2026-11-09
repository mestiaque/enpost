//! Tauri IPC layer: thin functions that check input, call a service and map errors.
//! No business logic and no SQL here.
pub mod app;
pub mod collections;
pub mod history;
pub mod http;
pub mod settings;
pub mod update;
