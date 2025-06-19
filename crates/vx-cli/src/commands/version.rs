//! Version command implementation

use crate::ui::UI;
use anyhow::Result;

pub async fn handle() -> Result<()> {
    UI::info(&format!("vx {}", env!("CARGO_PKG_VERSION")));
    UI::info("Universal Development Tool Manager");
    Ok(())
}
