//! Test shimexe-core API usage

use anyhow::Result;
use shimexe_core::ShimRunner;

fn main() -> Result<()> {
    println!("Testing shimexe-core API...");

    // Test ShimRunner with our created shim config
    let shim_config_path = r"C:\Users\hallo\.vx\tools\go\current\go.shim.toml";

    println!("Loading shim config from: {}", shim_config_path);

    match ShimRunner::from_file(shim_config_path) {
        Ok(runner) => {
            println!("Successfully created ShimRunner!");
            println!("Config: {:?}", runner.config());

            // Try to execute with version argument
            println!("Executing 'version' command...");
            match runner.execute(&["version".to_string()]) {
                Ok(exit_code) => {
                    println!(
                        "Command executed successfully with exit code: {}",
                        exit_code
                    );
                }
                Err(e) => {
                    println!("Failed to execute command: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create ShimRunner: {}", e);
        }
    }

    Ok(())
}
