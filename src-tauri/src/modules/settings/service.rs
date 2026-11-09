use super::model::{Settings, SettingsPatch};
use crate::core::error::AppResult;
use crate::database::{repositories::settings_repo, Database};
use tokio::sync::RwLock;

/// Current settings in memory, persisted on every change.
pub struct SettingsService {
    db: Database,
    current: RwLock<Settings>,
}

impl SettingsService {
    pub async fn load(db: Database) -> AppResult<Self> {
        let stored = settings_repo::load_all(&db.pool).await?;
        Ok(Self { db, current: RwLock::new(Settings::from_stored(&stored)) })
    }

    pub async fn get(&self) -> Settings {
        self.current.read().await.clone()
    }

    /// Validate, persist (one transaction), then publish. A rejected patch changes nothing.
    pub async fn update(&self, patch: &SettingsPatch) -> AppResult<Settings> {
        let mut cur = self.current.write().await;
        let next = cur.patched(patch);
        next.validate()?;
        settings_repo::save(&self.db.pool, &next.changed_pairs(patch)?).await?;
        *cur = next.clone();
        Ok(next)
    }

    pub async fn reset(&self) -> AppResult<Settings> {
        let mut cur = self.current.write().await;
        settings_repo::clear(&self.db.pool).await?;
        *cur = Settings::default();
        Ok(cur.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::settings::Theme;

    #[tokio::test]
    async fn updates_persist_and_invalid_patches_change_nothing() {
        let db = Database::open_in_memory().await.unwrap();
        let svc = SettingsService::load(db.clone()).await.unwrap();
        svc.update(&SettingsPatch { theme: Some(Theme::Light), sidebar_width: Some(300), ..Default::default() }).await.unwrap();
        let err = svc.update(&SettingsPatch { theme: Some(Theme::Glass), accent: Some("blue".into()), ..Default::default() }).await;
        assert!(err.is_err());
        assert_eq!(svc.get().await.theme, Theme::Light, "rejected patch must not apply partly");
        // A fresh service reads what was stored.
        let again = SettingsService::load(db.clone()).await.unwrap().get().await;
        assert_eq!((again.theme, again.sidebar_width), (Theme::Light, 300));
        assert_eq!(svc.reset().await.unwrap(), Settings::default());
        assert_eq!(SettingsService::load(db).await.unwrap().get().await, Settings::default());
    }
}
