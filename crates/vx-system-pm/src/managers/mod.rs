//! System package manager implementations

use async_trait::async_trait;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

pub mod apt;
pub mod chocolatey;
pub mod homebrew;
pub mod scoop;
pub mod winget;

pub use apt::AptManager;
pub use chocolatey::ChocolateyManager;
pub use homebrew::HomebrewManager;
pub use scoop::ScoopManager;
pub use winget::WingetManager;

/// Shared progress callback type for package managers
pub type ProgressCallback = Arc<dyn Fn(&str) + Send + Sync>;

fn spawn_stream_reader<R: Read + Send + 'static>(
    mut reader: R,
    sink: Arc<Mutex<Vec<u8>>>,
    tx: mpsc::Sender<String>,
) -> thread::JoinHandle<std::io::Result<()>> {
    thread::spawn(move || {
        let mut buffer = [0_u8; 1024];
        let mut segment = Vec::new();

        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }

            {
                let mut collected = sink.lock().unwrap();
                collected.extend_from_slice(&buffer[..n]);
            }

            for &b in &buffer[..n] {
                if b == b'\n' || b == b'\r' {
                    if !segment.is_empty() {
                        let msg = String::from_utf8_lossy(&segment).to_string();
                        let _ = tx.send(msg);
                        segment.clear();
                    }
                } else {
                    segment.push(b);
                }
            }
        }

        if !segment.is_empty() {
            let msg = String::from_utf8_lossy(&segment).to_string();
            let _ = tx.send(msg);
        }

        Ok(())
    })
}

pub(crate) fn run_command_with_progress(
    mut cmd: Command,
    progress_callback: &ProgressCallback,
) -> std::io::Result<Output> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| std::io::Error::other("failed to capture stderr"))?;

    let stdout_buf = Arc::new(Mutex::new(Vec::new()));
    let stderr_buf = Arc::new(Mutex::new(Vec::new()));

    let (tx, rx) = mpsc::channel::<String>();

    let stdout_handle = spawn_stream_reader(stdout, Arc::clone(&stdout_buf), tx.clone());
    let stderr_handle = spawn_stream_reader(stderr, Arc::clone(&stderr_buf), tx.clone());
    drop(tx);

    for msg in rx {
        if !msg.trim().is_empty() {
            progress_callback(&msg);
        }
    }

    let status = child.wait()?;

    stdout_handle
        .join()
        .map_err(|_| std::io::Error::other("stdout reader thread panicked"))??;
    stderr_handle
        .join()
        .map_err(|_| std::io::Error::other("stderr reader thread panicked"))??;

    let stdout = Arc::try_unwrap(stdout_buf)
        .map_err(|_| std::io::Error::other("stdout buffer still referenced"))?
        .into_inner()
        .map_err(|_| std::io::Error::other("stdout buffer poisoned"))?;
    let stderr = Arc::try_unwrap(stderr_buf)
        .map_err(|_| std::io::Error::other("stderr buffer still referenced"))?
        .into_inner()
        .map_err(|_| std::io::Error::other("stderr buffer poisoned"))?;

    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

/// System package manager trait
#[async_trait]
pub trait SystemPackageManager: Send + Sync {
    /// Package manager name
    fn name(&self) -> &str;

    /// Supported platforms
    fn supported_platforms(&self) -> Vec<&str>;

    /// Check if the package manager is installed
    async fn is_installed(&self) -> bool;

    /// Install the package manager itself
    async fn install_self(&self) -> crate::Result<()>;

    /// Install a package
    async fn install_package(&self, spec: &PackageInstallSpec) -> crate::Result<InstallResult>;

    /// Uninstall a package
    async fn uninstall_package(&self, package: &str) -> crate::Result<()>;

    /// Check if a package is installed
    async fn is_package_installed(&self, package: &str) -> crate::Result<bool>;

    /// Get installed version of a package
    async fn get_installed_version(&self, package: &str) -> crate::Result<Option<String>>;

    /// Priority (higher = preferred)
    fn priority(&self) -> i32 {
        50
    }

    /// Check if this package manager is supported on the current platform
    fn is_current_platform_supported(&self) -> bool {
        let current_os = std::env::consts::OS;
        self.supported_platforms()
            .iter()
            .any(|&p| p == current_os || p == "*")
    }
}

/// Package installation specification
#[derive(Debug, Clone, Default)]
pub struct PackageInstallSpec {
    /// Package name
    pub package: String,

    /// Version constraint
    pub version: Option<String>,

    /// Installation parameters (Chocolatey --params)
    pub params: Option<String>,

    /// Native installer arguments (Chocolatey --install-arguments)
    pub install_args: Option<String>,

    /// Silent installation
    pub silent: bool,

    /// Installation directory
    pub install_dir: Option<PathBuf>,
}

impl PackageInstallSpec {
    /// Create a new package install spec
    pub fn new(package: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            silent: true,
            ..Default::default()
        }
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set params
    pub fn with_params(mut self, params: impl Into<String>) -> Self {
        self.params = Some(params.into());
        self
    }

    /// Set install args
    pub fn with_install_args(mut self, args: impl Into<String>) -> Self {
        self.install_args = Some(args.into());
        self
    }

    /// Set install directory
    pub fn with_install_dir(mut self, dir: PathBuf) -> Self {
        self.install_dir = Some(dir);
        self
    }
}

/// Installation result
#[derive(Debug, Clone)]
pub struct InstallResult {
    /// Whether installation succeeded
    pub success: bool,

    /// Installed version
    pub version: Option<String>,

    /// Installation path
    pub install_path: Option<PathBuf>,

    /// Message from the installer
    pub message: Option<String>,
}

impl InstallResult {
    /// Create a successful result
    pub fn success() -> Self {
        Self {
            success: true,
            version: None,
            install_path: None,
            message: None,
        }
    }

    /// Create a failed result
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            version: None,
            install_path: None,
            message: Some(message.into()),
        }
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set install path
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.install_path = Some(path);
        self
    }
}
