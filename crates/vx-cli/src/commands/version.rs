//! Version command implementation

use crate::cli::OutputFormat;
use crate::output::{CommandOutput, OutputRenderer};
use anyhow::Result;
use serde::Serialize;

#[derive(Serialize)]
struct VersionOutput {
    name: &'static str,
    version: &'static str,
    description: &'static str,
}

impl CommandOutput for VersionOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer, "{} {}", self.name, self.version)?;
        writeln!(writer, "{}", self.description)?;
        Ok(())
    }

    fn render_compact(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer, "{} {}", self.name, self.version)?;
        Ok(())
    }
}

pub async fn handle(format: OutputFormat) -> Result<()> {
    let output = VersionOutput {
        name: "vx",
        version: env!("CARGO_PKG_VERSION"),
        description: "Universal Development Tool Manager",
    };

    OutputRenderer::new(format).render(&output)?;
    Ok(())
}
