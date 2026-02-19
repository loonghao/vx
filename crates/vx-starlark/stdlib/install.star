# @vx//stdlib:install.star
# Installation layout descriptors for vx provider scripts
#
# Design: Starlark scripts are pure computation — they do NOT perform real
# installation. Instead, functions like msi_install() return a descriptor dict
# that the Rust runtime interprets to perform the actual installation I/O.
#
# This keeps Starlark sandboxed and testable, while the Rust layer handles
# all real I/O (download, extraction, msiexec, filesystem, etc.).
#
# Usage:
#   load("@vx//stdlib:install.star", "msi_install", "archive_install", "binary_install")
#
# Example (MSI provider):
#   load("@vx//stdlib:install.star", "msi_install")
#   load("@vx//stdlib:platform.star", "is_windows")
#
#   def download_url(ctx, version):
#       if not is_windows(ctx):
#           return None
#       return "https://example.com/tool-{}.msi".format(version)
#
#   def install_layout(ctx, version):
#       if not is_windows(ctx):
#           return None
#       url = download_url(ctx, version)
#       return msi_install(url, executable_paths = ["bin/tool.exe"])

# ---------------------------------------------------------------------------
# MSI installer descriptor (Windows only)
# ---------------------------------------------------------------------------

def msi_install(url, executable_paths = None, strip_prefix = None, extra_args = None):
    """Return an MSI installation descriptor for the Rust runtime to execute.

    Uses msiexec /a (administrative install) to extract the MSI contents to
    the target directory without modifying the system registry.

    This function does NOT perform any real installation. It returns a descriptor
    dict that the Rust runtime interprets to run msiexec.

    Args:
        url:              Download URL for the .msi file
        executable_paths: List of relative paths to executables within the
                          extracted MSI (e.g. ["bin/tool.exe", "tool.exe"]).
                          If None, the Rust runtime will auto-detect executables.
        strip_prefix:     Directory prefix to strip from extracted paths
                          (e.g. "PFiles/MyTool" if msiexec extracts there).
                          If None, no stripping is performed.
        extra_args:       Extra msiexec command-line properties
                          (e.g. ["ADDLOCAL=ALL"]).

    Returns:
        An install descriptor dict consumed by the Rust runtime.

    Example:
        return msi_install(
            "https://example.com/tool-1.0.msi",
            executable_paths = ["bin/tool.exe"],
            strip_prefix = "PFiles/Tool",
        )
    """
    descriptor = {
        "__type":    "msi_install",
        "url":       url,
    }
    if executable_paths != None:
        descriptor["executable_paths"] = executable_paths
    if strip_prefix != None:
        descriptor["strip_prefix"] = strip_prefix
    if extra_args != None:
        descriptor["extra_args"] = extra_args
    return descriptor

# ---------------------------------------------------------------------------
# Archive installer descriptor (ZIP, TAR.GZ, TAR.XZ, etc.)
# ---------------------------------------------------------------------------

def archive_install(url, strip_prefix = None, executable_paths = None):
    """Return an archive installation descriptor for the Rust runtime to execute.

    Supports ZIP, TAR.GZ, TAR.XZ, TAR.BZ2 archives. The format is auto-detected
    from the URL file extension.

    This function does NOT perform any real installation. It returns a descriptor
    dict that the Rust runtime interprets to download and extract the archive.

    Args:
        url:              Download URL for the archive file
        strip_prefix:     Directory prefix to strip from extracted paths
                          (e.g. "tool-1.0.0-linux-x64" for archives that
                          contain a top-level directory).
                          If None, no stripping is performed.
        executable_paths: List of relative paths to executables within the
                          extracted archive (e.g. ["bin/tool", "tool"]).
                          If None, the Rust runtime will auto-detect executables.

    Returns:
        An install descriptor dict consumed by the Rust runtime.

    Example:
        return archive_install(
            "https://example.com/tool-1.0-linux-x64.tar.gz",
            strip_prefix = "tool-1.0-linux-x64",
            executable_paths = ["bin/tool"],
        )
    """
    descriptor = {
        "__type": "archive_install",
        "url":    url,
    }
    if strip_prefix != None:
        descriptor["strip_prefix"] = strip_prefix
    if executable_paths != None:
        descriptor["executable_paths"] = executable_paths
    return descriptor

# ---------------------------------------------------------------------------
# Binary installer descriptor (single executable file)
# ---------------------------------------------------------------------------

def binary_install(url, executable_name = None, permissions = "755"):
    """Return a binary installation descriptor for the Rust runtime to execute.

    Downloads a single executable file directly (no archive extraction needed).

    This function does NOT perform any real installation. It returns a descriptor
    dict that the Rust runtime interprets to download the binary.

    Args:
        url:             Download URL for the binary file
        executable_name: Target filename for the downloaded binary.
                         If None, the filename is derived from the URL.
        permissions:     Unix file permissions (default: "755").
                         Ignored on Windows.

    Returns:
        An install descriptor dict consumed by the Rust runtime.

    Example:
        return binary_install(
            "https://example.com/tool-linux-x64",
            executable_name = "tool",
        )
    """
    descriptor = {
        "__type":     "binary_install",
        "url":        url,
        "permissions": permissions,
    }
    if executable_name != None:
        descriptor["executable_name"] = executable_name
    return descriptor

# ---------------------------------------------------------------------------
# Convenience: platform-aware install layout
# ---------------------------------------------------------------------------

def platform_install(ctx, windows_url = None, macos_url = None, linux_url = None,
                     strip_prefix = None, executable_paths = None,
                     windows_msi = False):
    """Return the appropriate install descriptor for the current platform.

    Automatically selects the right URL and install method based on the
    current platform. MSI is used for Windows when windows_msi=True.

    Args:
        ctx:              Provider context dict (injected by vx runtime)
        windows_url:      Download URL for Windows
        macos_url:        Download URL for macOS
        linux_url:        Download URL for Linux
        strip_prefix:     Directory prefix to strip (for archive installs)
        executable_paths: List of relative paths to executables
        windows_msi:      If True and on Windows, use msi_install() instead
                          of archive_install() for the Windows URL.

    Returns:
        An install descriptor dict, or None if no URL for current platform.

    Example:
        def install_layout(ctx, version):
            return platform_install(
                ctx,
                windows_url = "https://example.com/tool-{}.msi".format(version),
                macos_url   = "https://example.com/tool-{}-macos.tar.gz".format(version),
                linux_url   = "https://example.com/tool-{}-linux.tar.gz".format(version),
                windows_msi = True,
                executable_paths = ["bin/tool.exe"],
            )
    """
    os = ctx["platform"]["os"]

    if os == "windows":
        if windows_url == None:
            return None
        if windows_msi:
            return msi_install(windows_url, executable_paths = executable_paths,
                               strip_prefix = strip_prefix)
        return archive_install(windows_url, strip_prefix = strip_prefix,
                               executable_paths = executable_paths)
    elif os == "macos":
        if macos_url == None:
            return None
        return archive_install(macos_url, strip_prefix = strip_prefix,
                               executable_paths = executable_paths)
    elif os == "linux":
        if linux_url == None:
            return None
        return archive_install(linux_url, strip_prefix = strip_prefix,
                               executable_paths = executable_paths)
    return None
