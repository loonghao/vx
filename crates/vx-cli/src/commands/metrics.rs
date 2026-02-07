//! `vx metrics` — View execution performance metrics and reports.
//!
//! Provides terminal visualization, multi-run comparison, HTML export,
//! and AI-friendly JSON summary of pipeline stage timings.

use anyhow::Result;

/// Handle `vx metrics` command.
pub async fn handle(last: usize, json: bool, html: Option<String>, clean: bool) -> Result<()> {
    let metrics_dir = vx_paths::VxPaths::default().base_dir.join("metrics");

    if clean {
        return handle_clean(&metrics_dir).await;
    }

    if !metrics_dir.exists() {
        println!("No metrics data found at {}", metrics_dir.display());
        println!("Run a command (e.g., `vx node --version`) to generate metrics.");
        return Ok(());
    }

    let runs = vx_metrics::load_metrics(&metrics_dir, last)?;

    if runs.is_empty() {
        println!("No metrics data found. Run a command to generate metrics.");
        return Ok(());
    }

    // JSON mode (AI-friendly)
    if json {
        let summary = vx_metrics::generate_ai_summary(&runs);
        println!("{}", serde_json::to_string_pretty(&summary)?);
        return Ok(());
    }

    // HTML export mode
    if let Some(output_path) = html {
        let html_content = vx_metrics::generate_html_report(&runs);
        let path = if output_path.is_empty() {
            metrics_dir.join("report.html")
        } else {
            std::path::PathBuf::from(&output_path)
        };
        std::fs::write(&path, &html_content)?;
        println!("HTML report written to: {}", path.display());

        // Try to open in browser
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/c", "start", &path.to_string_lossy()])
                .spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(&path)
                .spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(&path)
                .spawn();
        }

        return Ok(());
    }

    // Terminal mode
    if runs.len() == 1 || last == 1 {
        // Single run summary
        print!("{}", vx_metrics::render_summary(&runs[0]));
    } else {
        // Multi-run comparison
        print!("{}", vx_metrics::render_comparison(&runs));
    }

    // Always show insights
    print!("{}", vx_metrics::render_insights(&runs));

    Ok(())
}

async fn handle_clean(metrics_dir: &std::path::Path) -> Result<()> {
    if !metrics_dir.exists() {
        println!("No metrics directory to clean.");
        return Ok(());
    }

    let mut count = 0;
    for entry in std::fs::read_dir(metrics_dir)? {
        let entry = entry?;
        if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
            std::fs::remove_file(entry.path())?;
            count += 1;
        }
    }

    // Also remove any HTML reports
    for entry in std::fs::read_dir(metrics_dir)? {
        let entry = entry?;
        if entry.path().extension().map(|e| e == "html").unwrap_or(false) {
            std::fs::remove_file(entry.path())?;
            count += 1;
        }
    }

    println!("Removed {} metrics files.", count);
    Ok(())
}
