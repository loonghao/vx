use anyhow::Result;
use reqwest;
use std::fs;
use std::io::Write;

use crate::ui::UI;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[derive(Debug, Clone)]
pub struct InstallConfig {
    pub tool_name: String,
    pub version: String,
    pub install_method: InstallMethod,
    pub download_url: Option<String>,
    pub install_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub enum InstallMethod {
    /// Download and extract archive
    Archive { format: ArchiveFormat },
    /// Use system package manager
    PackageManager { manager: String, package: String },
    /// Run installer script
    Script { url: String },
    /// Download single binary
    Binary,
}

#[derive(Debug, Clone)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
    TarXz,
}

pub struct Installer {
    client: reqwest::Client,
}

impl Default for Installer {
    fn default() -> Self {
        Self::new()
    }
}

impl Installer {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Install a tool with the given configuration
    pub async fn install(&self, config: &InstallConfig) -> Result<PathBuf> {
        UI::step(&format!(
            "Installing {} {}...",
            config.tool_name, config.version
        ));

        // Create install directory
        fs::create_dir_all(&config.install_dir)?;

        match &config.install_method {
            InstallMethod::Archive { format } => self.install_from_archive(config, format).await,
            InstallMethod::PackageManager { manager, package } => {
                self.install_from_package_manager(config, manager, package)
                    .await
            }
            InstallMethod::Script { url } => self.install_from_script(config, url).await,
            InstallMethod::Binary => self.install_binary(config).await,
        }
    }

    /// Install from archive (zip, tar.gz, etc.)
    async fn install_from_archive(
        &self,
        config: &InstallConfig,
        format: &ArchiveFormat,
    ) -> Result<PathBuf> {
        let download_url = config
            .download_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Download URL not provided"))?;

        UI::info(&format!("Downloading from {}", download_url));

        // Download to temporary file
        let temp_dir = TempDir::new()?;
        let temp_file = temp_dir.path().join("download");

        let response = self.client.get(download_url).send().await?;

        // Get content length for progress bar
        let total_size = response.content_length().unwrap_or(0);
        let progress_bar = if total_size > 0 {
            Some(UI::new_progress_bar(total_size))
        } else {
            None
        };

        // Download with progress
        let mut file = fs::File::create(&temp_file)?;
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            if let Some(pb) = &progress_bar {
                pb.set_position(downloaded);
            }
        }

        if let Some(pb) = progress_bar {
            pb.finish();
        }

        let spinner = UI::new_spinner("Extracting archive...");

        // Extract based on format
        match format {
            ArchiveFormat::Zip => self.extract_zip(&temp_file, &config.install_dir)?,
            ArchiveFormat::TarGz => self.extract_tar_gz(&temp_file, &config.install_dir)?,
            ArchiveFormat::TarXz => self.extract_tar_xz(&temp_file, &config.install_dir)?,
        }

        spinner.finish_and_clear();

        // Find the executable
        let exe_path = self.find_executable(&config.install_dir, &config.tool_name)?;

        UI::success(&format!(
            "{} installed to {}",
            config.tool_name,
            exe_path.display()
        ));
        Ok(exe_path)
    }

    /// Install using system package manager
    async fn install_from_package_manager(
        &self,
        config: &InstallConfig,
        manager: &str,
        package: &str,
    ) -> Result<PathBuf> {
        UI::step(&format!("Installing {} using {}...", package, manager));

        let status = match manager {
            "chocolatey" | "choco" => Command::new("choco")
                .args(["install", package, "-y"])
                .status()?,
            "winget" => Command::new("winget").args(["install", package]).status()?,
            "brew" => Command::new("brew").args(["install", package]).status()?,
            "apt" => Command::new("sudo")
                .args(["apt", "install", "-y", package])
                .status()?,
            _ => return Err(anyhow::anyhow!("Unsupported package manager: {}", manager)),
        };

        if !status.success() {
            return Err(anyhow::anyhow!("Package installation failed"));
        }

        // Try to find the installed executable
        match which::which(&config.tool_name) {
            Ok(path) => {
                UI::success(&format!("{} installed successfully", config.tool_name));
                Ok(path)
            }
            Err(_) => Err(anyhow::anyhow!("Tool installed but not found in PATH")),
        }
    }

    /// Install from script
    async fn install_from_script(
        &self,
        config: &InstallConfig,
        script_url: &str,
    ) -> Result<PathBuf> {
        UI::step(&format!("Running installation script from {}", script_url));

        let response = self.client.get(script_url).send().await?;
        let script_content = response.text().await?;

        let temp_dir = TempDir::new()?;
        let script_path = temp_dir.path().join("install.sh");
        fs::write(&script_path, script_content)?;

        // Make script executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }

        // Run the script
        let status = if cfg!(windows) {
            Command::new("powershell")
                .args([
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    &script_path.to_string_lossy(),
                ])
                .status()?
        } else {
            Command::new("bash").arg(&script_path).status()?
        };

        if !status.success() {
            return Err(anyhow::anyhow!("Installation script failed"));
        }

        // Try to find the installed executable
        match which::which(&config.tool_name) {
            Ok(path) => {
                UI::success(&format!("{} installed successfully", config.tool_name));
                Ok(path)
            }
            Err(_) => Err(anyhow::anyhow!("Tool installed but not found in PATH")),
        }
    }

    /// Install single binary
    async fn install_binary(&self, config: &InstallConfig) -> Result<PathBuf> {
        let download_url = config
            .download_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Download URL not provided"))?;

        UI::info(&format!("Downloading binary from {}", download_url));

        let response = self.client.get(download_url).send().await?;

        // Get content length for progress bar
        let total_size = response.content_length().unwrap_or(0);
        let progress_bar = if total_size > 0 {
            Some(UI::new_progress_bar(total_size))
        } else {
            None
        };

        // Download with progress
        let mut downloaded = 0u64;
        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            bytes.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;

            if let Some(pb) = &progress_bar {
                pb.set_position(downloaded);
            }
        }

        if let Some(pb) = progress_bar {
            pb.finish();
        }

        let exe_name = if cfg!(windows) {
            format!("{}.exe", config.tool_name)
        } else {
            config.tool_name.clone()
        };

        let exe_path = config.install_dir.join(&exe_name);
        fs::write(&exe_path, bytes)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&exe_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&exe_path, perms)?;
        }

        UI::success(&format!(
            "{} installed to {}",
            config.tool_name,
            exe_path.display()
        ));
        Ok(exe_path)
    }

    /// Extract ZIP archive
    fn extract_zip(&self, archive_path: &Path, dest_dir: &Path) -> Result<()> {
        let file = fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = dest_dir.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            // Set permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }
        }

        Ok(())
    }

    /// Extract tar.gz archive
    fn extract_tar_gz(&self, archive_path: &Path, dest_dir: &Path) -> Result<()> {
        let tar_gz = fs::File::open(archive_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(dest_dir)?;
        Ok(())
    }

    /// Extract tar.xz archive
    fn extract_tar_xz(&self, _archive_path: &Path, _dest_dir: &Path) -> Result<()> {
        // For now, return an error as we don't have xz support
        Err(anyhow::anyhow!("tar.xz extraction not yet supported"))
    }

    /// Find executable in directory
    fn find_executable(&self, dir: &Path, tool_name: &str) -> Result<PathBuf> {
        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        };

        // Search in the directory and subdirectories
        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry?;
            if entry.file_name() == std::ffi::OsStr::new(&exe_name) {
                return Ok(entry.path().to_path_buf());
            }
        }

        Err(anyhow::anyhow!(
            "Executable {} not found in {}",
            exe_name,
            dir.display()
        ))
    }
}

/// Simple wrapper function for tool installation
pub fn install_tool(tool_name: &str, version: &str, download_url: &str) -> Result<PathBuf> {
    // Create a basic install config
    let install_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vx")
        .join("tools")
        .join(tool_name)
        .join(version);

    let config = InstallConfig {
        tool_name: tool_name.to_string(),
        version: version.to_string(),
        install_method: InstallMethod::Archive {
            format: if download_url.ends_with(".zip") {
                ArchiveFormat::Zip
            } else {
                ArchiveFormat::TarGz
            },
        },
        download_url: Some(download_url.to_string()),
        install_dir,
    };

    // Use tokio runtime to run async installer
    let rt = tokio::runtime::Runtime::new()?;
    let installer = Installer::new();
    rt.block_on(installer.install(&config))
}
