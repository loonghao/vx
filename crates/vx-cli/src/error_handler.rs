//! Structured error formatting for CLI output (RFC 0029 Phase 3.3)
//!
//! This module intercepts `PipelineError` (and its sub-errors) from `vx-resolver`
//! and formats them with colors, context, and fix suggestions for a better user experience.
//!
//! Instead of displaying raw `Error: resolve: runtime not found: xyz`, users see:
//!
//! ```text
//! ✗ error[resolve]: runtime not found: xyz
//!
//!   💡 Use 'vx list' to see all supported runtimes
//! ```

use colored::*;
use vx_resolver::{EnsureError, ExecuteError, PipelineError, PrepareError, ResolveError};

/// Format and display a pipeline error with structured output.
///
/// Returns the appropriate exit code for the error type.
pub fn handle_pipeline_error(err: &PipelineError) -> i32 {
    match err {
        PipelineError::Resolve(e) => {
            print_error_header("resolve");
            format_resolve_error(e);
        }
        PipelineError::Ensure(e) => {
            print_error_header("install");
            format_ensure_error(e);
        }
        PipelineError::Prepare(e) => {
            print_error_header("prepare");
            format_prepare_error(e);
        }
        PipelineError::Execute(e) => {
            print_error_header("execute");
            format_execute_error(e);
        }
        PipelineError::PlatformUnsupported { reasons } => {
            print_error_header("platform");
            eprintln!("  {}", "Platform not supported for this runtime:".red());
            for reason in reasons {
                eprintln!("    {} {}", "•".dimmed(), reason);
            }
            eprintln!();
            print_hint("This runtime is not available for your current platform.");
        }
        PipelineError::IncompatibleDependencies { details } => {
            print_error_header("dependencies");
            eprintln!("  {}", details.red());
        }
        PipelineError::PlatformCheckFailed { runtime, reason } => {
            print_error_header("platform");
            eprintln!(
                "  Platform check failed for {}: {}",
                runtime.cyan().bold(),
                reason
            );
        }
        PipelineError::Offline(msg) => {
            print_error_header("network");
            eprintln!("  {}", msg);
            eprintln!();
            print_hint("Check your internet connection and try again.");
            print_hint("Use 'vx --offline' to work with locally installed runtimes only.");
        }
    }

    1
}

/// Try to downcast an anyhow::Error to PipelineError and format it.
///
/// Returns `true` if the error was a PipelineError and was handled,
/// `false` otherwise (caller should use default formatting).
pub fn try_handle_error(err: &anyhow::Error) -> bool {
    if let Some(pipeline_err) = err.downcast_ref::<PipelineError>() {
        let code = handle_pipeline_error(pipeline_err);
        std::process::exit(code);
    }

    // Also try the individual error types (in case they weren't wrapped in PipelineError)
    if let Some(e) = err.downcast_ref::<ResolveError>() {
        print_error_header("resolve");
        format_resolve_error(e);
        std::process::exit(1);
    }
    if let Some(e) = err.downcast_ref::<EnsureError>() {
        print_error_header("install");
        format_ensure_error(e);
        std::process::exit(1);
    }
    if let Some(e) = err.downcast_ref::<PrepareError>() {
        print_error_header("prepare");
        format_prepare_error(e);
        std::process::exit(1);
    }
    if let Some(e) = err.downcast_ref::<ExecuteError>() {
        print_error_header("execute");
        format_execute_error(e);
        std::process::exit(1);
    }

    false
}

/// Format a ResolveError with context and suggestions
fn format_resolve_error(err: &ResolveError) {
    match err {
        ResolveError::RuntimeNotFound { name } => {
            eprintln!("  Runtime '{}' not found", name.cyan().bold());
            eprintln!();
            print_hint(&format!(
                "Use '{}' to see all supported runtimes",
                "vx list".cyan()
            ));
            print_hint(&format!(
                "Use '{}' to search for '{}'",
                format!("vx list | grep {}", name).cyan(),
                name
            ));
        }
        ResolveError::VersionNotFound { runtime, version } => {
            eprintln!(
                "  Version '{}' not found for {}",
                version.yellow(),
                runtime.cyan().bold()
            );
            eprintln!();
            print_hint(&format!(
                "Use '{}' to see available versions",
                format!("vx list {}", runtime).cyan()
            ));
        }
        ResolveError::NoLockedVersion { runtime } => {
            eprintln!("  No locked version for '{}'", runtime.cyan().bold());
            eprintln!();
            print_fix(&format!("vx lock {}", runtime));
            print_hint("Or specify a version explicitly: vx <runtime>@<version>");
        }
        ResolveError::DependencyCycle { cycle } => {
            eprintln!("  Dependency cycle detected:");
            eprintln!();
            eprintln!("    {}", cycle.join(" → ").yellow());
            eprintln!();
            print_hint("Check your project configuration for circular dependencies.");
        }
        ResolveError::PlatformNotSupported {
            runtime,
            required,
            current,
        } => {
            eprintln!(
                "  {} requires platform {}, but current platform is {}",
                runtime.cyan().bold(),
                required.yellow(),
                current.yellow()
            );
        }
        ResolveError::ResolutionFailed { runtime, reason } => {
            eprintln!(
                "  Failed to resolve version for {}: {}",
                runtime.cyan().bold(),
                reason
            );
        }
        ResolveError::UnknownWithDependency { runtime, available } => {
            eprintln!(
                "  --with dependency '{}' is not a known runtime",
                runtime.cyan().bold()
            );
            eprintln!();
            eprintln!("  {} {}", "Available:".dimmed(), available);
        }
        ResolveError::Other(e) => {
            eprintln!("  {}", e);
        }
    }
}

/// Format an EnsureError with context and suggestions
fn format_ensure_error(err: &EnsureError) {
    match err {
        EnsureError::InstallFailed {
            runtime,
            version,
            reason,
        } => {
            eprintln!(
                "  Failed to install {}@{}",
                runtime.cyan().bold(),
                version.yellow()
            );
            eprintln!("  {}", reason.dimmed());
            eprintln!();
            print_fix(&format!("vx install {}@{}", runtime, version));
        }
        EnsureError::DependencyInstallFailed {
            runtime,
            dep,
            reason,
        } => {
            eprintln!(
                "  Dependency {} required by {} failed to install",
                dep.cyan().bold(),
                runtime.cyan()
            );
            eprintln!("  {}", reason.dimmed());
            eprintln!();
            print_fix(&format!("vx install {}", dep));
        }
        EnsureError::DownloadFailed {
            runtime,
            version,
            url,
            reason,
        } => {
            eprintln!(
                "  Download failed for {}@{}",
                runtime.cyan().bold(),
                version.yellow()
            );
            eprintln!("  URL: {}", url.dimmed());
            eprintln!("  {}", reason.dimmed());
            eprintln!();
            print_hint("Check your internet connection and try again.");
        }
        EnsureError::AutoInstallDisabled { runtime, version } => {
            eprintln!(
                "  {}@{} is not installed and auto-install is disabled",
                runtime.cyan().bold(),
                version.yellow()
            );
            eprintln!();
            print_fix(&format!("vx install {}@{}", runtime, version));
            print_hint("Or enable auto-install in your configuration.");
        }
        EnsureError::Timeout {
            runtime,
            version,
            seconds,
        } => {
            eprintln!(
                "  Installation of {}@{} timed out after {}s",
                runtime.cyan().bold(),
                version.yellow(),
                seconds
            );
            eprintln!();
            print_hint("Try again with a better network connection.");
        }
        EnsureError::PlatformNotSupported { runtime, reason } => {
            eprintln!(
                "  {} is not supported on this platform",
                runtime.cyan().bold()
            );
            eprintln!("  {}", reason.dimmed());
        }
        EnsureError::PostInstallVerificationFailed { runtime, path } => {
            eprintln!(
                "  {} installed but executable not found",
                runtime.cyan().bold()
            );
            eprintln!("  Expected at: {}", path.display().to_string().dimmed());
            eprintln!();
            print_hint("The installation may be corrupted. Try reinstalling:");
            print_fix(&format!("vx install {} --force", runtime));
        }
        EnsureError::NoVersionsFound { runtime } => {
            eprintln!("  No versions found for {}", runtime.cyan().bold());
            eprintln!();
            print_hint("Check your internet connection or try again later.");
        }
        EnsureError::CommandFailed { exit_code } => {
            if let Some(code) = exit_code {
                eprintln!(
                    "  Installation command failed with exit code {}",
                    code.to_string().red()
                );
            } else {
                eprintln!("  Installation command failed (process terminated)");
            }
        }
        EnsureError::NotInstalled { runtime, hint } => {
            eprintln!("  {} is not installed", runtime.cyan().bold());
            if !hint.is_empty() {
                eprintln!("  {}", hint.dimmed());
            }
        }
        EnsureError::Other(e) => {
            eprintln!("  {}", e);
        }
    }
}

/// Format a PrepareError with context and suggestions
fn format_prepare_error(err: &PrepareError) {
    match err {
        PrepareError::UnknownRuntime { runtime } => {
            eprintln!(
                "  Unknown runtime '{}', cannot auto-install",
                runtime.cyan().bold()
            );
            eprintln!();
            print_hint(&format!(
                "Use '{}' to see all supported runtimes",
                "vx list".cyan()
            ));
        }
        PrepareError::NoExecutable { runtime } => {
            eprintln!(
                "  No executable found for {} after installation",
                runtime.cyan().bold()
            );
            eprintln!();
            print_fix(&format!("vx install {} --force", runtime));
        }
        PrepareError::ExecutableNotFound { path } => {
            eprintln!(
                "  Executable not found at: {}",
                path.display().to_string().yellow()
            );
        }
        PrepareError::EnvironmentFailed { runtime, reason } => {
            eprintln!(
                "  Failed to prepare environment for {}",
                runtime.cyan().bold()
            );
            eprintln!("  {}", reason.dimmed());
        }
        PrepareError::ProxyNotAvailable {
            runtime,
            proxy,
            reason,
        } => {
            eprintln!(
                "  Proxy runtime {} for {} is not available",
                proxy.cyan().bold(),
                runtime.cyan()
            );
            eprintln!("  {}", reason.dimmed());
            eprintln!();
            print_fix(&format!("vx install {}", proxy));
        }
        PrepareError::DependencyRequired {
            runtime,
            dependency,
            reason: _,
        } => {
            eprintln!(
                "  {} requires {} which is not installed",
                runtime.cyan().bold(),
                dependency.cyan().bold()
            );
            eprintln!();
            print_fix(&format!("vx install {}", dependency));
            print_hint("Or enable auto-install to install dependencies automatically.");
        }
        PrepareError::ProxyRetryFailed {
            runtime,
            dependency,
            reason,
        } => {
            eprintln!(
                "  Failed to prepare {} after installing {}",
                runtime.cyan().bold(),
                dependency.cyan()
            );
            eprintln!("  {}", reason.dimmed());
        }
        PrepareError::Other(e) => {
            eprintln!("  {}", e);
        }
    }
}

/// Format an ExecuteError with context and suggestions
fn format_execute_error(err: &ExecuteError) {
    match err {
        ExecuteError::SpawnFailed { executable, reason } => {
            eprintln!(
                "  Failed to spawn: {}",
                executable.display().to_string().yellow()
            );
            eprintln!("  {}", reason.dimmed());
            eprintln!();
            print_hint("Check that the file exists and has execute permissions.");
        }
        ExecuteError::Timeout { seconds } => {
            eprintln!(
                "  Execution timed out after {}s",
                seconds.to_string().yellow()
            );
            eprintln!();
            print_hint("The process took too long to complete.");
        }
        ExecuteError::Killed => {
            eprintln!("  Process was killed by signal");
        }
        ExecuteError::BundleExecutionFailed { tool, reason } => {
            eprintln!("  Failed to execute bundled tool '{}'", tool.cyan().bold());
            eprintln!("  {}", reason.dimmed());
        }
        ExecuteError::Other(e) => {
            eprintln!("  {}", e);
        }
    }
}

/// Print a formatted error header
fn print_error_header(category: &str) {
    eprintln!(
        "\n{} {}",
        "✗".red().bold(),
        format!("error[{}]", category).red().bold()
    );
}

/// Print a hint line
fn print_hint(msg: &str) {
    eprintln!("  {} {}", "💡".cyan(), msg.dimmed());
}

/// Print a fix suggestion line
fn print_fix(cmd: &str) {
    eprintln!("  {} To fix, run: {}", "💡".cyan(), cmd.cyan().bold());
}

/// Format a generic anyhow error with basic styling
pub fn format_generic_error(err: &anyhow::Error) {
    eprintln!("\n{} {}", "✗".red().bold(), "error".red().bold());
    eprintln!("  {}", err);

    // Show error chain if available
    let chain: Vec<_> = err.chain().skip(1).collect();
    if !chain.is_empty() {
        eprintln!();
        eprintln!("  {}:", "Caused by".dimmed());
        for (i, cause) in chain.iter().enumerate() {
            eprintln!("  {}. {}", (i + 1).to_string().dimmed(), cause);
        }
    }
}
