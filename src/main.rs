use anyhow::Result;
use clap::Parser;
use vx::cli::{commands::CommandHandler, Cli};
use vx::tracing_setup::init_tracing;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing with progress bars - community best practice
    init_tracing(cli.verbose);

    CommandHandler::handle(cli).await
}
