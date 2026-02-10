//! x-cmd configuration and URL builder

use vx_runtime::Platform;

/// Configuration for x-cmd
pub struct XCmdConfig;

impl XCmdConfig {
    /// Get the executable name
    pub fn executable_name() -> &'static str {
        "x"
    }

    /// Get the install script URL
    pub fn install_script_url() -> &'static str {
        "https://get.x-cmd.com"
    }

    /// Get the Windows install script URL
    pub fn install_script_url_windows() -> &'static str {
        "https://get.x-cmd.com/ps1"
    }

    /// Get search paths for the given platform
    pub fn search_paths(platform: &Platform) -> &'static [&'static str] {
        match platform.os {
            vx_runtime::Os::Windows => &[],
            vx_runtime::Os::MacOS | vx_runtime::Os::Linux => {
                &["/usr/local/bin"]
            }
            _ => &[],
        }
    }

    /// Get install command for the given platform
    pub fn install_command(platform: &Platform) -> &'static str {
        match platform.os {
            vx_runtime::Os::Windows => {
                "iex (irm https://get.x-cmd.com/ps1)"
            }
            _ => {
                "eval \"$(curl -fsSL https://get.x-cmd.com)\""
            }
        }
    }
}
