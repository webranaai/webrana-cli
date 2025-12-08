// ============================================
// WEBRANA CLI - Auto Update Checker
// Sprint 5.6: v1.0 Preparation
// Created by: ATLAS (Team Beta)
// ============================================

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const GITHUB_API_URL: &str = "https://api.github.com/repos/webranaai/webrana-cli/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Release information from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub html_url: String,
    pub published_at: String,
    pub body: Option<String>,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Update check result
#[derive(Debug)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable {
        current: String,
        latest: String,
        url: String,
        release_notes: Option<String>,
    },
    CheckFailed(String),
}

/// Check for updates from GitHub releases
pub async fn check_for_updates() -> UpdateStatus {
    match fetch_latest_release().await {
        Ok(release) => {
            let latest = release.tag_name.trim_start_matches('v');
            let current = CURRENT_VERSION;

            if is_newer_version(latest, current) {
                UpdateStatus::UpdateAvailable {
                    current: current.to_string(),
                    latest: latest.to_string(),
                    url: release.html_url,
                    release_notes: release.body,
                }
            } else {
                UpdateStatus::UpToDate
            }
        }
        Err(e) => UpdateStatus::CheckFailed(e.to_string()),
    }
}

/// Fetch latest release from GitHub API
async fn fetch_latest_release() -> Result<ReleaseInfo> {
    let client = reqwest::Client::new();

    let response = client
        .get(GITHUB_API_URL)
        .header("User-Agent", format!("webrana-cli/{}", CURRENT_VERSION))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context("Failed to connect to GitHub API")?;

    if !response.status().is_success() {
        anyhow::bail!("GitHub API returned status: {}", response.status());
    }

    let release: ReleaseInfo = response
        .json()
        .await
        .context("Failed to parse release info")?;

    Ok(release)
}

/// Compare version strings (semver-like)
fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v
            .split('.')
            .filter_map(|p| p.split('-').next())
            .filter_map(|p| p.parse().ok())
            .collect();

        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };

    let (l_major, l_minor, l_patch) = parse_version(latest);
    let (c_major, c_minor, c_patch) = parse_version(current);

    (l_major, l_minor, l_patch) > (c_major, c_minor, c_patch)
}

/// Get download URL for current platform
pub fn get_platform_download_url(release: &ReleaseInfo) -> Option<&ReleaseAsset> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let platform_patterns = match (os, arch) {
        ("linux", "x86_64") => vec!["linux-x86_64", "linux-amd64", "linux64"],
        ("linux", "aarch64") => vec!["linux-aarch64", "linux-arm64"],
        ("macos", "x86_64") => vec!["darwin-x86_64", "macos-x86_64", "macos-amd64"],
        ("macos", "aarch64") => vec!["darwin-aarch64", "macos-arm64", "darwin-arm64"],
        ("windows", "x86_64") => vec!["windows-x86_64", "windows-amd64", "win64", ".exe"],
        _ => vec![],
    };

    for asset in &release.assets {
        let name_lower = asset.name.to_lowercase();
        for pattern in &platform_patterns {
            if name_lower.contains(pattern) {
                return Some(asset);
            }
        }
    }

    None
}

/// Format update message for display
pub fn format_update_message(status: &UpdateStatus) -> String {
    match status {
        UpdateStatus::UpToDate => {
            format!("Webrana CLI v{} is up to date.", CURRENT_VERSION)
        }
        UpdateStatus::UpdateAvailable {
            current,
            latest,
            url,
            release_notes,
        } => {
            let mut msg = format!(
                "Update available: v{} -> v{}\nDownload: {}\n",
                current, latest, url
            );

            if let Some(notes) = release_notes {
                let preview: String = notes.chars().take(200).collect();
                msg.push_str(&format!("\nRelease notes:\n{}", preview));
                if notes.len() > 200 {
                    msg.push_str("...");
                }
            }

            msg
        }
        UpdateStatus::CheckFailed(error) => {
            format!("Update check failed: {}", error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("1.0.0", "0.4.0"));
        assert!(is_newer_version("0.5.0", "0.4.0"));
        assert!(is_newer_version("0.4.1", "0.4.0"));
        assert!(!is_newer_version("0.4.0", "0.4.0"));
        assert!(!is_newer_version("0.3.0", "0.4.0"));
    }

    #[test]
    fn test_version_parsing() {
        assert!(is_newer_version("1.0.0", "0.4.0-alpha"));
        assert!(is_newer_version("0.5.0-beta", "0.4.0"));
    }
}
