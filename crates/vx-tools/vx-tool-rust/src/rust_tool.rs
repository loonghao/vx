//! Rust toolchain implementations with environment isolation

use std::collections::HashMap;
use vx_core::{
    GitHubVersionParser, HttpUtils, Result, RustUrlBuilder, ToolContext, ToolExecutionResult,
    UrlBuilder, VersionInfo, VxTool,
};

/// Macro to generate Rust tool implementations using VxTool trait
macro_rules! rust_vx_tool {
    ($name:ident, $cmd:literal, $desc:literal) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            url_builder: RustUrlBuilder,
            version_parser: GitHubVersionParser,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    url_builder: RustUrlBuilder::new(),
                    version_parser: GitHubVersionParser::new("rust-lang", "rust"),
                }
            }
        }

        #[async_trait::async_trait]
        impl VxTool for $name {
            fn name(&self) -> &str {
                $cmd
            }

            fn description(&self) -> &str {
                $desc
            }

            fn aliases(&self) -> Vec<&str> {
                vec![]
            }

            async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
                // For Rust tools, fetch from GitHub releases
                let json = HttpUtils::fetch_json(RustUrlBuilder::versions_url()).await?;
                GitHubVersionParser::parse_versions(&json, include_prerelease)
            }

            async fn install_version(&self, version: &str, force: bool) -> Result<()> {
                if !force && self.is_version_installed(version).await? {
                    return Err(vx_core::VxError::VersionAlreadyInstalled {
                        tool_name: self.name().to_string(),
                        version: version.to_string(),
                    });
                }

                let install_dir = self.get_version_install_dir(version);
                let _exe_path = self.default_install_workflow(version, &install_dir).await?;

                // Verify installation
                if !self.is_version_installed(version).await? {
                    return Err(vx_core::VxError::InstallationFailed {
                        tool_name: self.name().to_string(),
                        version: version.to_string(),
                        message: "Installation verification failed".to_string(),
                    });
                }

                Ok(())
            }

            async fn execute(
                &self,
                args: &[String],
                context: &ToolContext,
            ) -> Result<ToolExecutionResult> {
                self.default_execute_workflow(args, context).await
            }

            async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
                let rust_url_builder = RustUrlBuilder::new();
                Ok(rust_url_builder.download_url(version))
            }

            fn metadata(&self) -> HashMap<String, String> {
                let mut meta = HashMap::new();
                meta.insert(
                    "homepage".to_string(),
                    "https://www.rust-lang.org/".to_string(),
                );
                meta.insert("ecosystem".to_string(), "rust".to_string());
                meta.insert(
                    "repository".to_string(),
                    "https://github.com/rust-lang/rust".to_string(),
                );
                meta.insert("license".to_string(), "MIT OR Apache-2.0".to_string());
                meta
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

// Define all Rust tools using the VxTool macro
rust_vx_tool!(CargoTool, "cargo", "Rust package manager and build tool");
rust_vx_tool!(RustcTool, "rustc", "The Rust compiler");
rust_vx_tool!(
    RustupTool,
    "rustup",
    "Rust toolchain installer and version manager"
);
rust_vx_tool!(RustdocTool, "rustdoc", "Rust documentation generator");
rust_vx_tool!(RustfmtTool, "rustfmt", "Rust code formatter");
rust_vx_tool!(
    ClippyTool,
    "clippy",
    "Rust linter for catching common mistakes"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_tool_creation() {
        let tool = CargoTool::new();
        assert_eq!(tool.name(), "cargo");
        assert!(!tool.description().is_empty());
        assert!(tool.aliases().is_empty());
    }

    #[test]
    fn test_rustc_tool_creation() {
        let tool = RustcTool::new();
        assert_eq!(tool.name(), "rustc");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_rust_tool_metadata() {
        let tool = CargoTool::new();
        let metadata = tool.metadata();

        assert!(metadata.contains_key("homepage"));
        assert!(metadata.contains_key("ecosystem"));
        assert_eq!(metadata.get("ecosystem"), Some(&"rust".to_string()));
    }
}
