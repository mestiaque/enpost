//! Application services, one module per feature area. Commands call these; these call
//! repositories. Later phases add http, collections, environments, scripting, …
pub mod history;
pub mod http;
pub mod settings;
