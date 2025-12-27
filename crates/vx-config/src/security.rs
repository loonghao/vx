//! Security scanning and configuration
//!
//! This module provides security-related functionality:
//! - Dependency vulnerability scanning
//! - Secret detection
//! - SAST integration
//! - License compliance

use crate::SecurityConfig;
use serde::{Deserialize, Serialize};

/// Security scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResult {
    /// Vulnerabilities found
    pub vulnerabilities: Vec<Vulnerability>,
    /// Secrets detected
    pub secrets: Vec<SecretFinding>,
    /// License violations
    pub license_violations: Vec<LicenseViolation>,
    /// Overall status
    pub status: ScanStatus,
}

/// Vulnerability finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// CVE ID
    pub id: String,
    /// Affected package
    pub package: String,
    /// Affected version
    pub version: String,
    /// Severity level
    pub severity: Severity,
    /// Description
    pub description: String,
    /// Fixed version (if available)
    pub fixed_version: Option<String>,
    /// References
    pub references: Vec<String>,
}

/// Secret finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretFinding {
    /// File path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Secret type
    pub secret_type: String,
    /// Matched pattern
    pub pattern: String,
    /// Is in baseline (ignored)
    pub in_baseline: bool,
}

/// License violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseViolation {
    /// Package name
    pub package: String,
    /// Package version
    pub version: String,
    /// License
    pub license: String,
    /// Reason for violation
    pub reason: String,
}

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    /// Parse severity from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(Severity::Low),
            "medium" => Some(Severity::Medium),
            "high" => Some(Severity::High),
            "critical" => Some(Severity::Critical),
            _ => None,
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("Invalid severity: {}", s))
    }
}

/// Scan status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanStatus {
    Pass,
    Warn,
    Fail,
}

/// Security scanner
pub struct SecurityScanner {
    config: SecurityConfig,
}

impl SecurityScanner {
    /// Create a new security scanner
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Check if scanning is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled.unwrap_or(false)
    }

    /// Get fail threshold severity
    pub fn fail_threshold(&self) -> Severity {
        self.config
            .fail_on
            .as_ref()
            .and_then(|s| Severity::parse(s))
            .unwrap_or(Severity::High)
    }

    /// Check if a vulnerability should fail the scan
    pub fn should_fail(&self, severity: Severity) -> bool {
        severity >= self.fail_threshold()
    }

    /// Get ignored CVEs
    pub fn ignored_cves(&self) -> Vec<String> {
        self.config
            .audit
            .as_ref()
            .map(|a| a.ignore.clone())
            .unwrap_or_default()
    }

    /// Check if a CVE is ignored
    pub fn is_cve_ignored(&self, cve: &str) -> bool {
        self.ignored_cves().contains(&cve.to_string())
    }

    /// Get allowed licenses
    pub fn allowed_licenses(&self) -> &[String] {
        &self.config.allowed_licenses
    }

    /// Get denied licenses
    pub fn denied_licenses(&self) -> &[String] {
        &self.config.denied_licenses
    }

    /// Check if a license is allowed
    pub fn is_license_allowed(&self, license: &str) -> bool {
        let allowed = self.allowed_licenses();
        let denied = self.denied_licenses();

        // If denied list is specified and license is in it, deny
        if !denied.is_empty() && denied.iter().any(|l| l.eq_ignore_ascii_case(license)) {
            return false;
        }

        // If allowed list is specified, license must be in it
        if !allowed.is_empty() {
            return allowed.iter().any(|l| l.eq_ignore_ascii_case(license));
        }

        // Default: allow
        true
    }

    /// Determine scan status from results
    pub fn determine_status(&self, result: &SecurityScanResult) -> ScanStatus {
        let threshold = self.fail_threshold();

        // Check vulnerabilities
        for vuln in &result.vulnerabilities {
            if !self.is_cve_ignored(&vuln.id) && vuln.severity >= threshold {
                return ScanStatus::Fail;
            }
        }

        // Check secrets (non-baseline)
        let active_secrets: Vec<_> = result.secrets.iter().filter(|s| !s.in_baseline).collect();
        if !active_secrets.is_empty() {
            return ScanStatus::Fail;
        }

        // Check license violations
        if !result.license_violations.is_empty() {
            return ScanStatus::Fail;
        }

        // Check for warnings
        let has_warnings = result
            .vulnerabilities
            .iter()
            .any(|v| !self.is_cve_ignored(&v.id));

        if has_warnings {
            ScanStatus::Warn
        } else {
            ScanStatus::Pass
        }
    }
}

/// Secret detection patterns
pub mod patterns {
    /// Common secret patterns
    pub const PATTERNS: &[(&str, &str)] = &[
        ("AWS Access Key", r"AKIA[0-9A-Z]{16}"),
        (
            "AWS Secret Key",
            r#"(?i)aws(.{0,20})?['"][0-9a-zA-Z/+]{40}['"]"#,
        ),
        ("GitHub Token", r"gh[pousr]_[A-Za-z0-9_]{36,255}"),
        (
            "Generic API Key",
            r#"(?i)(api[_-]?key|apikey)['"]?\s*[:=]\s*['"][a-zA-Z0-9]{20,}['"]"#,
        ),
        (
            "Generic Secret",
            r#"(?i)(secret|password|passwd|pwd)['"]?\s*[:=]\s*['"][^'"]{8,}['"]"#,
        ),
        (
            "Private Key",
            r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----",
        ),
        (
            "Slack Token",
            r"xox[baprs]-[0-9]{10,13}-[0-9]{10,13}[a-zA-Z0-9-]*",
        ),
        ("Stripe Key", r"sk_live_[0-9a-zA-Z]{24,}"),
        ("Google API Key", r"AIza[0-9A-Za-z\-_]{35}"),
        ("npm Token", r"npm_[A-Za-z0-9]{36}"),
    ];

    /// Get all pattern regexes
    pub fn get_patterns() -> Vec<(&'static str, regex::Regex)> {
        PATTERNS
            .iter()
            .filter_map(|(name, pattern)| regex::Regex::new(pattern).ok().map(|re| (*name, re)))
            .collect()
    }
}

/// Generate security report
pub fn generate_report(result: &SecurityScanResult) -> String {
    let mut report = String::new();

    report.push_str("# Security Scan Report\n\n");
    report.push_str(&format!("Status: {:?}\n\n", result.status));

    // Vulnerabilities
    if !result.vulnerabilities.is_empty() {
        report.push_str("## Vulnerabilities\n\n");
        for vuln in &result.vulnerabilities {
            report.push_str(&format!(
                "- **{}** ({:?}): {} @ {}\n  {}\n",
                vuln.id, vuln.severity, vuln.package, vuln.version, vuln.description
            ));
            if let Some(fixed) = &vuln.fixed_version {
                report.push_str(&format!("  Fixed in: {}\n", fixed));
            }
            report.push('\n');
        }
    }

    // Secrets
    let active_secrets: Vec<_> = result.secrets.iter().filter(|s| !s.in_baseline).collect();
    if !active_secrets.is_empty() {
        report.push_str("## Secrets Detected\n\n");
        for secret in active_secrets {
            report.push_str(&format!(
                "- **{}**: {}:{}\n  Pattern: {}\n\n",
                secret.secret_type, secret.file, secret.line, secret.pattern
            ));
        }
    }

    // License violations
    if !result.license_violations.is_empty() {
        report.push_str("## License Violations\n\n");
        for violation in &result.license_violations {
            report.push_str(&format!(
                "- **{}** @ {}: {} ({})\n",
                violation.package, violation.version, violation.license, violation.reason
            ));
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_license_check() {
        let config = SecurityConfig {
            enabled: Some(true),
            allowed_licenses: vec!["MIT".to_string(), "Apache-2.0".to_string()],
            denied_licenses: vec!["GPL-3.0".to_string()],
            ..Default::default()
        };

        let scanner = SecurityScanner::new(config);
        assert!(scanner.is_license_allowed("MIT"));
        assert!(scanner.is_license_allowed("Apache-2.0"));
        assert!(!scanner.is_license_allowed("GPL-3.0"));
        assert!(!scanner.is_license_allowed("BSD-3-Clause")); // Not in allowed list
    }

    #[test]
    fn test_cve_ignore() {
        let config = SecurityConfig {
            enabled: Some(true),
            audit: Some(crate::SecurityAuditConfig {
                enabled: Some(true),
                ignore: vec!["CVE-2021-12345".to_string()],
                ..Default::default()
            }),
            ..Default::default()
        };

        let scanner = SecurityScanner::new(config);
        assert!(scanner.is_cve_ignored("CVE-2021-12345"));
        assert!(!scanner.is_cve_ignored("CVE-2021-99999"));
    }
}
