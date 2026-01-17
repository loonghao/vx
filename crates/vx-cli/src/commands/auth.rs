//! Authentication commands for vx
//!
//! Provides authentication support for various services like GitHub.
//! Supports GitHub Device Flow OAuth for easy token acquisition.

use anyhow::{Context, Result};
use std::time::Duration;

/// GitHub OAuth Device Flow authentication
///
/// This follows the GitHub Device Flow specification:
/// https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#device-flow
pub struct GitHubDeviceFlow {
    client_id: String,
}

/// Device code response from GitHub
#[derive(Debug, serde::Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Access token response from GitHub
#[derive(Debug, serde::Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

impl GitHubDeviceFlow {
    /// Create a new GitHub Device Flow instance
    ///
    /// Uses vx's OAuth App client ID for public access
    pub fn new() -> Self {
        // vx's public OAuth App client ID (read-only scope for releases)
        // This is safe to embed - it only grants read access and requires user approval
        Self {
            client_id: "Ov23liYVuLhTqq4VLfRy".to_string(),
        }
    }

    /// Create with a custom client ID (for organizations)
    pub fn with_client_id(client_id: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
        }
    }

    /// Start the device flow and get the user code
    pub async fn start(&self) -> Result<DeviceCodeResponse> {
        let client = reqwest::Client::new();

        let response = client
            .post("https://github.com/login/device/code")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", &self.client_id),
                ("scope", &"public_repo".to_string()), // Read-only access to public repos
            ])
            .send()
            .await
            .context("Failed to initiate GitHub device flow")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error ({}): {}", status, body);
        }

        response
            .json::<DeviceCodeResponse>()
            .await
            .context("Failed to parse device code response")
    }

    /// Poll for access token until user approves or times out
    pub async fn poll_for_token(
        &self,
        device_code: &str,
        interval: u64,
        expires_in: u64,
    ) -> Result<String> {
        let client = reqwest::Client::new();
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(expires_in);
        let poll_interval = Duration::from_secs(interval.max(5)); // Minimum 5 seconds as per GitHub docs

        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("Authorization timed out. Please try again.");
            }

            tokio::time::sleep(poll_interval).await;

            let response = client
                .post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .form(&[
                    ("client_id", &self.client_id),
                    ("device_code", &device_code.to_string()),
                    (
                        "grant_type",
                        &"urn:ietf:params:oauth:grant-type:device_code".to_string(),
                    ),
                ])
                .send()
                .await
                .context("Failed to poll for access token")?;

            let token_response: AccessTokenResponse = response
                .json()
                .await
                .context("Failed to parse token response")?;

            if let Some(token) = token_response.access_token {
                return Ok(token);
            }

            if let Some(error) = token_response.error {
                match error.as_str() {
                    "authorization_pending" => {
                        // User hasn't authorized yet, continue polling
                        continue;
                    }
                    "slow_down" => {
                        // We're polling too fast, add 5 seconds
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                    "expired_token" => {
                        anyhow::bail!("Authorization expired. Please try again.");
                    }
                    "access_denied" => {
                        anyhow::bail!("Authorization was denied by the user.");
                    }
                    _ => {
                        let description = token_response
                            .error_description
                            .unwrap_or_else(|| error.clone());
                        anyhow::bail!("GitHub OAuth error: {}", description);
                    }
                }
            }
        }
    }
}

impl Default for GitHubDeviceFlow {
    fn default() -> Self {
        Self::new()
    }
}

/// Store a GitHub token securely
pub fn store_github_token(token: &str) -> Result<()> {
    let paths = vx_paths::VxPaths::new()?;
    let config_dir = paths.config_dir;
    std::fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

    let token_file = config_dir.join("github_token");

    // Write token with restricted permissions
    std::fs::write(&token_file, token).context("Failed to write token file")?;

    // Set file permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&token_file)?.permissions();
        perms.set_mode(0o600); // Owner read/write only
        std::fs::set_permissions(&token_file, perms)?;
    }

    Ok(())
}

/// Load a stored GitHub token
pub fn load_github_token() -> Option<String> {
    // First check environment variables (highest priority)
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }
    if let Ok(token) = std::env::var("GH_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }

    // Then check stored token file
    let token_file = vx_paths::VxPaths::new()
        .ok()?
        .config_dir
        .join("github_token");
    if token_file.exists() {
        if let Ok(token) = std::fs::read_to_string(&token_file) {
            let token = token.trim();
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }

    None
}

/// Remove stored GitHub token
pub fn remove_github_token() -> Result<()> {
    let paths = vx_paths::VxPaths::new()?;
    let token_file = paths.config_dir.join("github_token");
    if token_file.exists() {
        std::fs::remove_file(&token_file).context("Failed to remove token file")?;
    }
    Ok(())
}

/// Check GitHub token status
pub struct TokenStatus {
    pub source: TokenSource,
    pub scopes: Vec<String>,
    pub rate_limit: Option<RateLimitInfo>,
}

#[derive(Debug, Clone)]
pub enum TokenSource {
    /// Token from GITHUB_TOKEN environment variable
    EnvGitHubToken,
    /// Token from GH_TOKEN environment variable
    EnvGhToken,
    /// Token from vx config file
    ConfigFile,
    /// No token configured
    None,
}

impl std::fmt::Display for TokenSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenSource::EnvGitHubToken => write!(f, "GITHUB_TOKEN environment variable"),
            TokenSource::EnvGhToken => write!(f, "GH_TOKEN environment variable"),
            TokenSource::ConfigFile => write!(f, "vx config file (~/.vx/config/github_token)"),
            TokenSource::None => write!(f, "not configured"),
        }
    }
}

#[derive(Debug)]
pub struct RateLimitInfo {
    pub limit: u64,
    pub remaining: u64,
    pub reset: u64,
}

/// Get current token status
pub async fn get_token_status() -> Result<TokenStatus> {
    // Determine token source
    let source = if std::env::var("GITHUB_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
        .is_some()
    {
        TokenSource::EnvGitHubToken
    } else if std::env::var("GH_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
        .is_some()
    {
        TokenSource::EnvGhToken
    } else {
        let token_file = vx_paths::VxPaths::new()
            .ok()
            .map(|p| p.config_dir.join("github_token"));
        if token_file.as_ref().map(|f| f.exists()).unwrap_or(false)
            && token_file
                .as_ref()
                .and_then(|f| std::fs::read_to_string(f).ok())
                .filter(|t| !t.trim().is_empty())
                .is_some()
        {
            TokenSource::ConfigFile
        } else {
            TokenSource::None
        }
    };

    // If no token, return early
    if matches!(source, TokenSource::None) {
        return Ok(TokenStatus {
            source,
            scopes: vec![],
            rate_limit: None,
        });
    }

    // Get token and check with GitHub API
    let token = load_github_token().unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get("https://api.github.com/rate_limit")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vx-cli")
        .send()
        .await;

    let (scopes, rate_limit) = match response {
        Ok(resp) => {
            // Extract scopes from header
            let scopes = resp
                .headers()
                .get("x-oauth-scopes")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(", ").map(|s| s.to_string()).collect())
                .unwrap_or_default();

            // Parse rate limit info
            let rate_limit = if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                body.get("resources")
                    .and_then(|r| r.get("core"))
                    .map(|core| RateLimitInfo {
                        limit: core.get("limit").and_then(|v| v.as_u64()).unwrap_or(0),
                        remaining: core.get("remaining").and_then(|v| v.as_u64()).unwrap_or(0),
                        reset: core.get("reset").and_then(|v| v.as_u64()).unwrap_or(0),
                    })
            } else {
                None
            };

            (scopes, rate_limit)
        }
        Err(_) => (vec![], None),
    };

    Ok(TokenStatus {
        source,
        scopes,
        rate_limit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_flow_creation() {
        let flow = GitHubDeviceFlow::new();
        assert!(!flow.client_id.is_empty());
    }

    #[test]
    fn test_token_source_display() {
        assert_eq!(
            format!("{}", TokenSource::EnvGitHubToken),
            "GITHUB_TOKEN environment variable"
        );
        assert_eq!(
            format!("{}", TokenSource::EnvGhToken),
            "GH_TOKEN environment variable"
        );
    }
}
