//! Project type detection and configuration file discovery

use crate::{parsers::*, types::*, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Type alias for project detection result
type ProjectDetectionResult = (ProjectType, PathBuf, HashMap<String, String>);

/// Detect project information and configuration files
pub fn detect_project_info() -> Result<Option<ProjectInfo>> {
    let current_dir = get_current_directory()?;
    let detections = detect_all_project_types(&current_dir)?;

    if detections.is_empty() {
        return Ok(None);
    }

    let (detected_types, all_tool_versions) = collect_detection_results(&detections);
    let project_type = determine_project_type(&detected_types);
    let config_file = select_primary_config_file(&project_type, &current_dir);

    Ok(Some(ProjectInfo {
        project_type,
        config_file,
        tool_versions: all_tool_versions,
    }))
}

/// Get current working directory with error handling
fn get_current_directory() -> Result<PathBuf> {
    std::env::current_dir().map_err(|e| crate::error::ConfigError::Detection {
        message: format!("Failed to get current directory: {}", e),
    })
}

/// Detect all supported project types in the current directory
fn detect_all_project_types(current_dir: &Path) -> Result<Vec<ProjectDetectionResult>> {
    let mut detections = Vec::new();

    // Check for Python project (pyproject.toml)
    if let Some(detection) = detect_python_project(current_dir)? {
        detections.push(detection);
    }

    // Check for Rust project (Cargo.toml)
    if let Some(detection) = detect_rust_project(current_dir)? {
        detections.push(detection);
    }

    // Check for Node.js project (package.json)
    if let Some(detection) = detect_node_project(current_dir)? {
        detections.push(detection);
    }

    // Check for Go project (go.mod)
    if let Some(detection) = detect_go_project(current_dir)? {
        detections.push(detection);
    }

    Ok(detections)
}

/// Detect Python project and extract tool versions
fn detect_python_project(current_dir: &Path) -> Result<Option<ProjectDetectionResult>> {
    let pyproject_path = current_dir.join("pyproject.toml");
    if pyproject_path.exists() {
        if let Ok(versions) = parse_pyproject_toml(&pyproject_path) {
            return Ok(Some((ProjectType::Python, pyproject_path, versions)));
        }
    }
    Ok(None)
}

/// Detect Rust project and extract tool versions
fn detect_rust_project(current_dir: &Path) -> Result<Option<ProjectDetectionResult>> {
    let cargo_path = current_dir.join("Cargo.toml");
    if cargo_path.exists() {
        if let Ok(versions) = parse_cargo_toml(&cargo_path) {
            return Ok(Some((ProjectType::Rust, cargo_path, versions)));
        }
    }
    Ok(None)
}

/// Detect Node.js project and extract tool versions
fn detect_node_project(current_dir: &Path) -> Result<Option<ProjectDetectionResult>> {
    let package_path = current_dir.join("package.json");
    if package_path.exists() {
        if let Ok(versions) = parse_package_json(&package_path) {
            return Ok(Some((ProjectType::Node, package_path, versions)));
        }
    }
    Ok(None)
}

/// Detect Go project and extract tool versions
fn detect_go_project(current_dir: &Path) -> Result<Option<ProjectDetectionResult>> {
    let gomod_path = current_dir.join("go.mod");
    if gomod_path.exists() {
        if let Ok(versions) = parse_go_mod(&gomod_path) {
            return Ok(Some((ProjectType::Go, gomod_path, versions)));
        }
    }
    Ok(None)
}

/// Collect detection results and merge tool versions
fn collect_detection_results(
    detections: &[ProjectDetectionResult],
) -> (Vec<ProjectType>, HashMap<String, String>) {
    let mut detected_types = Vec::new();
    let mut all_tool_versions = HashMap::new();

    for (project_type, _path, tool_versions) in detections {
        detected_types.push(project_type.clone());
        all_tool_versions.extend(tool_versions.clone());
    }

    (detected_types, all_tool_versions)
}

/// Determine the final project type based on detected types
fn determine_project_type(detected_types: &[ProjectType]) -> ProjectType {
    if detected_types.len() == 1 {
        detected_types[0].clone()
    } else {
        ProjectType::Mixed
    }
}

/// Select the primary configuration file based on project type
fn select_primary_config_file(project_type: &ProjectType, current_dir: &Path) -> PathBuf {
    match project_type {
        ProjectType::Python => current_dir.join("pyproject.toml"),
        ProjectType::Rust => current_dir.join("Cargo.toml"),
        ProjectType::Node => current_dir.join("package.json"),
        ProjectType::Go => current_dir.join("go.mod"),
        ProjectType::Mixed => {
            // Prefer pyproject.toml for mixed projects
            let pyproject_path = current_dir.join("pyproject.toml");
            if pyproject_path.exists() {
                pyproject_path
            } else {
                let cargo_path = current_dir.join("Cargo.toml");
                if cargo_path.exists() {
                    cargo_path
                } else {
                    let package_path = current_dir.join("package.json");
                    if package_path.exists() {
                        package_path
                    } else {
                        current_dir.join("go.mod")
                    }
                }
            }
        }
        ProjectType::Unknown => current_dir.join(""),
    }
}
