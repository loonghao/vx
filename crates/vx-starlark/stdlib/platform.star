# @vx//stdlib:platform.star
# Platform detection utilities for vx provider scripts
#
# Usage:
#   load("@vx//stdlib:platform.star", "is_windows", "platform_triple", "arch_to_gnu")

def is_windows(ctx):
    """Return True if running on Windows."""
    return ctx.platform.os == "windows"

def is_macos(ctx):
    """Return True if running on macOS."""
    return ctx.platform.os == "macos"

def is_linux(ctx):
    """Return True if running on Linux."""
    return ctx.platform.os == "linux"

def is_x64(ctx):
    """Return True if running on x86_64 architecture."""
    return ctx.platform.arch == "x64"

def is_arm64(ctx):
    """Return True if running on ARM64 architecture."""
    return ctx.platform.arch == "arm64"

def platform_triple(ctx):
    """Get the Rust-style target triple for the current platform.

    Returns strings like:
    - "x86_64-pc-windows-msvc"
    - "x86_64-apple-darwin"
    - "aarch64-apple-darwin"
    - "x86_64-unknown-linux-gnu"
    - "aarch64-unknown-linux-gnu"
    """
    return ctx.platform.target

def arch_to_gnu(arch):
    """Convert vx arch name to GNU arch name.

    Args:
        arch: vx arch string ("x64", "arm64", "x86")

    Returns:
        GNU arch string ("x86_64", "aarch64", "i686")
    """
    mapping = {
        "x64": "x86_64",
        "arm64": "aarch64",
        "x86": "i686",
    }
    return mapping.get(arch, arch)

def arch_to_go(arch):
    """Convert vx arch name to Go GOARCH name.

    Args:
        arch: vx arch string ("x64", "arm64", "x86")

    Returns:
        Go GOARCH string ("amd64", "arm64", "386")
    """
    mapping = {
        "x64": "amd64",
        "arm64": "arm64",
        "x86": "386",
    }
    return mapping.get(arch, arch)

def os_to_go(os):
    """Convert vx OS name to Go GOOS name.

    Args:
        os: vx OS string ("windows", "macos", "linux")

    Returns:
        Go GOOS string ("windows", "darwin", "linux")
    """
    mapping = {
        "windows": "windows",
        "macos": "darwin",
        "linux": "linux",
    }
    return mapping.get(os, os)

def platform_ext(ctx):
    """Get the archive extension for the current platform.

    Returns:
    - ".zip" on Windows
    - ".tar.gz" on macOS/Linux
    """
    if is_windows(ctx):
        return ".zip"
    return ".tar.gz"

def exe_ext(ctx):
    """Get the executable extension for the current platform.

    Returns:
    - ".exe" on Windows
    - "" on macOS/Linux
    """
    if is_windows(ctx):
        return ".exe"
    return ""
