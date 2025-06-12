//! Version command implementation

use crate::ui::UI;
use vx_core::Result;

pub async fn handle() -> Result<()> {
    UI::info(&format!("vx {}", env!("CARGO_PKG_VERSION")));
    UI::info("Universal Development Tool Manager");
    Ok(())
}
