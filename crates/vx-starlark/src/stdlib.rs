//! Standard library for Starlark providers
//!
//! This module defines the built-in functions and types available to
//! Starlark provider scripts.

/// Built-in functions available to Starlark scripts
pub mod functions {
    /// Log a debug message
    pub fn debug(msg: &str) {
        tracing::debug!("{}", msg);
    }

    /// Log an info message
    pub fn info(msg: &str) {
        tracing::info!("{}", msg);
    }

    /// Log a warning message
    pub fn warn(msg: &str) {
        tracing::warn!("{}", msg);
    }

    /// Log an error message
    pub fn error(msg: &str) {
        tracing::error!("{}", msg);
    }
}

/// Template string utilities
pub mod template {
    use std::collections::HashMap;

    /// Render a template string with variables
    ///
    /// Supports `{variable}` syntax for variable substitution.
    ///
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// use vx_starlark::stdlib::template::render;
    ///
    /// let mut vars = HashMap::new();
    /// vars.insert("version", "20.0.0");
    /// vars.insert("platform", "windows");
    ///
    /// let result = render("https://example.com/{version}/tool-{platform}.zip", &vars);
    /// assert_eq!(result, "https://example.com/20.0.0/tool-windows.zip");
    /// ```
    pub fn render(template: &str, vars: &HashMap<&str, &str>) -> String {
        let mut result = template.to_string();
        for (key, value) in vars {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }

    /// Render with platform-specific variables
    pub fn render_with_platform(template: &str, os: &str, arch: &str, version: &str) -> String {
        let mut vars = HashMap::new();
        vars.insert("os", os);
        vars.insert("arch", arch);

        // Create owned strings for format results
        let platform = format!("{}-{}", os, arch);
        let vversion = format!("v{}", version);

        vars.insert("platform", &platform);
        vars.insert("version", version);
        vars.insert("vversion", &vversion);

        render(template, &vars)
    }
}

/// Version comparison utilities
pub mod version {
    use std::cmp::Ordering;

    /// Compare two semantic version strings
    ///
    /// Returns:
    /// - `Some(-1)` if a < b
    /// - `Some(0)` if a == b
    /// - `Some(1)` if a > b
    /// - `None` if comparison failed
    pub fn compare(a: &str, b: &str) -> Option<i32> {
        let a_parts = parse_version(a)?;
        let b_parts = parse_version(b)?;

        for (ai, bi) in a_parts.iter().zip(b_parts.iter()) {
            match ai.cmp(bi) {
                Ordering::Less => return Some(-1),
                Ordering::Greater => return Some(1),
                Ordering::Equal => continue,
            }
        }

        match a_parts.len().cmp(&b_parts.len()) {
            Ordering::Less => Some(-1),
            Ordering::Greater => Some(1),
            Ordering::Equal => Some(0),
        }
    }

    /// Check if version a is greater than b
    pub fn gt(a: &str, b: &str) -> bool {
        compare(a, b).map(|c| c > 0).unwrap_or(false)
    }

    /// Check if version a is less than b
    pub fn lt(a: &str, b: &str) -> bool {
        compare(a, b).map(|c| c < 0).unwrap_or(false)
    }

    /// Check if version a equals b
    pub fn eq(a: &str, b: &str) -> bool {
        compare(a, b).map(|c| c == 0).unwrap_or(false)
    }

    /// Check if version a is greater than or equal to b
    pub fn gte(a: &str, b: &str) -> bool {
        compare(a, b).map(|c| c >= 0).unwrap_or(false)
    }

    /// Check if version a is less than or equal to b
    pub fn lte(a: &str, b: &str) -> bool {
        compare(a, b).map(|c| c <= 0).unwrap_or(false)
    }

    /// Parse a version string into numeric parts
    fn parse_version(version: &str) -> Option<Vec<u64>> {
        let version = version.trim_start_matches('v');
        version
            .split('.')
            .map(|part| {
                // Handle versions like "1.0.0-rc1" by taking only the numeric part
                part.split(|c: char| !c.is_ascii_digit())
                    .next()
                    .unwrap_or("")
                    .parse()
                    .ok()
            })
            .collect()
    }

    /// Strip the 'v' prefix from a version string
    pub fn strip_v_prefix(version: &str) -> &str {
        version.trim_start_matches('v')
    }
}

/// URL utilities
pub mod url {
    /// Extract the filename from a URL
    pub fn filename(url: &str) -> Option<&str> {
        url.rsplit('/').next()
    }

    /// Get the host from a URL
    pub fn host(url: &str) -> Option<&str> {
        url.strip_prefix("http://")
            .or_else(|| url.strip_prefix("https://"))
            .and_then(|s| s.split('/').next())
    }
}
