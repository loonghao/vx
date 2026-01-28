//! Conflict Detection for Version Range Locking (RFC 0023)
//!
//! This module implements conflict detection for tool version constraints.
//! When multiple tools have incompatible dependency requirements, this module
//! detects and reports those conflicts with actionable suggestions.

use crate::runtime_map::RuntimeMap;
use crate::version::constraint::Version;
use crate::version::request::VersionRequest;
use semver::VersionReq;
use std::collections::HashMap;

/// Dependency requirement from a tool
#[derive(Debug, Clone)]
pub struct DependencyRequirement {
    /// Tool that has this requirement
    pub from_tool: String,
    /// Version range requirement (e.g., ">=18", "<20")
    pub version_range: String,
}

impl std::fmt::Display for DependencyRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} requires {}", self.from_tool, self.version_range)
    }
}

/// Version conflict between tools
#[derive(Debug, Clone)]
pub struct Conflict {
    /// The runtime that has conflicting requirements
    pub runtime: String,
    /// List of conflicting requirements
    pub requirements: Vec<DependencyRequirement>,
    /// Human-readable conflict message
    pub message: String,
    /// Suggested resolutions
    pub suggestions: Vec<String>,
}

impl std::fmt::Display for Conflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Version conflict for {}:", self.runtime)?;
        writeln!(f)?;
        writeln!(f, "  Requirements:")?;
        for req in &self.requirements {
            writeln!(f, "    - {}", req)?;
        }
        writeln!(f)?;
        writeln!(f, "  Conflict: {}", self.message)?;
        if !self.suggestions.is_empty() {
            writeln!(f)?;
            writeln!(f, "  Suggestions:")?;
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                writeln!(f, "    {}. {}", i + 1, suggestion)?;
            }
        }
        Ok(())
    }
}

/// Result of merging multiple version requirements
#[derive(Debug, Clone)]
pub struct MergedRequirement {
    /// The merged version requirement (if compatible)
    pub requirement: Option<VersionReq>,
    /// Whether the requirements are compatible
    pub is_compatible: bool,
    /// Description of the merged requirement
    pub description: String,
}

/// Conflict detector for version constraints
pub struct ConflictDetector {
    runtime_map: RuntimeMap,
}

impl ConflictDetector {
    /// Create a new conflict detector
    pub fn new(runtime_map: RuntimeMap) -> Self {
        Self { runtime_map }
    }

    /// Create a conflict detector with default runtime map
    pub fn with_defaults() -> Self {
        Self {
            runtime_map: RuntimeMap::empty(),
        }
    }

    /// Detect conflicts in a set of tools and their version requests
    pub fn detect_conflicts(
        &self,
        tools: &[(String, VersionRequest)],
    ) -> Result<Vec<Conflict>, ConflictDetectionError> {
        let mut conflicts = Vec::new();
        let mut requirements: HashMap<String, Vec<DependencyRequirement>> = HashMap::new();

        // Collect all dependency requirements from each tool
        for (tool_name, version_req) in tools {
            if let Ok(constraints) = self.get_tool_constraints(tool_name, version_req) {
                for constraint in constraints {
                    requirements
                        .entry(constraint.runtime.clone())
                        .or_default()
                        .push(DependencyRequirement {
                            from_tool: tool_name.clone(),
                            version_range: constraint.version.clone(),
                        });
                }
            }
        }

        // Check each runtime that has multiple requirements
        for (runtime, reqs) in &requirements {
            if reqs.len() > 1 {
                match self.try_merge_requirements(reqs) {
                    Ok(_merged) => {
                        // Requirements are compatible, no conflict
                    }
                    Err(conflict_info) => {
                        conflicts.push(Conflict {
                            runtime: runtime.clone(),
                            requirements: reqs.clone(),
                            message: conflict_info,
                            suggestions: self.generate_suggestions(runtime, reqs),
                        });
                    }
                }
            }
        }

        Ok(conflicts)
    }

    /// Get constraints for a tool based on its version
    fn get_tool_constraints(
        &self,
        tool_name: &str,
        _version_req: &VersionRequest,
    ) -> Result<Vec<ToolConstraint>, ConflictDetectionError> {
        // Get dependencies from runtime map
        let mut constraints = Vec::new();

        if let Some(spec) = self.runtime_map.get(tool_name) {
            // Add constraints from the spec's runtime dependencies
            for dep in &spec.dependencies {
                // Build version constraint string from min/max versions
                let version_constraint = match (&dep.min_version, &dep.max_version) {
                    (Some(min), Some(max)) => format!(">={}, <={}", min, max),
                    (Some(min), None) => format!(">={}", min),
                    (None, Some(max)) => format!("<={}", max),
                    (None, None) => "*".to_string(), // Any version
                };

                constraints.push(ToolConstraint {
                    runtime: dep.runtime_name.clone(),
                    version: version_constraint,
                });
            }
        }

        Ok(constraints)
    }

    /// Try to merge multiple version requirements
    ///
    /// Returns Ok(MergedRequirement) if compatible, Err(conflict_message) if not
    fn try_merge_requirements(
        &self,
        reqs: &[DependencyRequirement],
    ) -> Result<MergedRequirement, String> {
        if reqs.is_empty() {
            return Ok(MergedRequirement {
                requirement: None,
                is_compatible: true,
                description: "No requirements".to_string(),
            });
        }

        // Parse all requirements
        let mut parsed: Vec<(&DependencyRequirement, Option<VersionReq>)> = Vec::new();
        for req in reqs {
            let version_req = parse_version_range(&req.version_range);
            parsed.push((req, version_req));
        }

        // Check if all requirements can be satisfied simultaneously
        // We do this by finding a version that satisfies all requirements
        // This is a simplified check - for complex constraints, we would need SAT solving

        // Collect all constraints
        let all_constraints: Vec<&str> = reqs.iter().map(|r| r.version_range.as_str()).collect();

        // Try to find a common satisfying version
        if let Some((conflict_a, conflict_b)) = self.find_conflicting_pair(&parsed) {
            return Err(format!(
                "{} version must satisfy both {} AND {}, which may be impossible",
                reqs[0].from_tool, conflict_a.version_range, conflict_b.version_range
            ));
        }

        Ok(MergedRequirement {
            requirement: None, // Would compute actual merged req
            is_compatible: true,
            description: format!("Merged: {}", all_constraints.join(", ")),
        })
    }

    /// Find a pair of requirements that conflict
    fn find_conflicting_pair<'a>(
        &self,
        parsed: &[(&'a DependencyRequirement, Option<VersionReq>)],
    ) -> Option<(&'a DependencyRequirement, &'a DependencyRequirement)> {
        for (i, (req_a, ver_req_a)) in parsed.iter().enumerate() {
            for (req_b, ver_req_b) in parsed.iter().skip(i + 1) {
                if let (Some(a), Some(b)) = (ver_req_a, ver_req_b) {
                    // Check if there's no version that satisfies both
                    if self.requirements_conflict(a, b) {
                        return Some((req_a, req_b));
                    }
                }
            }
        }
        None
    }

    /// Check if two version requirements conflict
    fn requirements_conflict(&self, a: &VersionReq, b: &VersionReq) -> bool {
        // Simple heuristic: check a range of common versions
        let test_versions = [
            Version::new(14, 0, 0),
            Version::new(16, 0, 0),
            Version::new(18, 0, 0),
            Version::new(20, 0, 0),
            Version::new(22, 0, 0),
        ];

        // If no test version satisfies both, they might conflict
        for v in &test_versions {
            let semver_v = semver::Version::new(v.major as u64, v.minor as u64, v.patch as u64);
            if a.matches(&semver_v) && b.matches(&semver_v) {
                return false; // Found a version that satisfies both
            }
        }

        // Could not find a satisfying version in our test set
        // This doesn't guarantee conflict but is a strong indicator
        true
    }

    /// Generate suggestions for resolving a conflict
    fn generate_suggestions(&self, runtime: &str, reqs: &[DependencyRequirement]) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Suggestion 1: Update tools to compatible versions
        for req in reqs {
            suggestions.push(format!(
                "Update {}: vx install {} <compatible-version>",
                req.from_tool, req.from_tool
            ));
        }

        // Suggestion 2: Use a specific runtime version
        suggestions.push(format!(
            "Install a compatible {} version manually: vx install {}@<version>",
            runtime, runtime
        ));

        // Suggestion 3: Check tool documentation
        suggestions.push(format!(
            "Check {} compatibility requirements in the tool documentation",
            runtime
        ));

        suggestions
    }

    /// Check if a specific version satisfies all requirements for a runtime
    pub fn version_satisfies_all(
        &self,
        version: &Version,
        requirements: &[DependencyRequirement],
    ) -> bool {
        let semver_v = semver::Version::new(
            version.major as u64,
            version.minor as u64,
            version.patch as u64,
        );

        for req in requirements {
            if let Some(ver_req) = parse_version_range(&req.version_range) {
                if !ver_req.matches(&semver_v) {
                    return false;
                }
            }
        }
        true
    }

    /// Find a version that satisfies all requirements
    pub fn find_satisfying_version(
        &self,
        requirements: &[DependencyRequirement],
        available_versions: &[Version],
    ) -> Option<Version> {
        for version in available_versions {
            if self.version_satisfies_all(version, requirements) {
                return Some(version.clone());
            }
        }
        None
    }
}

/// A constraint from a tool on a runtime
#[derive(Debug, Clone)]
struct ToolConstraint {
    /// The runtime this constraint applies to
    runtime: String,
    /// The version constraint (e.g., ">=18", "^20")
    version: String,
}

/// Error during conflict detection
#[derive(Debug, Clone)]
pub enum ConflictDetectionError {
    /// Failed to get tool information
    ToolNotFound(String),
    /// Invalid version constraint
    InvalidConstraint(String),
}

impl std::fmt::Display for ConflictDetectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ToolNotFound(name) => write!(f, "Tool not found: {}", name),
            Self::InvalidConstraint(msg) => write!(f, "Invalid constraint: {}", msg),
        }
    }
}

impl std::error::Error for ConflictDetectionError {}

/// Parse a version range string into a semver VersionReq
fn parse_version_range(range: &str) -> Option<VersionReq> {
    let range = range.trim();

    // Handle common formats
    let normalized = if range.starts_with(">=")
        || range.starts_with("<=")
        || range.starts_with('>')
        || range.starts_with('<')
        || range.starts_with('^')
        || range.starts_with('~')
        || range.starts_with('=')
    {
        range.to_string()
    } else if range.contains('.') {
        // Assume exact version
        format!("={}", range)
    } else {
        // Assume major version
        format!("^{}.0.0", range)
    };

    VersionReq::parse(&normalized).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_range() {
        assert!(parse_version_range(">=18").is_some());
        assert!(parse_version_range("^20.0.0").is_some());
        assert!(parse_version_range("<21").is_some());
        assert!(parse_version_range("18.0.0").is_some());
        assert!(parse_version_range("18").is_some());
    }

    #[test]
    fn test_dependency_requirement_display() {
        let req = DependencyRequirement {
            from_tool: "vite".to_string(),
            version_range: ">=18".to_string(),
        };
        assert_eq!(format!("{}", req), "vite requires >=18");
    }

    #[test]
    fn test_conflict_display() {
        let conflict = Conflict {
            runtime: "node".to_string(),
            requirements: vec![
                DependencyRequirement {
                    from_tool: "vite".to_string(),
                    version_range: ">=18".to_string(),
                },
                DependencyRequirement {
                    from_tool: "legacy-tool".to_string(),
                    version_range: "<16".to_string(),
                },
            ],
            message: "Node.js version must satisfy both >=18 AND <16".to_string(),
            suggestions: vec!["Update legacy-tool".to_string()],
        };
        let output = format!("{}", conflict);
        assert!(output.contains("Version conflict for node"));
        assert!(output.contains("vite requires >=18"));
        assert!(output.contains("legacy-tool requires <16"));
    }

    #[test]
    fn test_conflict_detector_creation() {
        let detector = ConflictDetector::with_defaults();
        // Should not panic
        let tools: Vec<(String, VersionRequest)> = vec![];
        let conflicts = detector.detect_conflicts(&tools);
        assert!(conflicts.is_ok());
        assert!(conflicts.unwrap().is_empty());
    }

    #[test]
    fn test_version_satisfies_all() {
        let detector = ConflictDetector::with_defaults();
        let version = Version::new(20, 0, 0);
        let requirements = vec![
            DependencyRequirement {
                from_tool: "tool_a".to_string(),
                version_range: ">=18".to_string(),
            },
            DependencyRequirement {
                from_tool: "tool_b".to_string(),
                version_range: "<22".to_string(),
            },
        ];

        assert!(detector.version_satisfies_all(&version, &requirements));
    }

    #[test]
    fn test_version_does_not_satisfy_all() {
        let detector = ConflictDetector::with_defaults();
        let version = Version::new(16, 0, 0);
        let requirements = vec![
            DependencyRequirement {
                from_tool: "tool_a".to_string(),
                version_range: ">=18".to_string(),
            },
            DependencyRequirement {
                from_tool: "tool_b".to_string(),
                version_range: "<22".to_string(),
            },
        ];

        assert!(!detector.version_satisfies_all(&version, &requirements));
    }

    #[test]
    fn test_find_satisfying_version() {
        let detector = ConflictDetector::with_defaults();
        let requirements = vec![
            DependencyRequirement {
                from_tool: "tool_a".to_string(),
                version_range: ">=18".to_string(),
            },
            DependencyRequirement {
                from_tool: "tool_b".to_string(),
                version_range: "<22".to_string(),
            },
        ];

        let available = vec![
            Version::new(16, 0, 0),
            Version::new(18, 0, 0),
            Version::new(20, 0, 0),
            Version::new(22, 0, 0),
        ];

        let result = detector.find_satisfying_version(&requirements, &available);
        assert!(result.is_some());
        let v = result.unwrap();
        // Should be 18 or 20
        assert!(v.major >= 18 && v.major < 22);
    }
}
