//! Shell-specific script generators
//!
//! This module provides platform-specific script generation for different shells.

pub mod bash;
pub mod cmd;
pub mod powershell;

/// Shell type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    /// Bash shell (Linux/macOS default)
    Bash,
    /// PowerShell (Windows default, also available on Linux/macOS)
    PowerShell,
    /// Windows Command Prompt
    Cmd,
    /// POSIX sh (fallback)
    Sh,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
}

impl Shell {
    /// Detect the current shell from environment
    pub fn detect() -> Self {
        if cfg!(windows) {
            // On Windows, prefer PowerShell
            Shell::PowerShell
        } else {
            // On Unix, check SHELL environment variable
            if let Ok(shell) = std::env::var("SHELL") {
                if shell.contains("zsh") {
                    Shell::Zsh
                } else if shell.contains("fish") {
                    Shell::Fish
                } else if shell.contains("bash") {
                    Shell::Bash
                } else {
                    Shell::Sh
                }
            } else {
                Shell::Bash
            }
        }
    }

    /// Get the script file extension for this shell
    pub fn extension(&self) -> &'static str {
        match self {
            Shell::Bash | Shell::Sh | Shell::Zsh => "sh",
            Shell::PowerShell => "ps1",
            Shell::Cmd => "bat",
            Shell::Fish => "fish",
        }
    }

    /// Get the shell executable name
    pub fn executable(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Sh => "sh",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "pwsh",
            Shell::Cmd => "cmd",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_extension() {
        assert_eq!(Shell::Bash.extension(), "sh");
        assert_eq!(Shell::PowerShell.extension(), "ps1");
        assert_eq!(Shell::Cmd.extension(), "bat");
    }

    #[test]
    fn test_shell_executable() {
        assert_eq!(Shell::Bash.executable(), "bash");
        assert_eq!(Shell::PowerShell.executable(), "pwsh");
    }
}
