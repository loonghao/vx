//! Setup pipeline execution engine
//!
//! This module provides the setup pipeline functionality for `vx setup`.

mod executor;
mod hooks;

pub use executor::{SetupHookResult, SetupPipeline, SetupPipelineResult};
