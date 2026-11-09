//! Application settings: defaults, validation and the patch the UI sends.
//! The TypeScript side uses the generated `Settings` / `SettingsPatch` bindings.

use crate::core::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Theme {
    Dark,
    Light,
    Glass,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct Settings {
    // Appearance
    pub theme: Theme,
    /// `#rrggbb`
    pub accent: String,
    pub reduce_motion: bool,
    // Editor
    /// 10–24 px
    pub editor_font_size: u32,
    /// CSS font-family list; empty = the built-in monospace stack.
    pub editor_font_family: String,
    pub editor_word_wrap: bool,
    // Layout (remembered between runs)
    pub sidebar_collapsed: bool,
    /// 200–520 px
    pub sidebar_width: u32,
    /// Share of the workspace height given to the request editor, 0.2–0.8.
    pub editor_split: f64,
    // Requests (used from Phase 2)
    /// 0 = no timeout; otherwise 1 000–600 000 ms.
    pub request_timeout_ms: u32,
    pub follow_redirects: bool,
    /// 0–50
    pub max_redirects: u32,
    /// Turning this off needs an explicit confirmation in the UI.
    pub verify_tls: bool,
    // Data
    pub autosave: bool,
    /// 0 = keep forever; otherwise 1–3650 days.
    pub history_retention_days: u32,
    pub check_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            accent: "#3b82f6".into(),
            reduce_motion: false,
            editor_font_size: 13,
            editor_font_family: String::new(),
            editor_word_wrap: false,
            sidebar_collapsed: false,
            sidebar_width: 280,
            editor_split: 0.5,
            request_timeout_ms: 30_000,
            follow_redirects: true,
            max_redirects: 10,
            verify_tls: true,
            autosave: true,
            history_retention_days: 30,
            check_updates: true,
        }
    }
}

/// Changed fields only; every field is optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, optional_fields)]
pub struct SettingsPatch {
    pub theme: Option<Theme>,
    pub accent: Option<String>,
    pub reduce_motion: Option<bool>,
    pub editor_font_size: Option<u32>,
    pub editor_font_family: Option<String>,
    pub editor_word_wrap: Option<bool>,
    pub sidebar_collapsed: Option<bool>,
    pub sidebar_width: Option<u32>,
    pub editor_split: Option<f64>,
    pub request_timeout_ms: Option<u32>,
    pub follow_redirects: Option<bool>,
    pub max_redirects: Option<u32>,
    pub verify_tls: Option<bool>,
    pub autosave: Option<bool>,
    pub history_retention_days: Option<u32>,
    pub check_updates: Option<bool>,
}

fn is_hex_colour(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

impl Settings {
    /// Build from stored key → JSON pairs. Each key is applied on its own, so one bad or
    /// outdated value only falls back to its default.
    pub fn from_stored(stored: &HashMap<String, Value>) -> Self {
        let mut obj = match serde_json::to_value(Settings::default()) {
            Ok(Value::Object(m)) => m,
            _ => return Settings::default(),
        };
        for (k, v) in stored {
            if let Some(slot) = obj.get_mut(k) {
                let previous = std::mem::replace(slot, v.clone());
                if serde_json::from_value::<Settings>(Value::Object(obj.clone())).map(|s| s.validate().is_err()).unwrap_or(true) {
                    *obj.get_mut(k).expect("key exists") = previous;
                }
            }
        }
        serde_json::from_value(Value::Object(obj)).unwrap_or_default()
    }

    /// Reject values the UI should never send (the UI clamps, the backend still checks).
    pub fn validate(&self) -> AppResult<()> {
        let bad = |m: &str| Err(AppError::Validation(m.into()));
        if !is_hex_colour(&self.accent) {
            return bad("Accent must be a colour like #3b82f6");
        }
        if !(10..=24).contains(&self.editor_font_size) {
            return bad("Editor font size must be between 10 and 24");
        }
        if self.editor_font_family.len() > 200 || self.editor_font_family.contains([';', '{', '}']) {
            return bad("Editor font family is not a valid font list");
        }
        if !(200..=520).contains(&self.sidebar_width) {
            return bad("Sidebar width must be between 200 and 520");
        }
        if !(0.2..=0.8).contains(&self.editor_split) {
            return bad("Editor split must be between 0.2 and 0.8");
        }
        if self.request_timeout_ms != 0 && !(1_000..=600_000).contains(&self.request_timeout_ms) {
            return bad("Timeout must be 0 (none) or between 1 and 600 seconds");
        }
        if self.max_redirects > 50 {
            return bad("At most 50 redirects");
        }
        if self.history_retention_days > 3650 {
            return bad("History can be kept for at most 3650 days");
        }
        Ok(())
    }

    /// The result of applying a patch (not yet validated).
    pub fn patched(&self, p: &SettingsPatch) -> Settings {
        let mut s = self.clone();
        macro_rules! set {
            ($($f:ident),*) => { $( if let Some(v) = &p.$f { s.$f = v.clone(); } )* };
        }
        set!(
            theme, accent, reduce_motion, editor_font_size, editor_font_family, editor_word_wrap, sidebar_collapsed, sidebar_width, editor_split,
            request_timeout_ms, follow_redirects, max_redirects, verify_tls, autosave, history_retention_days, check_updates
        );
        s
    }

    /// Key → JSON pairs for the fields a patch touches (what gets written).
    pub fn changed_pairs(&self, p: &SettingsPatch) -> AppResult<Vec<(String, Value)>> {
        let patch = serde_json::to_value(p)?;
        let full = serde_json::to_value(self)?;
        let mut out = Vec::new();
        if let (Value::Object(pm), Value::Object(fm)) = (patch, full) {
            for (k, v) in pm {
                if !v.is_null() {
                    if let Some(val) = fm.get(&k) {
                        out.push((k, val.clone()));
                    }
                }
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn defaults_are_valid() {
        Settings::default().validate().unwrap();
    }

    #[test]
    fn bad_stored_values_fall_back_per_key() {
        let stored: HashMap<String, Value> = [
            ("theme".to_string(), json!("light")),
            ("accent".to_string(), json!("red")),          // invalid → default
            ("editorFontSize".to_string(), json!(99)),     // out of range → default
            ("sidebarWidth".to_string(), json!(320)),
            ("unknownKey".to_string(), json!(true)),       // ignored
        ]
        .into_iter()
        .collect();
        let s = Settings::from_stored(&stored);
        assert_eq!(s.theme, Theme::Light);
        assert_eq!(s.accent, "#3b82f6");
        assert_eq!(s.editor_font_size, 13);
        assert_eq!(s.sidebar_width, 320);
    }

    #[test]
    fn patch_and_validation() {
        let s = Settings::default();
        let p = SettingsPatch { theme: Some(Theme::Glass), editor_font_size: Some(15), ..Default::default() };
        let n = s.patched(&p);
        assert_eq!((n.theme, n.editor_font_size, n.accent.as_str()), (Theme::Glass, 15, "#3b82f6"));
        let pairs = n.changed_pairs(&p).unwrap();
        assert_eq!(pairs.len(), 2);
        assert!(s.patched(&SettingsPatch { accent: Some("#12345".into()), ..Default::default() }).validate().is_err());
        assert!(s.patched(&SettingsPatch { request_timeout_ms: Some(500), ..Default::default() }).validate().is_err());
        assert!(s.patched(&SettingsPatch { request_timeout_ms: Some(0), ..Default::default() }).validate().is_ok());
        assert!(s.patched(&SettingsPatch { editor_font_family: Some("x;}body{".into()), ..Default::default() }).validate().is_err());
    }
}
