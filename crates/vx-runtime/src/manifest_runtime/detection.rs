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
///
/// On Windows, searches for `.exe`, `.cmd`, and bare name in priority order.
/// Prefers extension-bearing files (`.exe`, `.cmd`) over bare names to avoid
/// accidentally picking up Unix shell scripts (e.g., `npm` bash script vs `npm.cmd`).
pub fn find_executable_recursive(
    dir: &std::path::Path,
    exe_name: &str,
    _exe_with_ext: &str,
    max_depth: usize,
) -> Option<std::path::PathBuf> {
    // Build candidate names in priority order
    let candidates: Vec<String> = if cfg!(windows) {
        vec![
            format!("{}.exe", exe_name),
            format!("{}.cmd", exe_name),
            exe_name.to_string(),
        ]
    } else {
        vec![exe_name.to_string()]
    };
    find_exe_recurse(dir, &candidates, 0, max_depth)
}

fn find_exe_recurse(
    dir: &std::path::Path,
    candidates: &[String],
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

    // Collect files and subdirectories in one pass
    let mut found_files: Vec<std::path::PathBuf> = Vec::new();
    let mut subdirs: Vec<std::path::PathBuf> = Vec::new();

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if path.is_file() && candidates.iter().any(|c| c == name) {
                found_files.push(path);
            } else if path.is_dir() {
                // Skip known non-target directories
                if !matches!(
                    name,
                    "node_modules" | "lib" | "share" | "include" | "man" | "doc" | "docs"
                ) {
                    subdirs.push(path);
                }
            }
        }
    }

    // Return the best match according to candidate priority order
    // (e.g., .exe before .cmd before bare name)
    for candidate in candidates {
        if let Some(path) = found_files.iter().find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == candidate.as_str())
        }) {
            return Some(path.clone());
        }
    }

    // Recurse into subdirectories
    for subdir in subdirs {
        if let Some(found) = find_exe_recurse(&subdir, candidates, current_depth + 1, max_depth) {
            return Some(found);
        }
    }

    None
}
