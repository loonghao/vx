//! VX - Universal Development Tool Manager
//!
//! Main binary entry point that delegates to the CLI implementation.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vx_cli::main().await
}
