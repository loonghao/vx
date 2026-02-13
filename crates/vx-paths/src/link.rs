//! Link strategy for file system operations
//!
//! This module provides cross-platform linking strategies to avoid
//! duplicating files when creating virtual environments.

use anyhow::Result;
use std::path::Path;

/// Link strategy for creating file references
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStrategy {
    /// Hard link (same filesystem, fastest, no extra space)
    HardLink,
    /// Symbolic link (cross-filesystem, Windows needs permissions)
    SymLink,
    /// Copy-on-Write (macOS APFS, Linux Btrfs/XFS)
    CopyOnWrite,
    /// Copy (fallback, slowest)
    Copy,
}

impl LinkStrategy {
    /// Automatically select the best strategy for the current platform
    pub fn auto() -> Self {
        if cfg!(target_os = "macos") {
            // macOS APFS supports CoW
            Self::CopyOnWrite
        } else if cfg!(target_os = "linux") {
            // Linux: prefer hard links
            Self::HardLink
        } else if cfg!(target_os = "windows") {
            // Windows: hard links work without special permissions
            Self::HardLink
        } else {
            Self::Copy
        }
    }

    /// Detect the best strategy for a given path
    pub fn detect(_path: &Path) -> Self {
        // For now, use auto detection based on platform
        // TODO: Actually test filesystem capabilities
        Self::auto()
    }

    /// Get a human-readable name for the strategy
    pub fn name(&self) -> &'static str {
        match self {
            Self::HardLink => "hard link",
            Self::SymLink => "symbolic link",
            Self::CopyOnWrite => "copy-on-write",
            Self::Copy => "copy",
        }
    }
}

impl Default for LinkStrategy {
    fn default() -> Self {
        Self::auto()
    }
}

/// Result of a link operation
#[derive(Debug)]
pub struct LinkResult {
    /// Whether the operation was successful
    pub success: bool,
    /// The strategy that was used
    pub strategy: LinkStrategy,
    /// Number of files linked
    pub files_linked: usize,
    /// Number of directories created
    pub dirs_created: usize,
}

impl LinkResult {
    /// Create a successful result
    pub fn success(strategy: LinkStrategy, files_linked: usize, dirs_created: usize) -> Self {
        Self {
            success: true,
            strategy,
            files_linked,
            dirs_created,
        }
    }

    /// Create a failed result
    pub fn failed(strategy: LinkStrategy) -> Self {
        Self {
            success: false,
            strategy,
            files_linked: 0,
            dirs_created: 0,
        }
    }
}

/// Create a link from src to dst using the specified strategy
pub fn create_link(src: &Path, dst: &Path, strategy: LinkStrategy) -> Result<()> {
    match strategy {
        LinkStrategy::HardLink => create_hard_link(src, dst),
        LinkStrategy::SymLink => create_symlink(src, dst),
        LinkStrategy::CopyOnWrite => create_cow_link(src, dst),
        LinkStrategy::Copy => copy_path(src, dst),
    }
}

/// Create a hard link
fn create_hard_link(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() {
        // Directories can't be hard-linked, need to recursively link files
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            create_hard_link(&src_path, &dst_path)?;
        }
    } else {
        // Ensure parent directory exists
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::hard_link(src, dst)?;
    }
    Ok(())
}

/// Create a symbolic link
fn create_symlink(src: &Path, dst: &Path) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, dst)?;
    }

    #[cfg(windows)]
    {
        if src.is_dir() {
            std::os::windows::fs::symlink_dir(src, dst)?;
        } else {
            std::os::windows::fs::symlink_file(src, dst)?;
        }
    }

    Ok(())
}

/// Create a copy-on-write link (or fallback to copy)
fn create_cow_link(src: &Path, dst: &Path) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: use clonefile
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let src_c = CString::new(src.as_os_str().as_bytes())?;
        let dst_c = CString::new(dst.as_os_str().as_bytes())?;

        // clonefile is available on macOS 10.12+
        unsafe extern "C" {
            fn clonefile(src: *const i8, dst: *const i8, flags: u32) -> i32;
        }


        let result = unsafe { clonefile(src_c.as_ptr(), dst_c.as_ptr(), 0) };
        if result == 0 {
            return Ok(());
        }
        // If clonefile fails, fall back to copy
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: try reflink, fall back to copy
        // This requires the reflink crate or ioctl FICLONE
        // For now, just copy
    }

    // Fallback to regular copy
    copy_path(src, dst)
}

/// Copy a file or directory
fn copy_path(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            copy_path(&src_path, &dst_path)?;
        }
    } else {
        // Ensure parent directory exists
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, dst)?;
    }
    Ok(())
}

/// Link a directory tree using the best available strategy
pub fn link_directory(src: &Path, dst: &Path) -> Result<LinkResult> {
    let strategy = LinkStrategy::detect(src);

    match create_link(src, dst, strategy) {
        Ok(()) => {
            // Count files and directories
            let (files, dirs) = count_entries(dst)?;
            Ok(LinkResult::success(strategy, files, dirs))
        }
        Err(e) => {
            // Try fallback strategies
            if strategy != LinkStrategy::Copy && create_link(src, dst, LinkStrategy::Copy).is_ok() {
                let (files, dirs) = count_entries(dst)?;
                return Ok(LinkResult::success(LinkStrategy::Copy, files, dirs));
            }
            Err(e)
        }
    }
}

/// Count files and directories in a path
fn count_entries(path: &Path) -> Result<(usize, usize)> {
    let mut files = 0;
    let mut dirs = 0;

    if path.is_dir() {
        dirs += 1;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let (f, d) = count_entries(&entry_path)?;
                files += f;
                dirs += d;
            } else {
                files += 1;
            }
        }
    } else {
        files += 1;
    }

    Ok((files, dirs))
}
