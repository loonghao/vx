//! Version command implementation

use vx_core::Result;
use crate::ui::UI;

pub async fn handle() -> Result<()> {
    UI::info(&format!("vx {}", env!("CARGO_PKG_VERSION")));
    UI::info("Universal Development Tool Manager");
    Ok(())
}
