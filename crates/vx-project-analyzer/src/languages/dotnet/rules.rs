//! .NET/C# script detection rules
//!
//! Defines rules for detecting common .NET scripts based on file presence.
//!
//! Note: Unlike other analyzers, .NET projects use variable-named files (.csproj, .sln)
//! so we use a simpler approach - the DotNetAnalyzer generates scripts directly
//! based on detect() results rather than relying on trigger-file-based rules.

use crate::languages::rules::ScriptRule;

/// All .NET/C# script detection rules
///
/// Note: .NET projects use files with variable names (*.csproj, *.sln),
/// so we use "global.json" and "Directory.Build.props" as proxy triggers
/// since they have fixed names. The DotNetAnalyzer also generates scripts
/// directly for projects detected by extension scanning.
pub const DOTNET_RULES: &[ScriptRule] = &[
    // =========================================================================
    // Build (triggered by global.json or Directory.Build.props)
    // =========================================================================
    ScriptRule::new("build", "dotnet build", "Build the .NET project")
        .triggers(&["global.json", "Directory.Build.props"])
        .priority(50),
    // =========================================================================
    // Test
    // =========================================================================
    ScriptRule::new("test", "dotnet test", "Run .NET tests")
        .triggers(&["global.json", "Directory.Build.props"])
        .priority(50),
    // =========================================================================
    // Restore
    // =========================================================================
    ScriptRule::new("restore", "dotnet restore", "Restore NuGet packages")
        .triggers(&["global.json", "Directory.Build.props"])
        .priority(50),
    // =========================================================================
    // Format
    // =========================================================================
    ScriptRule::new("format", "dotnet format", "Format code")
        .triggers(&["global.json", "Directory.Build.props"])
        .priority(50),
    // =========================================================================
    // Clean
    // =========================================================================
    ScriptRule::new("clean", "dotnet clean", "Clean build output")
        .triggers(&["global.json", "Directory.Build.props"])
        .priority(50),
];
