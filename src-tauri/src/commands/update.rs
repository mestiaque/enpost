//! In-app updates from GitHub releases, the same mechanism as ENViewer and ENDownloader.
//!
//! * `check_update`   – asks the GitHub API for the latest release (only when the user allows
//!                      it: Settings → General);
//! * `install_update` – downloads the installer for this kind of install, checks the SHA-256
//!                      GitHub publishes for it, and installs it in place:
//!                      .deb → `pkexec apt-get install` (system password prompt),
//!                      AppImage → the file is replaced,
//!                      Windows → the NSIS installer runs silently after ENPost quits;
//! * `restart_app`    – starts the new version once this process has exited.

use crate::core::error::{AppError, AppResult};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;
use ts_rs::TS;

const API_LATEST: &str = "https://api.github.com/repos/mestiaque/enpost/releases/latest";
const ASSET_PREFIX: &str = "ENPost_";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
    pub page: String,
    pub asset_name: String,
    pub url: String,
    #[ts(type = "number")]
    pub size: u64,
    pub sha256: String,
    /// This copy can update itself (.deb, AppImage or the Windows installer).
    pub installable: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Kind {
    Deb,
    AppImage(PathBuf),
    Windows,
    Unmanaged,
}

fn parse_version(s: &str) -> (u64, u64, u64) {
    let core = s.trim_start_matches(|c: char| !c.is_ascii_digit()).split('-').next().unwrap_or("");
    let mut it = core.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0))
}

fn install_kind() -> Kind {
    let exe = std::env::current_exe().unwrap_or_default();
    if cfg!(windows) {
        // The NSIS installer puts uninstall.exe next to the program.
        return match exe.parent() {
            Some(dir) if dir.join("uninstall.exe").exists() => Kind::Windows,
            _ => Kind::Unmanaged,
        };
    }
    if let Some(p) = std::env::var_os("APPIMAGE") {
        return Kind::AppImage(PathBuf::from(p));
    }
    if exe.starts_with("/usr/bin") && Path::new("/usr/bin/apt-get").exists() {
        return Kind::Deb;
    }
    Kind::Unmanaged
}

fn pick_asset<'a>(assets: &'a [serde_json::Value], kind: &Kind) -> Option<&'a serde_json::Value> {
    assets.iter().find(|a| {
        let n = a["name"].as_str().unwrap_or("");
        n.starts_with(ASSET_PREFIX)
            && match kind {
                Kind::Deb => n.ends_with("_amd64.deb"),
                Kind::AppImage(_) => n.ends_with("_amd64.AppImage"),
                Kind::Windows => n.ends_with("-setup.exe"),
                Kind::Unmanaged => false,
            }
    })
}

fn update_from_release(rel: &serde_json::Value, current: &str, kind: &Kind) -> Option<UpdateInfo> {
    let tag = rel["tag_name"].as_str().unwrap_or("");
    if tag.is_empty() || rel["draft"].as_bool() == Some(true) || rel["prerelease"].as_bool() == Some(true) {
        return None;
    }
    let v = parse_version(tag);
    if v <= parse_version(current) {
        return None;
    }
    let mut info = UpdateInfo {
        version: format!("{}.{}.{}", v.0, v.1, v.2),
        notes: rel["body"].as_str().unwrap_or("").trim().to_string(),
        page: rel["html_url"].as_str().unwrap_or_default().to_string(),
        asset_name: String::new(),
        url: String::new(),
        size: 0,
        sha256: String::new(),
        installable: false,
    };
    let empty = vec![];
    if let Some(a) = pick_asset(rel["assets"].as_array().unwrap_or(&empty), kind) {
        info.asset_name = a["name"].as_str().unwrap_or("").to_string();
        info.url = a["browser_download_url"].as_str().unwrap_or("").to_string();
        info.size = a["size"].as_u64().unwrap_or(0);
        info.sha256 = a["digest"].as_str().and_then(|d| d.strip_prefix("sha256:")).unwrap_or("").to_lowercase();
        // Without a published checksum nothing is installed.
        info.installable = !info.url.is_empty() && info.sha256.len() == 64;
    }
    Some(info)
}

fn client(app: &AppHandle, timeout: u64) -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(format!("ENPost/{} (update check)", app.package_info().version))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(timeout))
        .build()
        .map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> AppResult<Option<UpdateInfo>> {
    // The Snap Store updates a snap.
    if std::env::var_os("SNAP").is_some() {
        return Ok(None);
    }
    let current = app.package_info().version.to_string();
    let resp = client(&app, 20)?
        .get(API_LATEST)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| AppError::Network(format!("could not reach GitHub: {}", e.without_url())))?;
    // No release published yet.
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !resp.status().is_success() {
        return Err(AppError::Network(format!("GitHub answered {}", resp.status())));
    }
    let rel: serde_json::Value = resp.json().await.map_err(|e| AppError::Network(format!("unexpected answer from GitHub: {e}")))?;
    Ok(update_from_release(&rel, &current, &install_kind()))
}

#[derive(Serialize, Clone)]
struct Progress {
    done: u64,
    total: u64,
}

async fn download(app: &AppHandle, info: &UpdateInfo, dir: &Path) -> AppResult<PathBuf> {
    tokio::fs::create_dir_all(dir).await?;
    let name = Path::new(&info.asset_name).file_name().ok_or_else(|| AppError::Internal("bad asset name".into()))?;
    let out = dir.join(name);
    let part = out.with_extension("part");
    let resp = client(app, 1800)?.get(&info.url).send().await.map_err(|e| AppError::Network(format!("download failed: {}", e.without_url())))?;
    if !resp.status().is_success() {
        return Err(AppError::Network(format!("download failed: HTTP {}", resp.status())));
    }
    let total = resp.content_length().unwrap_or(info.size);
    let mut file = tokio::fs::File::create(&part).await?;
    let mut hasher = Sha256::new();
    let (mut done, mut last) = (0u64, 0u64);
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                let _ = tokio::fs::remove_file(&part).await;
                return Err(AppError::Network(format!("download failed: {}", e.without_url())));
            }
        };
        file.write_all(&chunk).await?;
        hasher.update(&chunk);
        done += chunk.len() as u64;
        if done - last > 512 * 1024 || done == total {
            last = done;
            let _ = app.emit("update-progress", Progress { done, total });
        }
    }
    file.flush().await?;
    drop(file);
    if format!("{:x}", hasher.finalize()) != info.sha256 {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(AppError::Validation("The downloaded file is damaged (checksum mismatch). Please try again.".into()));
    }
    tokio::fs::rename(&part, &out).await?;
    Ok(out)
}

/// Returns "restart" (installed; call restart_app) or "exiting" (the Windows installer takes over).
#[tauri::command]
pub async fn install_update(app: AppHandle, info: UpdateInfo) -> AppResult<String> {
    let kind = install_kind();
    if !info.installable || kind == Kind::Unmanaged {
        return Err(AppError::Validation("This copy cannot update itself (it is a development build).".into()));
    }
    let dir = app.path().app_cache_dir().map_err(|e| AppError::Internal(e.to_string()))?.join("updates");
    let file = download(&app, &info, &dir).await?;
    match kind {
        Kind::Deb => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644));
            }
            let out = tokio::process::Command::new("pkexec")
                .args(["apt-get", "install", "-y", "--allow-downgrades"])
                .arg(&file)
                .output()
                .await
                .map_err(|e| AppError::Internal(format!("could not start the installer (pkexec): {e}")))?;
            match out.status.code() {
                Some(0) => Ok("restart".into()),
                Some(126) | Some(127) => Err(AppError::Validation("Update cancelled: the administrator password was not entered.".into())),
                _ => {
                    let err = String::from_utf8_lossy(&out.stderr);
                    let tail: Vec<&str> = err.lines().rev().take(3).collect();
                    Err(AppError::Internal(format!("installing the update failed: {}", tail.into_iter().rev().collect::<Vec<_>>().join(" "))))
                }
            }
        }
        Kind::AppImage(target) => {
            let tmp = target.with_extension("AppImage.new");
            tokio::fs::copy(&file, &tmp).await.map_err(|e| AppError::Io(format!("cannot write next to {}: {e}", target.display())))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
            }
            tokio::fs::rename(&tmp, &target).await?;
            let _ = tokio::fs::remove_file(&file).await;
            Ok("restart".into())
        }
        Kind::Windows => {
            let exe = std::env::current_exe()?;
            let script = format!("timeout /t 2 /nobreak >nul & \"{}\" /S & start \"\" \"{}\"", file.display(), exe.display());
            let mut cmd = std::process::Command::new("cmd");
            cmd.args(["/C", &script]);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x0000_0008 | 0x0000_0200 | 0x0800_0000); // DETACHED | NEW_GROUP | NO_WINDOW
            }
            cmd.spawn().map_err(|e| AppError::Internal(format!("could not start the installer: {e}")))?;
            let app2 = app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(500));
                app2.exit(0);
            });
            Ok("exiting".into())
        }
        Kind::Unmanaged => unreachable!(),
    }
}

/// Start the freshly installed version after this process exits (a direct re-exec would be
/// caught by the single-instance guard while this process still runs).
#[tauri::command]
pub fn restart_app(app: AppHandle) -> AppResult<()> {
    let target = match install_kind() {
        Kind::AppImage(p) => p,
        _ => std::env::current_exe()?,
    };
    #[cfg(unix)]
    {
        std::process::Command::new("sh")
            .args(["-c", "sleep 1.5; exec \"$0\"", &target.to_string_lossy()])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
    }
    #[cfg(windows)]
    {
        let _ = &target;
    }
    app.exit(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn release(tag: &str) -> serde_json::Value {
        let sha = "a".repeat(64);
        json!({ "tag_name": tag, "body": " Fixes ", "html_url": "https://example.invalid/r", "assets": [
            { "name": "ENPost_9.0.0_x64-setup.exe", "browser_download_url": "https://x/w.exe", "size": 3, "digest": format!("sha256:{sha}") },
            { "name": "ENPost_9.0.0_amd64.AppImage", "browser_download_url": "https://x/a.AppImage", "size": 4, "digest": format!("sha256:{sha}") },
            { "name": "ENPost_9.0.0_amd64.deb", "browser_download_url": "https://x/a.deb", "size": 5, "digest": format!("sha256:{}", sha.to_uppercase()) },
            { "name": "Other_9.0.0_amd64.deb", "browser_download_url": "https://x/o.deb", "size": 5 }
        ]})
    }

    #[test]
    fn versions_compare_numerically() {
        assert_eq!(parse_version("v0.10.2"), (0, 10, 2));
        assert_eq!(parse_version("1.2"), (1, 2, 0));
        assert!(parse_version("v0.10.0") > parse_version("0.9.9"));
    }

    #[test]
    fn picks_the_asset_for_this_install() {
        let deb = update_from_release(&release("v9.0.0"), "0.1.0", &Kind::Deb).unwrap();
        assert_eq!((deb.asset_name.as_str(), deb.notes.as_str(), deb.installable), ("ENPost_9.0.0_amd64.deb", "Fixes", true));
        assert_eq!(deb.sha256, "a".repeat(64));
        assert_eq!(update_from_release(&release("v9.0.0"), "0.1.0", &Kind::AppImage("/x".into())).unwrap().asset_name, "ENPost_9.0.0_amd64.AppImage");
        assert_eq!(update_from_release(&release("v9.0.0"), "0.1.0", &Kind::Windows).unwrap().asset_name, "ENPost_9.0.0_x64-setup.exe");
        assert!(!update_from_release(&release("v9.0.0"), "0.1.0", &Kind::Unmanaged).unwrap().installable);
        assert!(update_from_release(&release("v0.1.0"), "0.1.0", &Kind::Deb).is_none(), "same version is no update");
    }

    #[test]
    fn no_checksum_means_not_installable() {
        let mut rel = release("v9.0.0");
        rel["assets"][2]["digest"] = serde_json::Value::Null;
        assert!(!update_from_release(&rel, "0.1.0", &Kind::Deb).unwrap().installable);
    }
}
