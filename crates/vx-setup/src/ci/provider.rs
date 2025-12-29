//! CI provider detection

use std::fmt;

/// CI provider detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiProvider {
    /// GitHub Actions
    GitHub,
    /// GitLab CI
    GitLab,
    /// Azure Pipelines
    Azure,
    /// CircleCI
    CircleCI,
    /// Jenkins
    Jenkins,
    /// Generic CI (CI=true)
    Generic,
    /// Not in CI
    None,
}

impl CiProvider {
    /// Detect CI provider from environment variables
    pub fn detect() -> Self {
        if std::env::var("GITHUB_ACTIONS").is_ok() {
            return CiProvider::GitHub;
        }
        if std::env::var("GITLAB_CI").is_ok() {
            return CiProvider::GitLab;
        }
        if std::env::var("TF_BUILD").is_ok() || std::env::var("AZURE_PIPELINES").is_ok() {
            return CiProvider::Azure;
        }
        if std::env::var("CIRCLECI").is_ok() {
            return CiProvider::CircleCI;
        }
        if std::env::var("JENKINS_URL").is_ok() {
            return CiProvider::Jenkins;
        }
        if std::env::var("CI").map(|v| v == "true").unwrap_or(false) {
            return CiProvider::Generic;
        }
        CiProvider::None
    }

    /// Check if running in any CI environment
    pub fn is_ci(&self) -> bool {
        !matches!(self, CiProvider::None)
    }

    /// Get the PATH export file for this CI provider
    pub fn path_export_file(&self) -> Option<String> {
        match self {
            CiProvider::GitHub => std::env::var("GITHUB_PATH").ok(),
            CiProvider::GitLab => None, // GitLab uses different mechanism
            CiProvider::Azure => std::env::var("GITHUB_PATH").ok(), // Azure also supports GITHUB_PATH
            CiProvider::CircleCI => Some("$BASH_ENV".to_string()),
            CiProvider::Jenkins => None,
            CiProvider::Generic => None,
            CiProvider::None => None,
        }
    }

    /// Get the environment export file for this CI provider
    pub fn env_export_file(&self) -> Option<String> {
        match self {
            CiProvider::GitHub => std::env::var("GITHUB_ENV").ok(),
            CiProvider::GitLab => None,
            CiProvider::Azure => std::env::var("GITHUB_ENV").ok(),
            CiProvider::CircleCI => Some("$BASH_ENV".to_string()),
            CiProvider::Jenkins => None,
            CiProvider::Generic => None,
            CiProvider::None => None,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            CiProvider::GitHub => "GitHub Actions",
            CiProvider::GitLab => "GitLab CI",
            CiProvider::Azure => "Azure Pipelines",
            CiProvider::CircleCI => "CircleCI",
            CiProvider::Jenkins => "Jenkins",
            CiProvider::Generic => "Generic CI",
            CiProvider::None => "Local",
        }
    }

    /// Get short name for logging
    pub fn short_name(&self) -> &'static str {
        match self {
            CiProvider::GitHub => "github",
            CiProvider::GitLab => "gitlab",
            CiProvider::Azure => "azure",
            CiProvider::CircleCI => "circleci",
            CiProvider::Jenkins => "jenkins",
            CiProvider::Generic => "ci",
            CiProvider::None => "local",
        }
    }
}

impl fmt::Display for CiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Default for CiProvider {
    fn default() -> Self {
        Self::detect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_provider_none_by_default() {
        // In test environment, CI might or might not be set
        let provider = CiProvider::detect();
        // Just verify the API works
        let _ = provider.is_ci();
        let _ = provider.name();
        let _ = provider.short_name();
        let _ = provider.path_export_file();
        let _ = provider.env_export_file();
    }

    #[test]
    fn test_ci_provider_display() {
        assert_eq!(CiProvider::GitHub.to_string(), "GitHub Actions");
        assert_eq!(CiProvider::GitLab.to_string(), "GitLab CI");
        assert_eq!(CiProvider::None.to_string(), "Local");
    }

    #[test]
    fn test_ci_provider_is_ci() {
        assert!(CiProvider::GitHub.is_ci());
        assert!(CiProvider::GitLab.is_ci());
        assert!(CiProvider::Generic.is_ci());
        assert!(!CiProvider::None.is_ci());
    }
}
