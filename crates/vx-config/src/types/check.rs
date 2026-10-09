//! `vx check` configuration

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// How loudly `vx check` reports a declared pin that the effective toolchain
/// disagrees with.
///
/// The default is `Warn`: a declared-but-unhonoured numeric pin is reported, but it
/// does not fail the command. Flipping the default to `Error` would turn every
/// already-drifting repository red on upgrade, which is a breaking change to a
/// published CLI contract. Repositories that want the pin enforced opt in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ToolchainPinMismatch {
    /// Report the mismatch as a warning; `vx check` still exits 0.
    #[default]
    Warn,
    /// Report the mismatch as an error; `vx check` exits non-zero.
    Error,
    /// Do not report the mismatch at all.
    Ignore,
}

impl ToolchainPinMismatch {
    /// True when a mismatch must fail `vx check`.
    pub fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }

    /// True when a mismatch should be suppressed entirely.
    pub fn is_ignored(self) -> bool {
        matches!(self, Self::Ignore)
    }
}

/// Configuration for the `vx check` command.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct CheckConfig {
    /// Severity for a numeric toolchain pin that the effective toolchain does not
    /// match (`"warn"` by default, `"error"` to fail, `"ignore"` to stay silent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toolchain_pin_mismatch: Option<ToolchainPinMismatch>,
}

impl CheckConfig {
    /// Severity for a toolchain pin mismatch, defaulting to `warn`.
    pub fn pin_mismatch_severity(&self) -> ToolchainPinMismatch {
        self.toolchain_pin_mismatch.unwrap_or_default()
    }
}
