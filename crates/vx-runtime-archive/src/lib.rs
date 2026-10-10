//! VX Runtime Archive
//!
//! This crate provides archive extraction utilities for vx.
//! Supports multiple archive formats:
//!
//! - `.tar.gz` / `.tgz` - Gzip compressed tar archives
//! - `.tar.xz` / `.txz` - XZ compressed tar archives
//! - `.tar.zst` / `.tzst` - Zstandard compressed tar archives
//! - `.zip` - ZIP archives
//! - `.7z` - 7-Zip archives
//!
//! # Why a separate archive crate?
//!
//! Archive handling libraries (tar, flate2, xz2, zstd, zip, sevenz-rust)
//! are heavy dependencies that take significant time to compile (~30-40s).
//! By separating them into their own crate:
//!
//! - Providers don't need to compile archive code
//! - Only `vx-runtime` (the facade) depends on this crate
//! - Faster incremental builds for provider development

use anyhow::{Result, anyhow};
use std::io::Read;
use std::path::Path;

/// Archive format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// Gzip compressed tar (.tar.gz, .tgz)
    TarGz,
    /// XZ compressed tar (.tar.xz, .txz)
    TarXz,
    /// Zstandard compressed tar (.tar.zst, .tzst)
    TarZst,
    /// ZIP archive (.zip)
    Zip,
    /// 7-Zip archive (.7z)
    #[cfg(feature = "extended-formats")]
    SevenZ,
}

impl ArchiveFormat {
    /// Detect archive format from file extension
    pub fn from_path(path: &Path) -> Option<Self> {
        let path_str = path.to_string_lossy().to_lowercase();

        if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
            Some(Self::TarGz)
        } else if path_str.ends_with(".tar.xz") || path_str.ends_with(".txz") {
            Some(Self::TarXz)
        } else if path_str.ends_with(".tar.zst") || path_str.ends_with(".tzst") {
            Some(Self::TarZst)
        } else if path_str.ends_with(".zip") {
            Some(Self::Zip)
        } else if path_str.ends_with(".7z") {
            #[cfg(feature = "extended-formats")]
            {
                Some(Self::SevenZ)
            }
            #[cfg(not(feature = "extended-formats"))]
            {
                None
            }
        } else {
            None
        }
    }

    /// Detect archive format from magic bytes
    pub fn from_magic_bytes(magic: &[u8]) -> Option<Self> {
        if magic.len() < 6 {
            return None;
        }

        // ZIP magic: PK\x03\x04
        if magic[0] == 0x50 && magic[1] == 0x4B {
            return Some(Self::Zip);
        }

        // GZIP magic: \x1f\x8b
        if magic[0] == 0x1f && magic[1] == 0x8b {
            return Some(Self::TarGz);
        }

        // XZ magic: \xFD7zXZ
        if magic[0] == 0xFD && magic[1] == 0x37 && magic[2] == 0x7A && magic[3] == 0x58 {
            return Some(Self::TarXz);
        }

        // Zstd magic: \x28\xB5\x2F\xFD
        if magic[0] == 0x28 && magic[1] == 0xB5 && magic[2] == 0x2F && magic[3] == 0xFD {
            return Some(Self::TarZst);
        }

        // 7z magic: 7z\xBC\xAF\x27\x1C
        #[cfg(feature = "extended-formats")]
        if magic[0] == 0x37
            && magic[1] == 0x7A
            && magic[2] == 0xBC
            && magic[3] == 0xAF
            && magic[4] == 0x27
            && magic[5] == 0x1C
        {
            return Some(Self::SevenZ);
        }

        None
    }
}

/// Archive extractor
pub struct ArchiveExtractor;

impl ArchiveExtractor {
    /// Create a new archive extractor
    pub fn new() -> Self {
        Self
    }

    /// Extract an archive to a directory
    pub fn extract(&self, archive: &Path, dest: &Path) -> Result<()> {
        std::fs::create_dir_all(dest)?;

        // First try to determine format by extension, then by magic bytes
        let format = match ArchiveFormat::from_path(archive) {
            Some(f) => f,
            None => Self::detect_from_file(archive)?,
        };

        tracing::debug!(
            archive = %archive.display(),
            dest = %dest.display(),
            format = ?format,
            "Extracting archive"
        );

        match format {
            ArchiveFormat::TarGz => self.extract_tar_gz(archive, dest)?,
            ArchiveFormat::TarXz => self.extract_tar_xz(archive, dest)?,
            ArchiveFormat::TarZst => self.extract_tar_zst(archive, dest)?,
            ArchiveFormat::Zip => self.extract_zip(archive, dest)?,
            #[cfg(feature = "extended-formats")]
            ArchiveFormat::SevenZ => self.extract_7z(archive, dest)?,
        }

        Ok(())
    }

    /// Detect format from file contents (magic bytes)
    fn detect_from_file(path: &Path) -> Result<ArchiveFormat> {
        let mut file = std::fs::File::open(path)?;
        let mut magic = [0u8; 6];
        file.read_exact(&mut magic)?;

        ArchiveFormat::from_magic_bytes(&magic)
            .ok_or_else(|| anyhow!("Unknown archive format: {}", path.display()))
    }

    fn extract_tar_gz(&self, archive: &Path, dest: &Path) -> Result<()> {
        let file = std::fs::File::open(archive)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(dest)?;
        Ok(())
    }

    fn extract_tar_xz(&self, archive: &Path, dest: &Path) -> Result<()> {
        use xz2::read::XzDecoder;
        let file = std::fs::File::open(archive)?;
        let decoder = XzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(dest)?;
        Ok(())
    }

    fn extract_tar_zst(&self, archive: &Path, dest: &Path) -> Result<()> {
        let file = std::fs::File::open(archive)?;
        let decoder = zstd::stream::read::Decoder::new(std::io::BufReader::new(file))?;
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(dest)?;
        Ok(())
    }

    fn extract_zip(&self, archive_path: &Path, dest: &Path) -> Result<()> {
        let file = std::fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            anyhow!(
                "Failed to open zip archive {}: {}",
                archive_path.display(),
                e
            )
        })?;
        let total_entries = archive.len();

        tracing::debug!(
            archive = %archive_path.display(),
            total_entries,
            dest = %dest.display(),
            "Extracting zip archive entry-by-entry"
        );

        let mut extracted_files: usize = 0;
        let mut extracted_dirs: usize = 0;
        let mut skipped_entries: usize = 0;
        #[cfg(unix)]
        let mut extracted_symlinks = Vec::new();
        #[cfg(windows)]
        let mut pending_links = Vec::new();

        for i in 0..total_entries {
            let mut entry = match archive.by_index(i) {
                Ok(entry) => entry,
                Err(e) => {
                    tracing::warn!(
                        index = i,
                        total = total_entries,
                        error = %e,
                        "Failed to read zip entry at index {}; skipping",
                        i
                    );
                    skipped_entries += 1;
                    continue;
                }
            };

            // enclosed_name() returns a sanitized path, filtering out
            // path-traversal attacks (e.g. ../etc/passwd). Entries with
            // invalid names are skipped.
            let entry_path = match entry.enclosed_name() {
                Some(path) => dest.join(path),
                None => {
                    tracing::debug!(
                        name = entry.name(),
                        "Skipping zip entry with invalid (non-enclosed) name"
                    );
                    skipped_entries += 1;
                    continue;
                }
            };

            ensure_zip_output_path(dest, &entry_path)?;
            #[cfg(windows)]
            {
                anyhow::ensure!(
                    !pending_links.iter().any(
                        |(link, _): &(std::path::PathBuf, std::path::PathBuf)| {
                            windows_zip_path_starts_with(&entry_path, link)
                        }
                    ),
                    "ZIP entry writes through a pending symlink"
                );
            }
            #[cfg(windows)]
            let pending_path = entry_path.clone();

            // Create parent directories before writing the file.
            // This handles archives where directory entries are implicit
            // (not stored as explicit entries), like Go's Windows archives.
            if let Some(parent) = entry_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    anyhow!(
                        "Failed to create parent directory {} for zip entry {}: {}",
                        parent.display(),
                        entry.name(),
                        e
                    )
                })?;
            }

            if entry.is_symlink() {
                #[cfg(unix)]
                {
                    anyhow::ensure!(entry.size() <= 4096, "ZIP symlink target is too long");
                    let mut target = String::new();
                    let mut limited = std::io::Read::take(&mut entry, 4097);
                    std::io::Read::read_to_string(&mut limited, &mut target)?;
                    anyhow::ensure!(target.len() <= 4096, "ZIP symlink target is too long");
                    validate_zip_symlink_target(dest, &entry_path, Path::new(&target))?;
                    std::os::unix::fs::symlink(&target, &entry_path)?;
                    extracted_symlinks.push(entry_path);
                    extracted_files += 1;
                }
                #[cfg(windows)]
                {
                    anyhow::ensure!(entry.size() <= 4096, "ZIP symlink target is too long");
                    let mut target = String::new();
                    let mut limited = std::io::Read::take(&mut entry, 4097);
                    std::io::Read::read_to_string(&mut limited, &mut target)?;
                    anyhow::ensure!(target.len() <= 4096, "ZIP symlink target is too long");
                    let source = windows_zip_link_target(dest, &pending_path, &target)?;
                    anyhow::ensure!(
                        pending_path
                            .symlink_metadata()
                            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound),
                        "ZIP symlink would overwrite an existing path"
                    );
                    pending_links.push((pending_path, source));
                    extracted_files += 1;
                }
                #[cfg(not(any(unix, windows)))]
                anyhow::bail!("ZIP symlink extraction is unsupported on this platform");
            } else if entry.is_dir() {
                std::fs::create_dir_all(&entry_path).map_err(|e| {
                    anyhow!(
                        "Failed to create directory {} from zip entry {}: {}",
                        entry_path.display(),
                        entry.name(),
                        e
                    )
                })?;
                extracted_dirs += 1;
            } else {
                let mut output = std::fs::File::create(&entry_path).map_err(|e| {
                    anyhow!(
                        "Failed to create file {} from zip entry {}: {}",
                        entry_path.display(),
                        entry.name(),
                        e
                    )
                })?;
                std::io::copy(&mut entry, &mut output).map_err(|e| {
                    anyhow!(
                        "Failed to extract zip entry {} to {}: {}",
                        entry.name(),
                        entry_path.display(),
                        e
                    )
                })?;
                #[cfg(unix)]
                if entry.is_file()
                    && let Some(mode) = entry.unix_mode()
                {
                    use std::os::unix::fs::PermissionsExt;

                    output
                        .set_permissions(std::fs::Permissions::from_mode(mode & 0o777))
                        .map_err(|e| {
                            anyhow!(
                                "Failed to preserve permissions for {}: {}",
                                entry_path.display(),
                                e
                            )
                        })?;
                }
                extracted_files += 1;
            }
        }

        #[cfg(unix)]
        validate_created_zip_symlinks(dest, &extracted_symlinks)?;
        #[cfg(windows)]
        materialize_windows_zip_links(dest, &pending_links)?;

        tracing::info!(
            archive = %archive_path.display(),
            total_entries,
            extracted_files,
            extracted_dirs,
            skipped_entries,
            "Zip extraction complete"
        );

        // Verify extraction was substantially complete.
        // We allow a small number of invalid/skipped entries, but if a significant
        // fraction is missing the extraction was likely truncated.
        if total_entries > 0 {
            let extracted_total = extracted_files + extracted_dirs;
            let missing = total_entries.saturating_sub(extracted_total + skipped_entries);
            let missing_ratio = missing as f64 / total_entries as f64;

            if missing_ratio > 0.05 {
                return Err(anyhow!(
                    "Zip extraction appears incomplete: {}/{} entries missing ({:.1}% missing, {} files + {} dirs extracted, {} skipped). \
                     The archive may be corrupt or extraction was interrupted.",
                    missing,
                    total_entries,
                    missing_ratio * 100.0,
                    extracted_files,
                    extracted_dirs,
                    skipped_entries,
                ));
            }

            if missing > 0 {
                tracing::warn!(
                    missing,
                    total_entries,
                    "{}/{} zip entries were not extracted (may be symlinks or special files)",
                    missing,
                    total_entries
                );
            }
        }

        Ok(())
    }

    #[cfg(feature = "extended-formats")]
    fn extract_7z(&self, archive: &Path, dest: &Path) -> Result<()> {
        sevenz_rust::decompress_file(archive, dest)
            .map_err(|e| anyhow!("Failed to extract 7z archive: {}", e))?;
        Ok(())
    }

    /// Check if a file is an archive by extension or magic bytes
    pub fn is_archive(path: &Path) -> bool {
        if ArchiveFormat::from_path(path).is_some() {
            return true;
        }

        // Try magic bytes
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut magic = [0u8; 6];
            if file.read_exact(&mut magic).is_ok() {
                return ArchiveFormat::from_magic_bytes(&magic).is_some();
            }
        }

        false
    }
}

impl Default for ArchiveExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Never follow an existing link while creating an archive entry.
fn ensure_zip_output_path(root: &Path, path: &Path) -> std::io::Result<()> {
    use std::path::Component;

    let invalid = || std::io::Error::new(std::io::ErrorKind::InvalidData, "Unsafe ZIP output path");
    let relative = path.strip_prefix(root).map_err(|_| invalid())?;
    if relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
    {
        return Err(invalid());
    }
    let mut current = root.to_path_buf();
    for component in std::iter::once(None).chain(relative.components().map(Some)) {
        if let Some(component) = component {
            current.push(component.as_os_str());
        }
        match current.symlink_metadata() {
            Ok(metadata) if zip_path_is_link(&metadata) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Symlink in ZIP output path",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn zip_path_is_link(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0 // FILE_ATTRIBUTE_REPARSE_POINT, including junctions.
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

#[cfg(windows)]
fn windows_zip_path_starts_with(path: &Path, prefix: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        #[link_name = "CompareStringOrdinal"]
        fn compare_string_ordinal(
            left: *const u16,
            left_length: i32,
            right: *const u16,
            right_length: i32,
            ignore_case: i32,
        ) -> i32;
    }

    let mut components = path.components();
    for prefix_component in prefix.components() {
        let Some(component) = components.next() else {
            return false;
        };
        let left: Vec<u16> = component.as_os_str().encode_wide().collect();
        let right: Vec<u16> = prefix_component.as_os_str().encode_wide().collect();
        let (Ok(left_length), Ok(right_length)) =
            (i32::try_from(left.len()), i32::try_from(right.len()))
        else {
            return false;
        };
        // SAFETY: Both pointers refer to buffers valid for their explicit UTF-16
        // lengths. Windows ordinal folding matches case-insensitive path names.
        let comparison = unsafe {
            compare_string_ordinal(left.as_ptr(), left_length, right.as_ptr(), right_length, 1)
        };
        if comparison != 2 {
            // CSTR_EQUAL
            return false;
        }
    }
    true
}

#[cfg(windows)]
fn windows_zip_link_target(
    root: &Path,
    link: &Path,
    target: &str,
) -> std::io::Result<std::path::PathBuf> {
    use std::path::Component;
    let invalid = || {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unsafe or noncanonical ZIP symlink target",
        )
    };
    if target.is_empty() || target.contains(['\\', ':', '\0']) {
        return Err(invalid());
    }
    let mut relative = link
        .parent()
        .and_then(|path| path.strip_prefix(root).ok())
        .ok_or_else(invalid)?
        .to_path_buf();
    let mut normal_seen = false;
    for component in Path::new(target).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir if !normal_seen && relative.pop() => {}
            Component::Normal(name) => {
                normal_seen = true;
                relative.push(name);
            }
            _ => return Err(invalid()),
        }
    }
    Ok(root.join(relative))
}

/// Windows ZIPs may contain portable relative links without requiring Developer
/// Mode. Materialize validated targets after all regular entries have arrived.
#[cfg(windows)]
fn materialize_windows_zip_links(
    root: &Path,
    links: &[(std::path::PathBuf, std::path::PathBuf)],
) -> std::io::Result<()> {
    use std::fs::{File, OpenOptions};
    use std::path::PathBuf;
    let invalid = || {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unsafe, cyclic or missing ZIP symlink target",
        )
    };

    fn copy_target(
        root: &Path,
        source: &Path,
        destination: &Path,
        created: &mut Vec<PathBuf>,
    ) -> std::io::Result<()> {
        ensure_zip_output_path(root, source)?;
        ensure_zip_output_path(root, destination)?;
        // Canonical existing paths also catch aliases with different Windows
        // case/short names before a directory could be copied into itself.
        let canonical_source = source.canonicalize()?;
        let canonical_destination = destination
            .parent()
            .ok_or_else(|| std::io::Error::other("Missing ZIP link parent"))?
            .canonicalize()?
            .join(
                destination
                    .file_name()
                    .ok_or_else(|| std::io::Error::other("Missing ZIP link name"))?,
            );
        if windows_zip_path_starts_with(&canonical_destination, &canonical_source) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "ZIP symlink target contains its destination",
            ));
        }
        let mut pending = vec![(source.to_path_buf(), destination.to_path_buf())];
        while let Some((source, destination)) = pending.pop() {
            ensure_zip_output_path(root, &source)?;
            ensure_zip_output_path(root, &destination)?;
            let metadata = source.symlink_metadata()?;
            if metadata.is_dir() {
                std::fs::create_dir(&destination)?;
                created.push(destination.clone());
                for entry in std::fs::read_dir(&source)? {
                    let entry = entry?;
                    pending.push((entry.path(), destination.join(entry.file_name())));
                }
            } else if metadata.is_file() {
                let mut output = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&destination)?;
                created.push(destination);
                std::io::copy(&mut File::open(source)?, &mut output)?;
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unsupported ZIP symlink target type",
                ));
            }
        }
        Ok(())
    }

    fn materialize(
        root: &Path,
        links: &[(PathBuf, PathBuf)],
        index: usize,
        states: &mut [u8],
        depth: usize,
        created: &mut Vec<PathBuf>,
    ) -> std::io::Result<()> {
        if states[index] == 2 {
            return Ok(());
        }
        if states[index] == 1 || depth >= 40 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Cyclic or excessive ZIP symlink chain",
            ));
        }
        states[index] = 1;
        let (destination, source) = &links[index];
        // Resolve an alias used by the target, and aliases anywhere inside a
        // directory target, before copying. This includes forward references.
        for (other, (link, _)) in links.iter().enumerate() {
            if windows_zip_path_starts_with(source, link)
                || windows_zip_path_starts_with(link, source)
            {
                materialize(root, links, other, states, depth + 1, created)?;
            }
        }
        copy_target(root, source, destination, created)?;
        states[index] = 2;
        Ok(())
    }

    let mut states = vec![0; links.len()];
    let mut created = Vec::new();
    let result = (|| {
        for (destination, source) in links {
            ensure_zip_output_path(root, destination)?;
            ensure_zip_output_path(root, source)?;
            match destination.symlink_metadata() {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                _ => return Err(invalid()),
            }
        }
        for index in 0..links.len() {
            materialize(root, links, index, &mut states, 0, &mut created)?;
        }
        Ok(())
    })();
    if result.is_err() {
        // Only remove paths this call created, in reverse order. Never recurse
        // through a pre-existing file, directory, symlink or junction.
        for path in created.iter().rev() {
            let _ = std::fs::remove_file(path).or_else(|_| std::fs::remove_dir(path));
        }
    }
    result
}

#[cfg(unix)]
fn validate_zip_symlink_target(root: &Path, link: &Path, target: &Path) -> std::io::Result<()> {
    use std::path::Component;

    fn resolve(
        root: &Path,
        base: &Path,
        target: &Path,
        budget: &mut usize,
    ) -> std::io::Result<std::path::PathBuf> {
        let invalid = || {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unsafe or noncanonical ZIP symlink target",
            )
        };
        let text = target.to_str().ok_or_else(invalid)?;
        if text.is_empty() || text.contains(['\\', ':', '\0']) {
            return Err(invalid());
        }
        let mut normal_seen = false;
        for component in target.components() {
            match component {
                Component::Normal(_) => normal_seen = true,
                Component::CurDir => {}
                Component::ParentDir if !normal_seen => {}
                _ => return Err(invalid()),
            }
        }
        let mut resolved = base.to_path_buf();
        for component in target.components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    if !resolved.pop() {
                        return Err(invalid());
                    }
                }
                Component::Normal(name) => {
                    resolved.push(name);
                    match root.join(&resolved).symlink_metadata() {
                        Ok(metadata) if metadata.file_type().is_symlink() => {
                            if *budget == 0 {
                                return Err(invalid());
                            }
                            *budget -= 1;
                            let nested = std::fs::read_link(root.join(&resolved))?;
                            let parent = resolved.parent().ok_or_else(invalid)?.to_path_buf();
                            resolved = resolve(root, &parent, &nested, budget)?;
                        }
                        Ok(_) => {}
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                        Err(error) => return Err(error),
                    }
                }
                _ => return Err(invalid()),
            }
        }
        Ok(resolved)
    }

    let parent = link
        .parent()
        .and_then(|parent| parent.strip_prefix(root).ok())
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Unsafe ZIP symlink path")
        })?;
    resolve(root, parent, target, &mut 40).map(|_| ())
}

#[cfg(unix)]
fn validate_created_zip_symlinks(root: &Path, links: &[std::path::PathBuf]) -> std::io::Result<()> {
    for link in links {
        let validation = std::fs::read_link(link)
            .and_then(|target| validate_zip_symlink_target(root, link, &target));
        if let Err(error) = validation {
            for created in links.iter().rev() {
                let _ = std::fs::remove_file(created);
            }
            return Err(error);
        }
    }
    Ok(())
}

/// Extract archive to destination directory
pub fn extract(archive: &Path, dest: &Path) -> Result<()> {
    ArchiveExtractor::new().extract(archive, dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "extended-formats")]
    fn test_format_detection() {
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.tar.gz")),
            Some(ArchiveFormat::TarGz)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.tgz")),
            Some(ArchiveFormat::TarGz)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.zip")),
            Some(ArchiveFormat::Zip)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.7z")),
            Some(ArchiveFormat::SevenZ)
        );
    }

    #[test]
    fn test_magic_bytes_zip() {
        let magic = [0x50, 0x4B, 0x03, 0x04, 0x00, 0x00];
        assert_eq!(
            ArchiveFormat::from_magic_bytes(&magic),
            Some(ArchiveFormat::Zip)
        );
    }

    #[test]
    fn test_magic_bytes_gzip() {
        let magic = [0x1F, 0x8B, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            ArchiveFormat::from_magic_bytes(&magic),
            Some(ArchiveFormat::TarGz)
        );
    }
}
