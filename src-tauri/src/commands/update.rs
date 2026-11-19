//! In-app updates from GitHub releases, the same mechanism as ENViewer and ENSave.
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
/// The latest release, as published to the public mestiaque/en-releases repository.
const MANIFEST: &str = "https://raw.githubusercontent.com/mestiaque/en-releases/main/enpost.json";
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
        notes: clean_notes(rel["body"].as_str().unwrap_or("")),
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
    let client = client(&app, 20)?;
    // The public en-releases list first (a plain file: no API rate limit, and the repository may be
    // private), the repository's release API as a fallback.
    let listed = match client.get(MANIFEST).send().await {
        Ok(r) if r.status().is_success() => Some(r),
        _ => None,
    };
    let resp = match listed {
        Some(r) => r,
        None => client
            .get(API_LATEST)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| AppError::Network(format!("could not reach GitHub: {}", e.without_url())))?,
    };
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
            let dir = exe.parent().ok_or_else(|| AppError::Internal("the program folder is unknown".into()))?.to_path_buf();
            let script = windows_update_script(std::process::id(), &nsis_step(&file, &dir), &exe, &update_log());
            spawn_update_script(&script).map_err(|e| AppError::Internal(format!("could not start the installer: {e}")))?;
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

// ---------------------------------------------------------------- release notes

/// Release notes as plain text for the update dialog: the lines GitHub adds to generated notes
/// ("Full Changelog: https://github.com/.../compare/...", pull-request credits) and Markdown
/// marks are left out.
fn clean_notes(body: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for raw in body.replace('\r', "").lines() {
        let low = raw.to_lowercase();
        let heading = low.trim_start_matches(|c: char| c == '#' || c.is_whitespace()).trim();
        if low.contains("full changelog")
            || low.contains("made their first contribution")
            || heading == "new contributors"
            || (low.contains("github.com/") && (low.contains("/compare/") || low.contains("/commits/")))
        {
            continue;
        }
        let mut line = raw.to_string();
        // "* Fix by @someone in https://github.com/.../pull/3" → "* Fix"
        if let Some(i) = line.find(" by @") {
            if line[i..].contains(" in https://github.com/") {
                line.truncate(i);
            }
        }
        let line = strip_links(&line).replace("**", "").replace("__", "").replace('`', "");
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        let mut rest = line[indent.len()..].to_string();
        if rest.starts_with('#') {
            rest = rest.trim_start_matches('#').trim_start().to_string();
        } else if rest.starts_with("* ") || rest.starts_with("- ") {
            rest = format!("\u{2022} {}", rest[2..].trim_start());
        }
        lines.push(format!("{indent}{rest}").trim_end().to_string());
    }
    let mut text = lines.join("\n");
    while text.contains("\n\n\n") {
        text = text.replace("\n\n\n", "\n\n");
    }
    text.trim().to_string()
}

/// `[text](link)` → `text`.
fn strip_links(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(open) = rest.find('[') {
        if let Some(mid) = rest[open..].find("](").map(|i| open + i) {
            if let Some(end) = rest[mid + 2..].find(')').map(|i| mid + 2 + i) {
                out.push_str(&rest[..open]);
                out.push_str(&rest[open + 1..mid]);
                rest = &rest[end + 1..];
                continue;
            }
        }
        out.push_str(&rest[..=open]);
        rest = &rest[open + 1..];
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------- Windows: finish the update
//
// After this process quits, a hidden PowerShell script waits for it to exit, installs the update
// and starts the new version; every step is logged to %TEMP%\ENPost-update.log. The script travels
// as -EncodedCommand: a `cmd /C "..."` line does not work here, because Windows programs escape
// the quotes around paths as \" (which cmd.exe does not understand) and because cmd's `timeout`
// fails without a console.

/// A PowerShell single-quoted string literal.
#[cfg_attr(not(windows), allow(dead_code))]
fn ps_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

#[cfg_attr(not(windows), allow(dead_code))]
fn base64(bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = ((chunk[0] as u32) << 16) | ((*chunk.get(1).unwrap_or(&0) as u32) << 8) | (*chunk.get(2).unwrap_or(&0) as u32);
        for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            out.push(if i <= chunk.len() { ABC[((n >> shift) & 63) as usize] as char } else { '=' });
        }
    }
    out
}

/// PowerShell that waits for this process (`pid`) to quit, runs `step`, then starts `exe`.
#[cfg_attr(not(windows), allow(dead_code))]
fn windows_update_script(pid: u32, step: &str, exe: &Path, log: &Path) -> String {
    [
        "$ErrorActionPreference = 'Continue'".to_string(),
        format!("$log = {}", ps_quote(&log.to_string_lossy())),
        "function Log($m) { Add-Content -LiteralPath $log -Value ((Get-Date -Format s) + '  ' + $m) -Encoding UTF8 }".to_string(),
        format!("Log 'Update: waiting for ENPost (process {pid}) to quit'"),
        format!("Wait-Process -Id {pid} -Timeout 60 -ErrorAction SilentlyContinue"),
        format!("try {{ {step} }} catch {{ Log ('The update was not installed: ' + $_) }}"),
        format!("try {{ Start-Process -FilePath {} }} catch {{ Log ('Could not start ENPost: ' + $_) }}", ps_quote(&exe.to_string_lossy())),
    ]
    .join("\n")
}

/// Runs the NSIS installer silently into `dir` (`/D=` must come last and stay unquoted).
#[cfg_attr(not(windows), allow(dead_code))]
fn nsis_step(installer: &Path, dir: &Path) -> String {
    format!(
        "$p = Start-Process -FilePath {} -Wait -PassThru -ArgumentList '/S', {}; Log ('Installer finished with exit code ' + $p.ExitCode)",
        ps_quote(&installer.to_string_lossy()),
        ps_quote(&format!("/D={}", dir.display()))
    )
}

/// Arguments for `powershell.exe` that run `script` hidden (nothing in them needs quoting).
#[cfg_attr(not(windows), allow(dead_code))]
fn powershell_args(script: &str) -> Vec<String> {
    let utf16: Vec<u8> = script.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
    ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-WindowStyle", "Hidden", "-EncodedCommand"]
        .iter()
        .map(|s| s.to_string())
        .chain([base64(&utf16)])
        .collect()
}

/// Starts the update script; this process should quit right after.
#[cfg_attr(not(windows), allow(dead_code))]
fn spawn_update_script(script: &str) -> std::io::Result<()> {
    let root = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let mut cmd = std::process::Command::new(root.join(r"System32\WindowsPowerShell\v1.0\powershell.exe"));
    cmd.args(powershell_args(script))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000 | 0x0000_0200); // CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP
    }
    cmd.spawn().map(|_| ())
}

#[cfg_attr(not(windows), allow(dead_code))]
fn update_log() -> PathBuf {
    std::env::temp_dir().join("ENPost-update.log")
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
        let deb = update_from_release(&release("v9.0.0"), "0.1.1", &Kind::Deb).unwrap();
        assert_eq!((deb.asset_name.as_str(), deb.notes.as_str(), deb.installable), ("ENPost_9.0.0_amd64.deb", "Fixes", true));
        assert_eq!(deb.sha256, "a".repeat(64));
        assert_eq!(update_from_release(&release("v9.0.0"), "0.1.1", &Kind::AppImage("/x".into())).unwrap().asset_name, "ENPost_9.0.0_amd64.AppImage");
        assert_eq!(update_from_release(&release("v9.0.0"), "0.1.1", &Kind::Windows).unwrap().asset_name, "ENPost_9.0.0_x64-setup.exe");
        assert!(!update_from_release(&release("v9.0.0"), "0.1.1", &Kind::Unmanaged).unwrap().installable);
        assert!(update_from_release(&release("v0.1.1"), "0.1.1", &Kind::Deb).is_none(), "same version is no update");
    }

    #[test]
    fn no_checksum_means_not_installable() {
        let mut rel = release("v9.0.0");
        rel["assets"][2]["digest"] = serde_json::Value::Null;
        assert!(!update_from_release(&rel, "0.1.1", &Kind::Deb).unwrap().installable);
    }

    #[test]
    fn release_notes_leave_out_github_links() {
        assert_eq!(clean_notes("**Full Changelog**: https://github.com/mestiaque/x/compare/v0.2.9...v0.2.10"), "");
        let notes = clean_notes(
            "## What's Changed\r\n* Faster scans by @me in https://github.com/a/b/pull/3\r\n* **Bold** fix, see [docs](https://x)\r\n\r\n\
             ## New Contributors\r\n* @x made their first contribution in https://github.com/a/b/pull/1\r\n\r\n\
             **Full Changelog**: https://github.com/a/b/compare/v1...v2",
        );
        assert_eq!(notes, "What's Changed\n\u{2022} Faster scans\n\u{2022} Bold fix, see docs");
    }

    #[test]
    fn windows_update_script_waits_quotes_and_encodes() {
        assert_eq!(base64(b"Hi"), "SGk=");
        assert_eq!(base64(b"abc"), "YWJj");
        assert_eq!(base64(b"ab"), "YWI=");
        let step = nsis_step(Path::new(r"C:\Users\O'Neil\Cache\ENPost_9.0.0_x64-setup.exe"), Path::new(r"C:\Users\O'Neil\AppData\Local\ENPost"));
        assert!(step.contains(r"-FilePath 'C:\Users\O''Neil\Cache\ENPost_9.0.0_x64-setup.exe'"));
        assert!(step.contains(r"-ArgumentList '/S', '/D=C:\Users\O''Neil\AppData\Local\ENPost'"));
        let script = windows_update_script(4242, &step, Path::new(r"C:\Program Files\ENPost\app.exe"), Path::new(r"C:\Temp\ENPost-update.log"));
        assert!(script.contains("Wait-Process -Id 4242"));
        assert!(script.contains(r"Start-Process -FilePath 'C:\Program Files\ENPost\app.exe'"));
        let args = powershell_args("A");
        assert_eq!(args.last().map(String::as_str), Some("QQA=")); // "A" in UTF-16LE
        assert!(args.iter().all(|a| !a.contains('"')));
    }
}
