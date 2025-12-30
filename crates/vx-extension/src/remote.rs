//! Remote extension installation and management
//!
//! This module handles installing extensions from remote sources like GitHub.

use crate::error::{ExtensionError, ExtensionResult};
use crate::ExtensionConfig;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};
use vx_paths::VxPaths;

/// Remote extension source specification
#[derive(Debug, Clone)]
pub enum RemoteSource {
    /// GitHub repository (github:user/repo or github:user/repo@version)
    GitHub {
        owner: String,
        repo: String,
        version: Option<String>,
        /// Subdirectory path within the repository (e.g., "examples/extensions/hello-world")
        subdir: Option<String>,
    },
    /// Direct Git URL
    GitUrl {
        url: String,
        version: Option<String>,
    },
}

impl RemoteSource {
    /// Parse a source string into a RemoteSource
    ///
    /// Supported formats:
    /// - `github:user/repo`
    /// - `github:user/repo@v1.0.0`
    /// - `https://github.com/user/repo`
    /// - `https://github.com/user/repo@v1.0.0`
    /// - `https://github.com/user/repo/tree/branch/path/to/extension`
    /// - `git@github.com:user/repo.git`
    pub fn parse(source: &str) -> ExtensionResult<Self> {
        // GitHub shorthand: github:user/repo[@version]
        if let Some(rest) = source.strip_prefix("github:") {
            return Self::parse_github_shorthand(rest);
        }

        // GitHub HTTPS URL
        if source.starts_with("https://github.com/") {
            return Self::parse_github_url(source);
        }

        // GitHub SSH URL
        if source.starts_with("git@github.com:") {
            return Self::parse_github_ssh(source);
        }

        // Generic git URL
        if source.starts_with("https://") || source.starts_with("git://") {
            return Self::parse_git_url(source);
        }

        Err(ExtensionError::RemoteInstallFailed {
            src: source.to_string(),
            reason: "Unsupported source format. Supported formats:\n\
                 - github:user/repo[@version]\n\
                 - https://github.com/user/repo[@version]\n\
                 - https://github.com/user/repo/tree/branch/path/to/extension\n\
                 - git@github.com:user/repo.git[@version]"
                .to_string(),
        })
    }

    fn parse_github_shorthand(rest: &str) -> ExtensionResult<Self> {
        let (repo_part, version) = Self::split_version(rest);
        let parts: Vec<&str> = repo_part.split('/').collect();

        if parts.len() < 2 {
            return Err(ExtensionError::RemoteInstallFailed {
                src: format!("github:{}", rest),
                reason: "Invalid GitHub shorthand. Expected format: github:user/repo".to_string(),
            });
        }

        // Support github:user/repo/path/to/extension format
        let (owner, repo, subdir) = if parts.len() > 2 {
            (
                parts[0].to_string(),
                parts[1].to_string(),
                Some(parts[2..].join("/")),
            )
        } else {
            (parts[0].to_string(), parts[1].to_string(), None)
        };

        Ok(Self::GitHub {
            owner,
            repo,
            version,
            subdir,
        })
    }

    fn parse_github_url(url: &str) -> ExtensionResult<Self> {
        let (url_part, version) = Self::split_version(url);
        let path = url_part
            .strip_prefix("https://github.com/")
            .unwrap_or(url_part);
        let path = path.trim_end_matches(".git");
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() < 2 {
            return Err(ExtensionError::RemoteInstallFailed {
                src: url.to_string(),
                reason: "Invalid GitHub URL. Expected format: https://github.com/user/repo"
                    .to_string(),
            });
        }

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();

        // Check for tree/branch/path format: https://github.com/user/repo/tree/branch/path
        let (version, subdir) = if parts.len() > 2 && parts[2] == "tree" {
            if parts.len() > 4 {
                // Has branch and path: tree/branch/path/to/ext
                let branch = parts[3].to_string();
                let subpath = parts[4..].join("/");
                (Some(branch), Some(subpath))
            } else if parts.len() == 4 {
                // Only branch: tree/branch
                (Some(parts[3].to_string()), None)
            } else {
                (version, None)
            }
        } else if parts.len() > 2 {
            // Direct path without tree: user/repo/path/to/ext
            (version, Some(parts[2..].join("/")))
        } else {
            (version, None)
        };

        Ok(Self::GitHub {
            owner,
            repo,
            version,
            subdir,
        })
    }

    fn parse_github_ssh(url: &str) -> ExtensionResult<Self> {
        // For SSH URLs, the @ is part of the URL format, so we need special handling
        // Format: git@github.com:user/repo.git[@version]
        let url_without_prefix = url.strip_prefix("git@github.com:").unwrap_or(url);

        // Now split version from the path part
        let (path_part, version) = Self::split_version(url_without_prefix);
        let path = path_part.trim_end_matches(".git");
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() < 2 {
            return Err(ExtensionError::RemoteInstallFailed {
                src: url.to_string(),
                reason: "Invalid GitHub SSH URL. Expected format: git@github.com:user/repo.git"
                    .to_string(),
            });
        }

        Ok(Self::GitHub {
            owner: parts[0].to_string(),
            repo: parts[1].to_string(),
            version,
            subdir: None,
        })
    }

    fn parse_git_url(url: &str) -> ExtensionResult<Self> {
        let (url_part, version) = Self::split_version(url);
        Ok(Self::GitUrl {
            url: url_part.to_string(),
            version,
        })
    }

    fn split_version(s: &str) -> (&str, Option<String>) {
        if let Some(idx) = s.rfind('@') {
            let (base, ver) = s.split_at(idx);
            (base, Some(ver[1..].to_string()))
        } else {
            (s, None)
        }
    }

    /// Get the clone URL for this remote source
    pub fn clone_url(&self) -> String {
        match self {
            Self::GitHub { owner, repo, .. } => {
                format!("https://github.com/{}/{}.git", owner, repo)
            }
            Self::GitUrl { url, .. } => url.clone(),
        }
    }

    /// Get the version/tag to checkout
    pub fn version(&self) -> Option<&str> {
        match self {
            Self::GitHub { version, .. } => version.as_deref(),
            Self::GitUrl { version, .. } => version.as_deref(),
        }
    }

    /// Get the subdirectory path within the repository
    pub fn subdir(&self) -> Option<&str> {
        match self {
            Self::GitHub { subdir, .. } => subdir.as_deref(),
            Self::GitUrl { .. } => None,
        }
    }

    /// Get a display name for this source
    pub fn display_name(&self) -> String {
        match self {
            Self::GitHub {
                owner,
                repo,
                subdir,
                ..
            } => {
                if let Some(path) = subdir {
                    format!("{}/{}/{}", owner, repo, path)
                } else {
                    format!("{}/{}", owner, repo)
                }
            }
            Self::GitUrl { url, .. } => url.clone(),
        }
    }
}

/// Remote extension installer
pub struct RemoteInstaller {
    /// Cache directory for cloned repositories
    cache_dir: PathBuf,
    /// User extensions directory
    user_dir: PathBuf,
}

impl RemoteInstaller {
    /// Create a new remote installer
    pub fn new() -> ExtensionResult<Self> {
        let vx_paths = VxPaths::new().map_err(|e| {
            ExtensionError::io(
                format!("Failed to initialize vx paths: {}", e),
                None,
                std::io::Error::other(e.to_string()),
            )
        })?;

        Ok(Self {
            cache_dir: vx_paths.base_dir.join("extensions-cache"),
            user_dir: vx_paths.base_dir.join("extensions"),
        })
    }

    /// Install an extension from a remote source
    pub async fn install(&self, source: &str) -> ExtensionResult<InstalledExtension> {
        let remote = RemoteSource::parse(source)?;
        info!("Installing extension from {}", remote.display_name());

        // Ensure directories exist
        std::fs::create_dir_all(&self.cache_dir).map_err(|e| {
            ExtensionError::io(
                "Failed to create cache directory",
                Some(self.cache_dir.clone()),
                e,
            )
        })?;
        std::fs::create_dir_all(&self.user_dir).map_err(|e| {
            ExtensionError::io(
                "Failed to create extensions directory",
                Some(self.user_dir.clone()),
                e,
            )
        })?;

        // Clone or update the repository
        let cache_path = self.get_cache_path(&remote);
        self.clone_or_update(&remote, &cache_path)?;

        // Determine the extension source path (may be a subdirectory)
        let ext_source_path = if let Some(subdir) = remote.subdir() {
            cache_path.join(subdir)
        } else {
            cache_path
        };

        // Load and validate the extension config
        let config = self.load_and_validate_config(&ext_source_path, source)?;
        let ext_name = config.extension.name.clone();

        // Copy to user extensions directory
        let target_path = self.user_dir.join(&ext_name);
        self.copy_extension(&ext_source_path, &target_path, &ext_name)?;

        // Save source metadata for updates
        let metadata_path = target_path.join(".vx-source");
        std::fs::write(&metadata_path, source).map_err(|e| {
            ExtensionError::io("Failed to write source metadata", Some(metadata_path), e)
        })?;

        info!("Successfully installed extension '{}'", ext_name);

        Ok(InstalledExtension {
            name: ext_name,
            version: config.extension.version,
            path: target_path,
            source: source.to_string(),
        })
    }

    /// Update an installed extension
    pub async fn update(&self, name: &str) -> ExtensionResult<InstalledExtension> {
        let target_path = self.user_dir.join(name);

        if !target_path.exists() {
            return Err(ExtensionError::ExtensionNotFound {
                name: name.to_string(),
                available: self.list_installed()?,
                searched_paths: vec![self.user_dir.clone()],
            });
        }

        // Try to find the original source from metadata
        let metadata_path = target_path.join(".vx-source");
        let source = if metadata_path.exists() {
            std::fs::read_to_string(&metadata_path).map_err(|e| {
                ExtensionError::io(
                    "Failed to read extension source metadata",
                    Some(metadata_path.clone()),
                    e,
                )
            })?
        } else {
            return Err(ExtensionError::UpdateFailed {
                name: name.to_string(),
                reason:
                    "No source metadata found. This extension may have been installed manually."
                        .to_string(),
            });
        };

        // Reinstall from source
        self.install(&source).await
    }

    /// Check for updates for an extension
    pub async fn check_update(&self, name: &str) -> ExtensionResult<Option<UpdateInfo>> {
        let target_path = self.user_dir.join(name);

        if !target_path.exists() {
            return Err(ExtensionError::ExtensionNotFound {
                name: name.to_string(),
                available: self.list_installed()?,
                searched_paths: vec![self.user_dir.clone()],
            });
        }

        // Load current version
        let config_path = target_path.join("vx-extension.toml");
        let current_config = ExtensionConfig::from_file(&config_path)?;
        let current_version = &current_config.extension.version;

        // Try to find the original source
        let metadata_path = target_path.join(".vx-source");
        if !metadata_path.exists() {
            return Ok(None);
        }

        let source = std::fs::read_to_string(&metadata_path).map_err(|e| {
            ExtensionError::io(
                "Failed to read extension source metadata",
                Some(metadata_path.clone()),
                e,
            )
        })?;

        let remote = RemoteSource::parse(&source)?;

        // Fetch latest version from remote
        let cache_path = self.get_cache_path(&remote);
        if cache_path.exists() {
            // Update the cache
            self.git_fetch(&cache_path)?;
        } else {
            // Clone fresh
            self.clone_or_update(&remote, &cache_path)?;
        }

        // Load remote version
        let remote_config_path = cache_path.join("vx-extension.toml");
        if !remote_config_path.exists() {
            return Ok(None);
        }

        let remote_config = ExtensionConfig::from_file(&remote_config_path)?;
        let remote_version = &remote_config.extension.version;

        if remote_version != current_version {
            Ok(Some(UpdateInfo {
                name: name.to_string(),
                current_version: current_version.clone(),
                latest_version: remote_version.clone(),
                source,
            }))
        } else {
            Ok(None)
        }
    }

    /// Uninstall an extension
    pub fn uninstall(&self, name: &str) -> ExtensionResult<()> {
        let target_path = self.user_dir.join(name);

        if !target_path.exists() {
            return Err(ExtensionError::ExtensionNotFound {
                name: name.to_string(),
                available: self.list_installed()?,
                searched_paths: vec![self.user_dir.clone()],
            });
        }

        // Remove the extension directory
        std::fs::remove_dir_all(&target_path).map_err(|e| {
            ExtensionError::io(
                format!("Failed to remove extension '{}'", name),
                Some(target_path),
                e,
            )
        })?;

        info!("Uninstalled extension '{}'", name);
        Ok(())
    }

    /// List installed extensions
    fn list_installed(&self) -> ExtensionResult<Vec<String>> {
        let mut extensions = Vec::new();

        if !self.user_dir.exists() {
            return Ok(extensions);
        }

        let entries = std::fs::read_dir(&self.user_dir).map_err(|e| {
            ExtensionError::io(
                "Failed to read extensions directory",
                Some(self.user_dir.clone()),
                e,
            )
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("vx-extension.toml").exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    extensions.push(name.to_string());
                }
            }
        }

        Ok(extensions)
    }

    /// Get the cache path for a remote source
    fn get_cache_path(&self, remote: &RemoteSource) -> PathBuf {
        match remote {
            RemoteSource::GitHub { owner, repo, .. } => {
                self.cache_dir.join("github.com").join(owner).join(repo)
            }
            RemoteSource::GitUrl { url, .. } => {
                // Create a safe directory name from the URL
                let safe_name = url.replace("://", "_").replace(['/', ':', '.'], "_");
                self.cache_dir.join("git").join(safe_name)
            }
        }
    }

    /// Clone or update a repository
    fn clone_or_update(&self, remote: &RemoteSource, cache_path: &Path) -> ExtensionResult<()> {
        if cache_path.exists() {
            debug!("Updating cached repository at {:?}", cache_path);
            self.git_fetch(cache_path)?;
            self.git_checkout(cache_path, remote.version())?;
        } else {
            debug!("Cloning repository to {:?}", cache_path);
            self.git_clone(&remote.clone_url(), cache_path)?;
            if let Some(version) = remote.version() {
                self.git_checkout(cache_path, Some(version))?;
            }
        }
        Ok(())
    }

    /// Clone a git repository
    fn git_clone(&self, url: &str, target: &Path) -> ExtensionResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ExtensionError::io(
                    "Failed to create cache directory",
                    Some(parent.to_path_buf()),
                    e,
                )
            })?;
        }

        let output = Command::new("git")
            .args(["clone", "--depth", "1", url])
            .arg(target)
            .output()
            .map_err(|e| ExtensionError::GitOperationFailed {
                operation: "clone".to_string(),
                reason: format!("Failed to execute git: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ExtensionError::GitOperationFailed {
                operation: "clone".to_string(),
                reason: stderr.to_string(),
            });
        }

        Ok(())
    }

    /// Fetch updates from remote
    fn git_fetch(&self, repo_path: &Path) -> ExtensionResult<()> {
        let output = Command::new("git")
            .args(["fetch", "--all", "--tags"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| ExtensionError::GitOperationFailed {
                operation: "fetch".to_string(),
                reason: format!("Failed to execute git: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Git fetch warning: {}", stderr);
        }

        Ok(())
    }

    /// Checkout a specific version/tag
    fn git_checkout(&self, repo_path: &Path, version: Option<&str>) -> ExtensionResult<()> {
        let target = version.unwrap_or("HEAD");

        // First try to checkout as a tag
        let output = Command::new("git")
            .args(["checkout", target])
            .current_dir(repo_path)
            .output()
            .map_err(|e| ExtensionError::GitOperationFailed {
                operation: "checkout".to_string(),
                reason: format!("Failed to execute git: {}", e),
            })?;

        if !output.status.success() {
            // Try with origin/ prefix for branches
            let output = Command::new("git")
                .args(["checkout", &format!("origin/{}", target)])
                .current_dir(repo_path)
                .output()
                .map_err(|e| ExtensionError::GitOperationFailed {
                    operation: "checkout".to_string(),
                    reason: format!("Failed to execute git: {}", e),
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ExtensionError::GitOperationFailed {
                    operation: "checkout".to_string(),
                    reason: format!("Failed to checkout '{}': {}", target, stderr),
                });
            }
        }

        Ok(())
    }

    /// Load and validate extension config from cache
    fn load_and_validate_config(
        &self,
        cache_path: &Path,
        source: &str,
    ) -> ExtensionResult<ExtensionConfig> {
        let config_path = cache_path.join("vx-extension.toml");

        if !config_path.exists() {
            return Err(ExtensionError::RemoteInstallFailed {
                src: source.to_string(),
                reason: "Repository does not contain a vx-extension.toml file".to_string(),
            });
        }

        ExtensionConfig::from_file(&config_path)
    }

    /// Copy extension from cache to user directory
    fn copy_extension(
        &self,
        cache_path: &Path,
        target_path: &Path,
        name: &str,
    ) -> ExtensionResult<()> {
        // Remove existing extension if present
        if target_path.exists() {
            std::fs::remove_dir_all(target_path).map_err(|e| {
                ExtensionError::io(
                    format!("Failed to remove existing extension '{}'", name),
                    Some(target_path.to_path_buf()),
                    e,
                )
            })?;
        }

        // Copy the extension (excluding .git directory)
        Self::copy_dir_recursive(cache_path, target_path)?;

        Ok(())
    }

    /// Recursively copy a directory, excluding .git
    fn copy_dir_recursive(src: &Path, dst: &Path) -> ExtensionResult<()> {
        std::fs::create_dir_all(dst).map_err(|e| {
            ExtensionError::io("Failed to create directory", Some(dst.to_path_buf()), e)
        })?;

        for entry in std::fs::read_dir(src).map_err(|e| {
            ExtensionError::io("Failed to read directory", Some(src.to_path_buf()), e)
        })? {
            let entry = entry.map_err(|e| {
                ExtensionError::io("Failed to read directory entry", Some(src.to_path_buf()), e)
            })?;

            let src_path = entry.path();
            let file_name = entry.file_name();

            // Skip .git directory
            if file_name == ".git" {
                continue;
            }

            let dst_path = dst.join(&file_name);

            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path).map_err(|e| {
                    ExtensionError::io(
                        format!("Failed to copy file: {}", src_path.display()),
                        Some(src_path.clone()),
                        e,
                    )
                })?;
            }
        }

        Ok(())
    }
}

impl Default for RemoteInstaller {
    fn default() -> Self {
        Self::new().expect("Failed to create remote installer")
    }
}

/// Information about an installed extension
#[derive(Debug, Clone)]
pub struct InstalledExtension {
    /// Extension name
    pub name: String,
    /// Extension version
    pub version: String,
    /// Installation path
    pub path: PathBuf,
    /// Original source
    pub source: String,
}

/// Information about an available update
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Extension name
    pub name: String,
    /// Current installed version
    pub current_version: String,
    /// Latest available version
    pub latest_version: String,
    /// Source to update from
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_shorthand() {
        let source = RemoteSource::parse("github:user/repo").unwrap();
        match source {
            RemoteSource::GitHub {
                owner,
                repo,
                version,
                subdir,
            } => {
                assert_eq!(owner, "user");
                assert_eq!(repo, "repo");
                assert!(version.is_none());
                assert!(subdir.is_none());
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn test_parse_github_shorthand_with_version() {
        let source = RemoteSource::parse("github:user/repo@v1.0.0").unwrap();
        match source {
            RemoteSource::GitHub {
                owner,
                repo,
                version,
                subdir,
            } => {
                assert_eq!(owner, "user");
                assert_eq!(repo, "repo");
                assert_eq!(version, Some("v1.0.0".to_string()));
                assert!(subdir.is_none());
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn test_parse_github_shorthand_with_subdir() {
        let source = RemoteSource::parse("github:user/repo/examples/hello").unwrap();
        match source {
            RemoteSource::GitHub {
                owner,
                repo,
                version,
                subdir,
            } => {
                assert_eq!(owner, "user");
                assert_eq!(repo, "repo");
                assert!(version.is_none());
                assert_eq!(subdir, Some("examples/hello".to_string()));
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn test_parse_github_https_url() {
        let source = RemoteSource::parse("https://github.com/user/repo").unwrap();
        match source {
            RemoteSource::GitHub {
                owner,
                repo,
                version,
                subdir,
            } => {
                assert_eq!(owner, "user");
                assert_eq!(repo, "repo");
                assert!(version.is_none());
                assert!(subdir.is_none());
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn test_parse_github_tree_url() {
        let source =
            RemoteSource::parse("https://github.com/user/repo/tree/main/examples/hello").unwrap();
        match source {
            RemoteSource::GitHub {
                owner,
                repo,
                version,
                subdir,
            } => {
                assert_eq!(owner, "user");
                assert_eq!(repo, "repo");
                assert_eq!(version, Some("main".to_string()));
                assert_eq!(subdir, Some("examples/hello".to_string()));
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn test_parse_github_ssh_url() {
        let source = RemoteSource::parse("git@github.com:user/repo.git").unwrap();
        match source {
            RemoteSource::GitHub {
                owner,
                repo,
                version,
                subdir,
            } => {
                assert_eq!(owner, "user");
                assert_eq!(repo, "repo");
                assert!(version.is_none());
                assert!(subdir.is_none());
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn test_parse_invalid_source() {
        let result = RemoteSource::parse("invalid-source");
        assert!(result.is_err());
    }

    #[test]
    fn test_clone_url() {
        let source = RemoteSource::GitHub {
            owner: "user".to_string(),
            repo: "repo".to_string(),
            version: None,
        };
        assert_eq!(source.clone_url(), "https://github.com/user/repo.git");
    }
}
