use anyhow::Result;
use reqwest;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::process::Command;
use which::which;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,
}

impl Version {
    pub fn parse(version_str: &str) -> Result<Self> {
        let version_str = version_str.trim_start_matches('v');
        let parts: Vec<&str> = version_str.split('.').collect();

        if parts.len() < 3 {
            return Err(anyhow::anyhow!("Invalid version format: {}", version_str));
        }

        let major = parts[0].parse()?;
        let minor = parts[1].parse()?;

        // Handle patch version with pre-release
        let patch_part = parts[2];
        let (patch, pre) = if let Some(dash_pos) = patch_part.find('-') {
            let patch = patch_part[..dash_pos].parse()?;
            let pre = Some(patch_part[dash_pos + 1..].to_string());
            (patch, pre)
        } else {
            (patch_part.parse()?, None)
        };

        Ok(Self {
            major,
            minor,
            patch,
            pre,
        })
    }

    pub fn as_string(&self) -> String {
        match &self.pre {
            Some(pre) => format!("{}.{}.{}-{}", self.major, self.minor, self.patch, pre),
            None => format!("{}.{}.{}", self.major, self.minor, self.patch),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

pub struct VersionManager;

impl VersionManager {
    /// Check if a tool is installed and get its version
    pub fn get_installed_version(tool_name: &str) -> Result<Option<Version>> {
        // Check if tool is available in PATH
        if which(tool_name).is_err() {
            return Ok(None);
        }

        // Try to get version
        let output = Command::new(tool_name).arg("--version").output()?;

        if !output.status.success() {
            return Ok(None);
        }

        let version_output = String::from_utf8_lossy(&output.stdout);
        let version_line = version_output.lines().next().unwrap_or("");

        // Extract version number from output
        let version_str = Self::extract_version_from_output(version_line)?;
        let version = Version::parse(&version_str)?;

        Ok(Some(version))
    }

    /// Get latest stable version from GitHub releases (for tools that support it)
    pub async fn get_latest_version(tool_name: &str) -> Result<Version> {
        match tool_name {
            "uv" => Self::get_uv_latest_version().await,
            "node" => Self::get_node_latest_version().await,
            _ => Err(anyhow::anyhow!(
                "Unsupported tool for version checking: {}",
                tool_name
            )),
        }
    }

    async fn get_uv_latest_version() -> Result<Version> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.github.com/repos/astral-sh/uv/releases/latest")
            .header("User-Agent", "vx-tool")
            .send()
            .await?;

        let release: serde_json::Value = response.json().await?;
        let tag_name = release["tag_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Could not find tag_name in release"))?;

        Version::parse(tag_name)
    }

    async fn get_node_latest_version() -> Result<Version> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://nodejs.org/dist/index.json")
            .header("User-Agent", "vx-tool")
            .send()
            .await?;

        let releases: serde_json::Value = response.json().await?;
        let latest = releases
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| anyhow::anyhow!("Could not find latest Node.js version"))?;

        let version_str = latest["version"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Could not find version in Node.js release"))?;

        Version::parse(version_str)
    }

    fn extract_version_from_output(output: &str) -> Result<String> {
        // Common patterns for version extraction
        let patterns = [
            r"(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?)",  // Standard semver
            r"v(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?)", // With 'v' prefix
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(output) {
                    if let Some(version) = captures.get(1) {
                        return Ok(version.as_str().to_string());
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Could not extract version from output: {}",
            output
        ))
    }
}
