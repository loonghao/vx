//! rtk-style output filtering for vx subprocess output.
//!
//! Only activates when `VX_OUTPUT=compact` AND stdout is NOT a TTY.
//! Default behavior (TTY or no `VX_OUTPUT=compact`) is completely unchanged.

pub mod filter;
pub mod rules;
pub mod stream;

pub use filter::{OutputFilter, OutputFilterConfig};
pub use rules::FilterRules;
