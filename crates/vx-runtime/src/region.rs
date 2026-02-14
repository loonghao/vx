//! Region detection utilities
//!
//! Detects the user's geographic region for mirror selection and CDN acceleration.
//! Uses system locale, timezone, and environment variable heuristics.

/// Detected geographic region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// China (mainland)
    China,
    /// Global / unknown
    Global,
}

impl Region {
    /// Get the region string used in provider.toml mirror configs
    pub fn as_str(&self) -> &str {
        match self {
            Region::China => "cn",
            Region::Global => "global",
        }
    }
}

/// Detect the user's region for download mirror selection
///
/// Decision logic (in order):
/// 1. `VX_MIRROR_REGION` env var → explicit override
/// 2. `VX_CDN=1` → implies China region
/// 3. `VX_CDN=0` → implies Global region
/// 4. CI environment → Global (CI has direct access)
/// 5. System locale/timezone heuristics → detect China
/// 6. Default → Global
pub fn detect_region() -> Region {
    // 1. Explicit region override
    if let Ok(val) = std::env::var("VX_MIRROR_REGION") {
        let val = val.to_lowercase();
        return match val.as_str() {
            "cn" | "china" => Region::China,
            _ => Region::Global,
        };
    }

    // 2. VX_CDN hints
    if let Ok(val) = std::env::var("VX_CDN") {
        let val = val.to_lowercase();
        return if matches!(val.as_str(), "1" | "true" | "yes" | "on") {
            Region::China
        } else {
            Region::Global
        };
    }

    // 3. CI environments have direct access
    if is_ci_environment() {
        return Region::Global;
    }

    // 4. Detect China environment via system indicators
    if is_china_environment() {
        return Region::China;
    }

    // 5. Default: Global
    Region::Global
}

/// Check if running in a CI environment
pub fn is_ci_environment() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("JENKINS_URL").is_ok()
        || std::env::var("TRAVIS").is_ok()
        || std::env::var("CIRCLECI").is_ok()
        || std::env::var("CODEBUILD_BUILD_ID").is_ok()
        || std::env::var("TF_BUILD").is_ok()
}

/// Detect if the system environment suggests a China-based user
///
/// Uses multiple heuristics:
/// - `LANG` / `LC_ALL` / `LANGUAGE` environment variables containing `zh_CN`
/// - `TZ` environment variable set to `Asia/Shanghai` or `Asia/Chongqing`
/// - Proxy environment variables containing `.cn` or `china`
pub fn is_china_environment() -> bool {
    // Check locale environment variables (Linux/macOS)
    for var in &["LANG", "LC_ALL", "LANGUAGE", "LC_CTYPE"] {
        if let Ok(val) = std::env::var(var) {
            let val_lower = val.to_lowercase();
            if val_lower.starts_with("zh_cn") || val_lower.contains("zh_cn") {
                return true;
            }
        }
    }

    // Check timezone
    if let Ok(tz) = std::env::var("TZ")
        && (tz == "Asia/Shanghai"
            || tz == "Asia/Chongqing"
            || tz == "Asia/Chungking"
            || tz == "PRC"
            || tz == "CST-8")
    {
        return true;
    }

    // Check common China-specific proxy environment hints
    if let Ok(proxy) = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
    {
        let proxy_lower = proxy.to_lowercase();
        if proxy_lower.contains(".cn") || proxy_lower.contains("china") {
            return true;
        }
    }

    false
}
