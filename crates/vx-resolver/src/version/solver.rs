//! Version solver implementation

use super::constraint::Version;
use super::request::VersionRequest;
use super::resolved::ResolvedVersion;
use super::strategy::{
    GitVersionStrategy, GoVersionStrategy, Pep440Strategy, SemverStrategy, VersionStrategy,
};
use crate::runtime_spec::Ecosystem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vx_versions::VersionInfo;

/// Version solver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    /// Whether to prefer LTS versions
    pub prefer_lts: bool,
    /// Whether to allow prerelease versions
    pub allow_prerelease: bool,
    /// Default ecosystem for unknown tools
    pub default_ecosystem: Ecosystem,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            prefer_lts: true,
            allow_prerelease: false,
            default_ecosystem: Ecosystem::NodeJs,
        }
    }
}

/// Solver status (inspired by rez)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SolverStatus {
    /// Not yet started
    #[default]
    Pending,
    /// Resolution successful
    Solved,
    /// Cannot satisfy constraints
    Failed,
    /// Cyclic dependency detected
    Cyclic,
    /// Resolution in progress
    InProgress,
}

/// Solver error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolverError {
    /// No version found matching the constraint
    NoVersionFound { tool: String, constraint: String },
    /// Conflicting constraints
    ConflictingConstraints {
        tool: String,
        constraints: Vec<String>,
    },
    /// Network/fetch error
    FetchError { tool: String, error: String },
    /// Invalid version format
    InvalidVersion { tool: String, version: String },
}

impl std::fmt::Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoVersionFound { tool, constraint } => {
                write!(f, "No version found for {} matching {}", tool, constraint)
            }
            Self::ConflictingConstraints { tool, constraints } => {
                write!(
                    f,
                    "Conflicting constraints for {}: {}",
                    tool,
                    constraints.join(", ")
                )
            }
            Self::FetchError { tool, error } => {
                write!(f, "Failed to fetch versions for {}: {}", tool, error)
            }
            Self::InvalidVersion { tool, version } => {
                write!(f, "Invalid version {} for {}", version, tool)
            }
        }
    }
}

impl std::error::Error for SolverError {}

/// Solver result
#[derive(Debug, Clone, Default)]
pub struct SolverResult {
    /// Solver status
    pub status: SolverStatus,
    /// Resolved versions
    pub resolved: HashMap<String, ResolvedVersion>,
    /// Errors encountered
    pub errors: Vec<SolverError>,
}

impl SolverResult {
    /// Create a successful result
    pub fn success(resolved: HashMap<String, ResolvedVersion>) -> Self {
        Self {
            status: SolverStatus::Solved,
            resolved,
            errors: vec![],
        }
    }

    /// Create a failed result
    pub fn failed(errors: Vec<SolverError>) -> Self {
        Self {
            status: SolverStatus::Failed,
            resolved: HashMap::new(),
            errors,
        }
    }

    /// Check if resolution was successful
    pub fn is_success(&self) -> bool {
        self.status == SolverStatus::Solved
    }

    /// Get a resolved version by tool name
    pub fn get(&self, tool: &str) -> Option<&ResolvedVersion> {
        self.resolved.get(tool)
    }
}

/// Version solver
pub struct VersionSolver {
    /// Version strategies by ecosystem
    strategies: HashMap<Ecosystem, Box<dyn VersionStrategy>>,
    /// Configuration
    config: SolverConfig,
}

impl VersionSolver {
    /// Create a new version solver
    pub fn new() -> Self {
        Self::with_config(SolverConfig::default())
    }

    /// Create a solver with custom config
    pub fn with_config(config: SolverConfig) -> Self {
        let mut solver = Self {
            strategies: HashMap::new(),
            config,
        };

        // Register default strategies
        solver.register_strategy(Box::new(SemverStrategy::new(Ecosystem::NodeJs)));
        solver.register_strategy(Box::new(Pep440Strategy::new()));
        solver.register_strategy(Box::new(GoVersionStrategy::new()));
        solver.register_strategy(Box::new(GitVersionStrategy::new()));
        solver.register_strategy(Box::new(SemverStrategy::new(Ecosystem::Rust)));
        solver.register_strategy(Box::new(SemverStrategy::new(Ecosystem::Generic)));

        solver
    }

    /// Register a version strategy
    pub fn register_strategy(&mut self, strategy: Box<dyn VersionStrategy>) {
        self.strategies.insert(strategy.ecosystem(), strategy);
    }

    /// Get strategy for an ecosystem
    pub fn get_strategy(&self, ecosystem: &Ecosystem) -> &dyn VersionStrategy {
        self.strategies
            .get(ecosystem)
            .map(|s| s.as_ref())
            .unwrap_or_else(|| {
                self.strategies
                    .get(&self.config.default_ecosystem)
                    .map(|s| s.as_ref())
                    .unwrap_or_else(|| {
                        // Fallback to Node.js strategy
                        self.strategies
                            .get(&Ecosystem::NodeJs)
                            .expect("NodeJs strategy must always be registered")
                            .as_ref()
                    })
            })
    }

    /// Resolve a single tool's version
    pub fn resolve(
        &self,
        tool: &str,
        request: &VersionRequest,
        available: &[VersionInfo],
        ecosystem: &Ecosystem,
    ) -> Result<ResolvedVersion, SolverError> {
        let strategy = self.get_strategy(ecosystem);

        // Filter by prerelease setting
        let filtered: Vec<_> = if self.config.allow_prerelease {
            available.to_vec()
        } else {
            available
                .iter()
                .filter(|v| !v.prerelease)
                .cloned()
                .collect()
        };

        // If prefer_lts is set and we have LTS versions, prefer them for Latest constraint
        let candidates = if self.config.prefer_lts
            && matches!(
                request.constraint,
                super::constraint::VersionConstraint::Latest
            ) {
            let lts_versions: Vec<_> = filtered.iter().filter(|v| v.lts).cloned().collect();
            if !lts_versions.is_empty() {
                lts_versions
            } else {
                filtered
            }
        } else {
            filtered
        };

        strategy
            .select_best_match(&request.constraint, &candidates)
            .map(|mut resolved| {
                resolved.resolved_from = request.raw.clone();
                resolved.source = ecosystem.to_string();
                resolved
            })
            .ok_or_else(|| SolverError::NoVersionFound {
                tool: tool.to_string(),
                constraint: request.constraint.to_string(),
            })
    }

    /// Resolve multiple tools at once
    pub fn resolve_all(
        &self,
        requests: &[(String, VersionRequest, Ecosystem, Vec<VersionInfo>)],
    ) -> SolverResult {
        let mut resolved = HashMap::new();
        let mut errors = Vec::new();

        for (tool, request, ecosystem, available) in requests {
            match self.resolve(tool, request, available, ecosystem) {
                Ok(version) => {
                    resolved.insert(tool.clone(), version);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            SolverResult::success(resolved)
        } else {
            SolverResult {
                status: SolverStatus::Failed,
                resolved,
                errors,
            }
        }
    }

    /// Resolve a version string to a full version
    ///
    /// This is a convenience method that parses the request and resolves it.
    pub fn resolve_version_string(
        &self,
        tool: &str,
        version_str: &str,
        available: &[VersionInfo],
        ecosystem: &Ecosystem,
    ) -> Result<String, SolverError> {
        let request = VersionRequest::parse(version_str);
        let resolved = self.resolve(tool, &request, available, ecosystem)?;
        Ok(resolved.version_string())
    }

    /// Check if a version string matches a constraint
    pub fn version_satisfies(
        &self,
        version_str: &str,
        constraint_str: &str,
        ecosystem: &Ecosystem,
    ) -> bool {
        let Some(version) = Version::parse(version_str) else {
            return false;
        };

        let request = VersionRequest::parse(constraint_str);
        let strategy = self.get_strategy(ecosystem);
        strategy.satisfies(&version, &request.constraint)
    }
}

impl Default for VersionSolver {
    fn default() -> Self {
        Self::new()
    }
}
