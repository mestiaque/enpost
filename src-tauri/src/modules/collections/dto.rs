//! Workspaces, collections, folders and saved requests as the frontend sees them.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CollectionNode {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FolderNode {
    pub id: String,
    pub collection_id: String,
    pub parent_id: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RequestSummary {
    pub id: String,
    pub collection_id: String,
    pub folder_id: Option<String>,
    pub name: String,
    pub method: String,
    pub url: String,
    pub favorite: bool,
}

/// Everything the sidebar needs, in display order (each list sorted by its parent's order).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WorkspaceTree {
    pub workspace_id: String,
    pub collections: Vec<CollectionNode>,
    pub folders: Vec<FolderNode>,
    pub requests: Vec<RequestSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct KeyValueRow {
    pub id: String,
    pub key: String,
    pub value: String,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FormRowDef {
    pub id: String,
    pub key: String,
    pub value: String,
    pub enabled: bool,
    /// "text" | "file"
    pub kind: String,
}

/// The editable parts of a request besides method and URL (stored as JSON).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct RequestDefinition {
    pub params: Vec<KeyValueRow>,
    pub headers: Vec<KeyValueRow>,
    /// "none" | "json" | "text" | "xml" | "form-data" | "urlencoded" | "binary"
    pub body_type: String,
    pub body: String,
    pub urlencoded: Vec<KeyValueRow>,
    pub form: Vec<FormRowDef>,
    pub binary_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SavedRequest {
    pub id: String,
    pub collection_id: String,
    pub folder_id: Option<String>,
    pub name: String,
    pub description: String,
    pub method: String,
    pub url: String,
    pub definition: RequestDefinition,
    pub favorite: bool,
    #[ts(type = "number")]
    pub updated_at: i64,
}

/// Fields of a request to write; `None` keeps the stored value.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, optional_fields)]
pub struct RequestPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub method: Option<String>,
    pub url: Option<String>,
    pub definition: Option<RequestDefinition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ItemKind {
    Collection,
    Folder,
    Request,
}

/// Where a dragged item goes: a collection (and optionally a folder in it) and the position
/// among siblings of the same kind (folders and requests are ordered separately).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MoveTarget {
    pub collection_id: Option<String>,
    pub folder_id: Option<String>,
    pub index: u32,
}
