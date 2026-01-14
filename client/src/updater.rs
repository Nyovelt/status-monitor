use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process;
use tracing::{info, warn};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub struct Updater {
    client: Client,
    github_repo: Option<String>,
}

impl Updater {
    pub fn new(github_repo: Option<String>) -> Self {
        Self {
            client: Client::new(),
            github_repo,
        }
    }

    /// Check for updates and apply if available
    pub async fn check_and_update(&self) -> anyhow::Result<bool> {
        let repo = match &self.github_repo {
            Some(r) => r,
            None => {
                info!("Auto-update disabled (GITHUB_REPO not set)");
                return Ok(false);
            }
        };

        info!("Checking for updates... (current version: {})", CURRENT_VERSION);

        // Fetch latest release
        let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
        let response = self
            .client
            .get(&url)
            .header("User-Agent", "status-monitor-client")
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to fetch release info: {}", response.status());
            return Ok(false);
        }

        let release: GithubRelease = response.json().await?;
        let latest_version = release.tag_name.trim_start_matches('v');

        if !is_newer_version(latest_version, CURRENT_VERSION) {
            info!("Already running latest version");
            return Ok(false);
        }

        info!("New version available: {} -> {}", CURRENT_VERSION, latest_version);

        // Find the appropriate binary asset
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let asset_name = format!("status-monitor-client-{}-{}", os, arch);

        let asset = release
            .assets
            .iter()
            .find(|a| a.name.contains(&asset_name) || a.name == "status-monitor-client")
            .or_else(|| release.assets.first());

        let asset = match asset {
            Some(a) => a,
            None => {
                warn!("No suitable binary found in release");
                return Ok(false);
            }
        };

        info!("Downloading update from: {}", asset.browser_download_url);

        // Download new binary
        let binary_data = self
            .client
            .get(&asset.browser_download_url)
            .header("User-Agent", "status-monitor-client")
            .send()
            .await?
            .bytes()
            .await?;

        // Write to temp file
        let temp_path = "/tmp/status-monitor-client-new";
        fs::write(temp_path, &binary_data)?;

        // Make executable
        let mut perms = fs::metadata(temp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(temp_path, perms)?;

        // Get current executable path
        let current_exe = env::current_exe()?;
        let install_path = current_exe.to_string_lossy();

        info!("Installing update to: {}", install_path);

        // Replace current binary
        fs::rename(temp_path, &*install_path)?;

        info!("Update installed successfully. Restarting...");

        // Exit to let systemd restart us
        process::exit(0);
    }
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let latest_parts = parse_version(latest);
    let current_parts = parse_version(current);

    for i in 0..latest_parts.len().max(current_parts.len()) {
        let latest_part = latest_parts.get(i).copied().unwrap_or(0);
        let current_part = current_parts.get(i).copied().unwrap_or(0);

        if latest_part > current_part {
            return true;
        } else if latest_part < current_part {
            return false;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("1.0.1", "1.0.0"));
        assert!(is_newer_version("1.1.0", "1.0.0"));
        assert!(is_newer_version("2.0.0", "1.9.9"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0.1"));
        assert!(!is_newer_version("0.9.0", "1.0.0"));
    }
}
