//! VX - Universal Development Tool Manager
//!
//! Main binary entry point that delegates to the CLI implementation.

#[tokio::main]
async fn main() {
    match vx_cli::main().await {
        Ok(()) => {}
        Err(err) => {
            // Try structured error formatting for PipelineError
            if !vx_cli::error_handler::try_handle_error(&err) {
                // Fallback: format generic errors with basic styling
                vx_cli::error_handler::format_generic_error(&err);
                std::process::exit(1);
            }
        }
    }
}
