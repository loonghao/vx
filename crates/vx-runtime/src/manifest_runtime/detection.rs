//! Version detection and executable search logic for manifest-driven runtimes.

use anyhow::Result;

use super::types::DetectionConfig;

/// Detect the installed version using the detection configuration.
///
/// Runs the detection command and extracts the version string using the
/// configured regex pattern.
pub async fn detect_version(
    executable: &str,
    detection: &DetectionConfig,
) -> Result<Option<String>> {
    // Find the executable
    let executable_path = match which::which(executable) {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };

    // Build the command
    let command = detection.command.replace("{executable}", executable);
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(None);
    }

    // Execute the command
    let output = tokio::process::Command::new(&executable_path)
        .args(&parts[1..])
        .output()
        .await?;

    if !output.status.success() {
        return Ok(None);
    }

    // Parse the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Extract version using regex
    let re = regex::Regex::new(&detection.pattern)?;
    if let Some(captures) = re.captures(&combined)
        && let Some(version) = captures.get(1)
    {
        return Ok(Some(version.as_str().to_string()));
    }

    Ok(None)
}

/// Recursively search for an executable in a directory up to a given depth.
///
/// This is used by `prepare_execution` to find bundled executables that may
/// be nested inside an extra directory level (e.g., when strip_prefix was
/// not applied or the archive has a different nesting structure than expected).
pub fn find_executable_recursive(
    dir: &std::path::Path,
    exe_name: &str,
    exe_with_ext: &str,
    max_depth: usize,
) -> Option<std::path::PathBuf> {
    find_exe_recurse(dir, exe_name, exe_with_ext, 0, max_depth)
}

fn find_exe_recurse(
    dir: &std::path::Path,
    exe_name: &str,
    exe_with_ext: &str,
    current_depth: usize,
    max_depth: usize,
) -> Option<std::path::PathBuf> {
    if current_depth > max_depth {
        return None;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return None,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && path.is_file()
            && (name == exe_name || name == exe_with_ext)
        {
            return Some(path);
        }
        if path.is_dir() {
            // Skip known non-target directories
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str())
                && matches!(
                    dir_name,
                    "node_modules" | "lib" | "share" | "include" | "man" | "doc" | "docs"
                )
            {
                continue;
            }
            if let Some(found) =
                find_exe_recurse(&path, exe_name, exe_with_ext, current_depth + 1, max_depth)
            {
                return Some(found);
            }
        }
    }
    None
}
