//! Terminal detection and adaptation.
//!
//! This module provides cross-platform terminal detection, including
//! capability detection and CI environment recognition.

use std::env;

/// Terminal capabilities.
#[derive(Debug, Clone, Default)]
pub struct TermCapabilities {
    /// Whether the terminal supports colors.
    pub color: bool,
    /// Whether the terminal supports Unicode.
    pub unicode: bool,
    /// Whether the terminal is interactive (TTY).
    pub interactive: bool,
    /// Whether hyperlinks are supported.
    pub hyperlinks: bool,
    /// Terminal width.
    pub width: Option<u16>,
    /// Terminal height.
    pub height: Option<u16>,
}

/// CI environment types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiEnvironment {
    /// GitHub Actions.
    GitHubActions,
    /// GitLab CI.
    GitLabCi,
    /// Jenkins.
    Jenkins,
    /// Azure Pipelines.
    AzurePipelines,
    /// CircleCI.
    CircleCi,
    /// Travis CI.
    TravisCi,
    /// Bitbucket Pipelines.
    BitbucketPipelines,
    /// TeamCity.
    TeamCity,
    /// Buildkite.
    Buildkite,
    /// Generic CI (CI env var is set).
    Generic,
}

impl CiEnvironment {
    /// Detect the CI environment.
    pub fn detect() -> Option<Self> {
        if env::var("GITHUB_ACTIONS").is_ok() {
            return Some(CiEnvironment::GitHubActions);
        }
        if env::var("GITLAB_CI").is_ok() {
            return Some(CiEnvironment::GitLabCi);
        }
        if env::var("JENKINS_URL").is_ok() {
            return Some(CiEnvironment::Jenkins);
        }
        if env::var("TF_BUILD").is_ok() {
            return Some(CiEnvironment::AzurePipelines);
        }
        if env::var("CIRCLECI").is_ok() {
            return Some(CiEnvironment::CircleCi);
        }
        if env::var("TRAVIS").is_ok() {
            return Some(CiEnvironment::TravisCi);
        }
        if env::var("BITBUCKET_BUILD_NUMBER").is_ok() {
            return Some(CiEnvironment::BitbucketPipelines);
        }
        if env::var("TEAMCITY_VERSION").is_ok() {
            return Some(CiEnvironment::TeamCity);
        }
        if env::var("BUILDKITE").is_ok() {
            return Some(CiEnvironment::Buildkite);
        }
        if env::var("CI").is_ok() {
            return Some(CiEnvironment::Generic);
        }
        None
    }

    /// Check if this CI environment supports ANSI colors.
    pub fn supports_color(&self) -> bool {
        matches!(
            self,
            CiEnvironment::GitHubActions
                | CiEnvironment::GitLabCi
                | CiEnvironment::AzurePipelines
                | CiEnvironment::CircleCi
                | CiEnvironment::TravisCi
                | CiEnvironment::Buildkite
        )
    }

    /// Get the CI environment name.
    pub fn name(&self) -> &'static str {
        match self {
            CiEnvironment::GitHubActions => "GitHub Actions",
            CiEnvironment::GitLabCi => "GitLab CI",
            CiEnvironment::Jenkins => "Jenkins",
            CiEnvironment::AzurePipelines => "Azure Pipelines",
            CiEnvironment::CircleCi => "CircleCI",
            CiEnvironment::TravisCi => "Travis CI",
            CiEnvironment::BitbucketPipelines => "Bitbucket Pipelines",
            CiEnvironment::TeamCity => "TeamCity",
            CiEnvironment::Buildkite => "Buildkite",
            CiEnvironment::Generic => "CI",
        }
    }
}

/// Terminal type detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalType {
    /// Windows Terminal (modern).
    WindowsTerminal,
    /// ConEmu/Cmder.
    ConEmu,
    /// Windows Console (cmd.exe or PowerShell without Windows Terminal).
    WindowsConsole,
    /// iTerm2.
    ITerm2,
    /// VS Code integrated terminal.
    VSCode,
    /// WezTerm.
    WezTerm,
    /// Hyper.
    Hyper,
    /// Alacritty.
    Alacritty,
    /// Kitty.
    Kitty,
    /// Generic Unix terminal.
    Unix,
    /// Unknown terminal.
    Unknown,
}

impl TerminalType {
    /// Detect the terminal type.
    pub fn detect() -> Self {
        // Windows Terminal
        if env::var("WT_SESSION").is_ok() {
            return TerminalType::WindowsTerminal;
        }

        // ConEmu/Cmder
        if env::var("ConEmuANSI").is_ok() || env::var("CMDER_ROOT").is_ok() {
            return TerminalType::ConEmu;
        }

        // VS Code
        if env::var("TERM_PROGRAM")
            .map(|v| v == "vscode")
            .unwrap_or(false)
        {
            return TerminalType::VSCode;
        }

        // iTerm2
        if env::var("TERM_PROGRAM")
            .map(|v| v == "iTerm.app")
            .unwrap_or(false)
        {
            return TerminalType::ITerm2;
        }

        // WezTerm
        if env::var("TERM_PROGRAM")
            .map(|v| v == "WezTerm")
            .unwrap_or(false)
        {
            return TerminalType::WezTerm;
        }

        // Hyper
        if env::var("TERM_PROGRAM")
            .map(|v| v == "Hyper")
            .unwrap_or(false)
        {
            return TerminalType::Hyper;
        }

        // Alacritty
        if env::var("TERM")
            .map(|v| v.contains("alacritty"))
            .unwrap_or(false)
        {
            return TerminalType::Alacritty;
        }

        // Kitty
        if env::var("KITTY_WINDOW_ID").is_ok() {
            return TerminalType::Kitty;
        }

        // Windows Console (fallback for Windows)
        #[cfg(windows)]
        {
            TerminalType::WindowsConsole
        }

        // Unix (fallback for Unix-like systems)
        #[cfg(unix)]
        {
            TerminalType::Unix
        }

        #[cfg(not(any(unix, windows)))]
        {
            TerminalType::Unknown
        }
    }

    /// Check if this terminal supports hyperlinks.
    pub fn supports_hyperlinks(&self) -> bool {
        matches!(
            self,
            TerminalType::WindowsTerminal
                | TerminalType::ITerm2
                | TerminalType::VSCode
                | TerminalType::WezTerm
                | TerminalType::Hyper
                | TerminalType::Kitty
        )
    }

    /// Check if this terminal supports Unicode well.
    pub fn supports_unicode(&self) -> bool {
        !matches!(self, TerminalType::WindowsConsole | TerminalType::Unknown)
    }
}

/// Terminal information and utilities.
#[derive(Debug, Clone)]
pub struct Term {
    capabilities: TermCapabilities,
    ci_environment: Option<CiEnvironment>,
    terminal_type: TerminalType,
}

impl Default for Term {
    fn default() -> Self {
        Self::detect()
    }
}

impl Term {
    /// Detect terminal capabilities.
    pub fn detect() -> Self {
        let ci_environment = CiEnvironment::detect();
        let terminal_type = TerminalType::detect();
        let is_tty = Self::detect_tty();
        let supports_color = Self::detect_color_support(is_tty, ci_environment);
        let supports_unicode = Self::detect_unicode_support(&terminal_type);
        let (width, height) = Self::detect_size();

        Self {
            capabilities: TermCapabilities {
                color: supports_color,
                unicode: supports_unicode,
                interactive: is_tty && ci_environment.is_none(),
                hyperlinks: terminal_type.supports_hyperlinks(),
                width,
                height,
            },
            ci_environment,
            terminal_type,
        }
    }

    /// Create a term with minimal capabilities (for testing).
    pub fn minimal() -> Self {
        Self {
            capabilities: TermCapabilities::default(),
            ci_environment: None,
            terminal_type: TerminalType::Unknown,
        }
    }

    /// Check if the terminal is a TTY.
    fn detect_tty() -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            unsafe { libc::isatty(std::io::stderr().as_raw_fd()) != 0 }
        }

        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Console::{
                GetConsoleMode, GetStdHandle, STD_ERROR_HANDLE,
            };
            unsafe {
                let handle = GetStdHandle(STD_ERROR_HANDLE);
                let mut mode = 0;
                GetConsoleMode(handle, &mut mode) != 0
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            false
        }
    }

    /// Enable ANSI support on Windows.
    #[cfg(windows)]
    pub fn enable_ansi_support() -> bool {
        use windows_sys::Win32::System::Console::{
            GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
            STD_OUTPUT_HANDLE,
        };

        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut mode = 0;
            if GetConsoleMode(handle, &mut mode) == 0 {
                return false;
            }
            SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0
        }
    }

    /// Enable ANSI support (no-op on non-Windows).
    #[cfg(not(windows))]
    pub fn enable_ansi_support() -> bool {
        true
    }

    /// Detect color support.
    fn detect_color_support(is_tty: bool, ci: Option<CiEnvironment>) -> bool {
        // NO_COLOR takes precedence
        if env::var("NO_COLOR").is_ok() {
            return false;
        }

        // FORCE_COLOR forces colors
        if env::var("FORCE_COLOR").is_ok() {
            return true;
        }

        // TERM=dumb means no colors
        if env::var("TERM").map(|t| t == "dumb").unwrap_or(false) {
            return false;
        }

        // CI environments may support colors
        if let Some(ci_env) = ci {
            return ci_env.supports_color();
        }

        // TTY usually supports colors
        if is_tty {
            return true;
        }

        false
    }

    /// Detect Unicode support.
    fn detect_unicode_support(terminal_type: &TerminalType) -> bool {
        // Check terminal type first
        if !terminal_type.supports_unicode() {
            // Even Windows Console can support Unicode with proper codepage
            #[cfg(windows)]
            {
                // Check if we're using UTF-8 codepage
                if env::var("LANG")
                    .map(|v| v.to_lowercase().contains("utf"))
                    .unwrap_or(false)
                {
                    return true;
                }
            }
            return false;
        }

        // Check LANG environment variable
        if let Ok(lang) = env::var("LANG") {
            if lang.to_lowercase().contains("utf") {
                return true;
            }
        }

        // Check LC_ALL
        if let Ok(lc_all) = env::var("LC_ALL") {
            if lc_all.to_lowercase().contains("utf") {
                return true;
            }
        }

        // Default to true on modern systems
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            true
        }

        #[cfg(windows)]
        {
            // Windows Terminal and modern terminals support Unicode
            matches!(
                terminal_type,
                TerminalType::WindowsTerminal | TerminalType::ConEmu | TerminalType::VSCode
            )
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
        {
            false
        }
    }

    /// Detect terminal size.
    fn detect_size() -> (Option<u16>, Option<u16>) {
        terminal_size::terminal_size()
            .map(|(w, h)| (Some(w.0), Some(h.0)))
            .unwrap_or((None, None))
    }

    /// Get terminal capabilities.
    pub fn capabilities(&self) -> &TermCapabilities {
        &self.capabilities
    }

    /// Get the terminal type.
    pub fn terminal_type(&self) -> TerminalType {
        self.terminal_type
    }

    /// Check if colors are supported.
    pub fn supports_color(&self) -> bool {
        self.capabilities.color
    }

    /// Check if Unicode is supported.
    pub fn supports_unicode(&self) -> bool {
        self.capabilities.unicode
    }

    /// Check if the terminal is interactive.
    pub fn is_interactive(&self) -> bool {
        self.capabilities.interactive
    }

    /// Check if the terminal is a TTY.
    pub fn is_tty(&self) -> bool {
        Self::detect_tty()
    }

    /// Check if hyperlinks are supported.
    pub fn supports_hyperlinks(&self) -> bool {
        self.capabilities.hyperlinks
    }

    /// Get the terminal size.
    pub fn size(&self) -> Option<(u16, u16)> {
        match (self.capabilities.width, self.capabilities.height) {
            (Some(w), Some(h)) => Some((w, h)),
            _ => None,
        }
    }

    /// Get the terminal width.
    pub fn width(&self) -> Option<u16> {
        self.capabilities.width
    }

    /// Get the terminal height.
    pub fn height(&self) -> Option<u16> {
        self.capabilities.height
    }

    /// Get the CI environment, if any.
    pub fn ci_environment(&self) -> Option<CiEnvironment> {
        self.ci_environment
    }

    /// Check if running in a CI environment.
    pub fn is_ci(&self) -> bool {
        self.ci_environment.is_some()
    }

    /// Clear the screen.
    pub fn clear_screen(&self) {
        print!("\x1b[2J\x1b[H");
    }

    /// Clear the current line.
    pub fn clear_line(&self) {
        print!("\r\x1b[K");
    }

    /// Move cursor up by n lines.
    pub fn move_cursor_up(&self, n: u16) {
        print!("\x1b[{}A", n);
    }

    /// Move cursor down by n lines.
    pub fn move_cursor_down(&self, n: u16) {
        print!("\x1b[{}B", n);
    }

    /// Hide the cursor.
    pub fn hide_cursor(&self) {
        print!("\x1b[?25l");
    }

    /// Show the cursor.
    pub fn show_cursor(&self) {
        print!("\x1b[?25h");
    }

    /// Create a hyperlink.
    pub fn hyperlink(&self, url: &str, text: &str) -> String {
        if self.supports_hyperlinks() {
            format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
        } else {
            text.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_environment_detect() {
        // This test depends on the actual environment
        let _ = CiEnvironment::detect();
    }

    #[test]
    fn test_ci_environment_name() {
        assert_eq!(CiEnvironment::GitHubActions.name(), "GitHub Actions");
        assert_eq!(CiEnvironment::GitLabCi.name(), "GitLab CI");
    }

    #[test]
    fn test_terminal_type_detect() {
        let _ = TerminalType::detect();
    }

    #[test]
    fn test_term_detect() {
        let term = Term::detect();
        // Just verify it doesn't panic
        let _ = term.supports_color();
        let _ = term.supports_unicode();
        let _ = term.is_interactive();
        let _ = term.terminal_type();
    }

    #[test]
    fn test_term_minimal() {
        let term = Term::minimal();
        assert!(!term.supports_color());
        assert!(!term.supports_unicode());
        assert!(!term.is_interactive());
        assert_eq!(term.terminal_type(), TerminalType::Unknown);
    }

    #[test]
    fn test_hyperlink_no_support() {
        let term = Term::minimal();
        let result = term.hyperlink("https://example.com", "Example");
        assert_eq!(result, "Example");
    }

    #[test]
    fn test_enable_ansi_support() {
        // Just verify it doesn't panic
        let _ = Term::enable_ansi_support();
    }
}
