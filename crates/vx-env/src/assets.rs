//! Embedded shell initialization scripts
//!
//! This module uses `rust-embed` to embed shell init scripts at compile time,
//! making them easier to maintain than hardcoded strings.

use rust_embed::Embed;

/// Embedded shell initialization assets
#[derive(Embed)]
#[folder = "assets/shell/"]
pub struct ShellAssets;

/// Shell script types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellScript {
    PowerShell,
    Bash,
    Zsh,
    Cmd,
}

impl ShellScript {
    /// Get the asset filename for this shell type
    fn filename(&self) -> &'static str {
        match self {
            Self::PowerShell => "powershell_init.ps1",
            Self::Bash => "bash_init.sh",
            Self::Zsh => "zsh_init.zsh",
            Self::Cmd => "cmd_init.bat",
        }
    }

    /// Get the raw script content from embedded assets
    pub fn get_raw(&self) -> Option<String> {
        ShellAssets::get(self.filename()).map(|f| String::from_utf8_lossy(&f.data).into_owned())
    }

    /// Get the script with project-specific substitutions
    pub fn render(&self, project_name: &str, tools: &str) -> Option<String> {
        let raw = self.get_raw()?;

        match self {
            Self::PowerShell => {
                // PowerShell uses parameters, we'll create a wrapper that sets them
                Some(render_powershell_script(&raw, project_name, tools))
            }
            Self::Bash | Self::Zsh => {
                // Bash/Zsh use environment variables, set them inline
                Some(render_unix_script(&raw, project_name, tools))
            }
            Self::Cmd => {
                // CMD uses environment variables
                Some(render_cmd_script(&raw, project_name, tools))
            }
        }
    }
}

/// Render PowerShell init script with project-specific values
fn render_powershell_script(template: &str, project_name: &str, tools: &str) -> String {
    // For PowerShell, we create a script that sets the values and then runs the main script
    format!(
        r#"# Auto-generated VX PowerShell init script
$ProjectName = "{project_name}"
$Tools = "{tools}"

{template}
"#
    )
}

/// Render Bash/Zsh init script with project-specific values
fn render_unix_script(template: &str, project_name: &str, tools: &str) -> String {
    format!(
        r#"# Auto-generated VX shell init script
VX_PROJECT_NAME="{project_name}"
VX_TOOLS="{tools}"

{template}
"#
    )
}

/// Render CMD init script with project-specific values
fn render_cmd_script(template: &str, project_name: &str, tools: &str) -> String {
    format!(
        r#"@echo off
REM Auto-generated VX CMD init script
set VX_PROJECT_NAME={project_name}
set VX_TOOLS={tools}

{template}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powershell_script_exists() {
        assert!(ShellScript::PowerShell.get_raw().is_some());
    }

    #[test]
    fn test_bash_script_exists() {
        assert!(ShellScript::Bash.get_raw().is_some());
    }

    #[test]
    fn test_zsh_script_exists() {
        assert!(ShellScript::Zsh.get_raw().is_some());
    }

    #[test]
    fn test_cmd_script_exists() {
        assert!(ShellScript::Cmd.get_raw().is_some());
    }

    #[test]
    fn test_render_powershell() {
        let script = ShellScript::PowerShell
            .render("my-project", "node@20, go@1.21")
            .unwrap();
        assert!(script.contains("my-project"));
        assert!(script.contains("node@20, go@1.21"));
    }

    #[test]
    fn test_render_bash() {
        let script = ShellScript::Bash
            .render("my-project", "node@20, go@1.21")
            .unwrap();
        assert!(script.contains("my-project"));
        assert!(script.contains("node@20, go@1.21"));
    }
}
