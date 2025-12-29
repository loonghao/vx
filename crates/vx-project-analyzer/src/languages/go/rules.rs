//! Go script detection rules
//!
//! Defines rules for detecting common Go scripts based on file presence.

use crate::languages::rules::ScriptRule;

/// All Go script detection rules
///
/// Rules are evaluated by priority (highest first).
/// For each script name, only the highest priority matching rule is used.
pub const GO_RULES: &[ScriptRule] = &[
    // =========================================================================
    // Build
    // =========================================================================
    ScriptRule::new("build", "go build ./...", "Build the project")
        .triggers(&["go.mod"])
        .priority(50),
    // =========================================================================
    // Test
    // =========================================================================
    ScriptRule::new("test", "go test ./...", "Run tests")
        .triggers(&["go.mod"])
        .priority(50),
    ScriptRule::new(
        "test-verbose",
        "go test -v ./...",
        "Run tests with verbose output",
    )
    .triggers(&["go.mod"])
    .priority(40),
    // =========================================================================
    // Linting & Formatting
    // =========================================================================
    ScriptRule::new("lint", "golangci-lint run", "Run golangci-lint")
        .triggers(&[".golangci.yml", ".golangci.yaml", ".golangci.toml"])
        .priority(100),
    ScriptRule::new("lint", "go vet ./...", "Run go vet")
        .triggers(&["go.mod"])
        .excludes(&[".golangci.yml", ".golangci.yaml", ".golangci.toml"])
        .priority(50),
    ScriptRule::new("format", "gofmt -w .", "Format code")
        .triggers(&["go.mod"])
        .priority(50),
    ScriptRule::new("format", "goimports -w .", "Format and organize imports")
        .triggers(&["go.mod"])
        .priority(40),
    // =========================================================================
    // Mod management
    // =========================================================================
    ScriptRule::new("tidy", "go mod tidy", "Tidy dependencies")
        .triggers(&["go.mod"])
        .priority(50),
    ScriptRule::new("download", "go mod download", "Download dependencies")
        .triggers(&["go.mod"])
        .priority(50),
    // =========================================================================
    // Run
    // =========================================================================
    ScriptRule::new("run", "go run .", "Run the main package")
        .triggers(&["main.go"])
        .priority(50),
    ScriptRule::new("run", "go run ./cmd/...", "Run cmd package")
        .triggers(&["cmd"])
        .excludes(&["main.go"])
        .priority(40),
    // =========================================================================
    // Generate
    // =========================================================================
    ScriptRule::new("generate", "go generate ./...", "Run go generate")
        .triggers(&["go.mod"])
        .priority(50),
];
