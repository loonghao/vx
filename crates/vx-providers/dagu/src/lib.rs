//! Dagu Workflow Engine Provider for vx
//!
//! This provider adds support for Dagu - a self-contained, powerful workflow engine
//! with a built-in Web UI, designed for defining and running DAGs (Directed Acyclic Graphs).
//!
//! ## Features
//!
//! - Version management via GitHub releases
//! - Cross-platform support (Windows, macOS, Linux)
//! - Automatic installation and verification
//! - Built-in Web UI for workflow management
//! - YAML-based workflow definition with cron scheduling
//! - Can invoke `vx python`, `vx node`, etc. in workflow steps
//!
//! ## Usage with vx
//!
//! ```bash
//! # Install dagu
//! vx install dagu
//!
//! # Start the dagu server (Web UI at http://localhost:8080)
//! vx dagu server
//!
//! # Run a workflow
//! vx dagu start my-workflow
//!
//! # Check workflow status
//! vx dagu status my-workflow
//! ```
//!
//! ## Integration with vx environment
//!
//! Dagu workflows can leverage the full vx environment by using `vx` as the command prefix:
//!
//! ```yaml
//! # my-workflow.yaml
//! schedule: "0 9 * * *"
//! env:
//!   - VX_HOME: "~/.vx"
//! steps:
//!   - name: "Run Python script"
//!     command: "vx python script.py"
//!   - name: "Run Node.js script"
//!     command: "vx node app.js"
//!     depends:
//!       - "Run Python script"
//! ```

use std::sync::Arc;

pub mod config;
pub mod provider;
pub mod runtime;

pub use config::DaguUrlBuilder;
pub use provider::DaguProvider;
pub use runtime::DaguRuntime;

/// Factory function to create the Dagu provider
pub fn create_provider() -> Arc<dyn vx_runtime::Provider> {
    Arc::new(provider::DaguProvider::new())
}
