# @vx//stdlib:platform.star
# Platform detection utilities for vx provider scripts
#
# Usage:
#   load("@vx//stdlib:platform.star", "is_windows", "platform_triple", "arch_to_gnu")
#   load("@vx//stdlib:platform.star", "platform_map", "platform_select")
#   load("@vx//stdlib:platform.star", "rust_triple", "go_os_arch", "archive_ext",
#                                      "exe_suffix", "expand_asset")
#
# Note: ctx is a struct injected by the vx runtime:
#   ctx.platform.os     -> "windows" | "macos" | "linux"
#   ctx.platform.arch   -> "x64" | "arm64" | "x86"
#   ctx.platform.target -> "x86_64-pc-windows-msvc" | ...
#
# ┌─────────────────────────────────────────────────────────────────────────┐
# │  Detection helpers                                                      │
# │  is_windows/is_macos/is_linux/is_x64/is_arm64                          │
# │  platform_triple()    Rust-style target triple from ctx                 │
# │  arch_to_gnu/arch_to_go/os_to_go  Arch/OS name conversions             │
# │  platform_ext/exe_ext  Archive/exe extension helpers                   │
# │  platform_map()       Look up value from {os}/{arch} keyed dict        │
# │  platform_select()    Select value by OS                               │
# ├─────────────────────────────────────────────────────────────────────────┤
# │  Asset template helpers (shared by layout.star & provider_templates)   │
# │  RUST_TRIPLES_MUSL / RUST_TRIPLES_GNU  Target triple dicts             │
# │  rust_triple(ctx, linux_libc)  Resolve Rust target triple              │
# │  go_os_arch(ctx)      Resolve Go-style (os, arch) tuple                │
# │  archive_ext(ctx)     "zip" on Windows, "tar.gz" elsewhere             │
# │  exe_suffix(ctx)      ".exe" on Windows, "" elsewhere                  │
# │  expand_asset(template, ctx, version, ...)  Expand asset template      │
# └─────────────────────────────────────────────────────────────────────────┘

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

# ---------------------------------------------------------------------------
# platform_map / platform_select — generic platform dispatch helpers
# ---------------------------------------------------------------------------

def platform_map(ctx, mapping, fallback = None):
    """Look up a value from a platform-keyed dict.

    The key is "{os}/{arch}" (e.g. "linux/x64", "macos/arm64").
    Use this to avoid repeating the same lookup pattern in every provider.

    Args:
        ctx:      Provider context
        mapping:  Dict with "{os}/{arch}" keys (e.g. {"linux/x64": "amd64"})
        fallback: Value to return when the key is not found (default: None)

    Returns:
        The mapped value, or `fallback` if not found.

    Example:
        _ARCH = {
            "windows/x64":   "x64",
            "macos/x64":     "x64",
            "macos/arm64":   "aarch64",
            "linux/x64":     "x64",
            "linux/arm64":   "aarch64",
        }
        arch = platform_map(ctx, _ARCH)
        if not arch:
            return None
    """
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return mapping.get(key, fallback)

def platform_select(ctx, windows = None, macos = None, linux = None,
                    fallback = None):
    """Select a value based on the current OS.

    Simpler than platform_map when you only need OS-level dispatch
    (not arch-level). Equivalent to a match on ctx.platform.os.

    Args:
        ctx:      Provider context
        windows:  Value to return on Windows
        macos:    Value to return on macOS
        linux:    Value to return on Linux
        fallback: Value to return for unknown OS (default: None)

    Returns:
        The value for the current OS, or `fallback`.

    Example:
        exe_dir = platform_select(ctx,
            windows = ctx.install_dir,
            macos   = ctx.install_dir + "/bin",
            linux   = ctx.install_dir + "/bin",
        )
    """
    os = ctx.platform.os
    if os == "windows" and windows != None:
        return windows
    if os == "macos" and macos != None:
        return macos
    if os == "linux" and linux != None:
        return linux
    return fallback

# ---------------------------------------------------------------------------
# Rust target triple helpers
# ---------------------------------------------------------------------------

# Rust target triples — musl on Linux (portable, no glibc version dependency)
RUST_TRIPLES_MUSL = {
    "windows/x64":   "x86_64-pc-windows-msvc",
    "windows/arm64": "aarch64-pc-windows-msvc",
    "macos/x64":     "x86_64-apple-darwin",
    "macos/arm64":   "aarch64-apple-darwin",
    "linux/x64":     "x86_64-unknown-linux-musl",
    "linux/arm64":   "aarch64-unknown-linux-musl",
}

# Rust target triples — gnu on Linux
RUST_TRIPLES_GNU = {
    "windows/x64":   "x86_64-pc-windows-msvc",
    "windows/arm64": "aarch64-pc-windows-msvc",
    "macos/x64":     "x86_64-apple-darwin",
    "macos/arm64":   "aarch64-apple-darwin",
    "linux/x64":     "x86_64-unknown-linux-gnu",
    "linux/arm64":   "aarch64-unknown-linux-gnu",
}

def rust_triple(ctx, linux_libc = "musl"):
    """Resolve the Rust target triple for the current platform.

    Args:
        ctx:        Provider context
        linux_libc: Linux C library: "musl" (default, portable) or "gnu"

    Returns:
        Rust target triple string, or None if platform is unsupported.

    Example:
        triple = rust_triple(ctx)           # musl (default)
        triple = rust_triple(ctx, "gnu")    # gnu
    """
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    if linux_libc == "gnu":
        return RUST_TRIPLES_GNU.get(key)
    return RUST_TRIPLES_MUSL.get(key)

def go_os_arch(ctx):
    """Resolve Go-style (os, arch) tuple for the current platform.

    Returns:
        A tuple (go_os, go_arch), e.g. ("linux", "amd64").

    Example:
        go_os, go_arch = go_os_arch(ctx)
    """
    return (os_to_go(ctx.platform.os), arch_to_go(ctx.platform.arch))

def archive_ext(ctx):
    """Return the archive extension for the current platform.

    Returns:
        "zip" on Windows, "tar.gz" elsewhere.
    """
    return "zip" if ctx.platform.os == "windows" else "tar.gz"

def exe_suffix(ctx):
    """Return the executable suffix for the current platform.

    Returns:
        ".exe" on Windows, "" elsewhere.
    """
    return ".exe" if ctx.platform.os == "windows" else ""

def expand_asset(template, ctx, version, triple = None, go_os = None, go_arch = None):
    """Expand an asset filename template with platform-specific values.

    Placeholders:
        {version}  - version without 'v' prefix  (e.g. "1.0.0")
        {vversion} - version with 'v' prefix      (e.g. "v1.0.0")
        {triple}   - Rust target triple           (e.g. "x86_64-unknown-linux-musl")
        {os}       - Go GOOS                      (e.g. "linux", "darwin", "windows")
        {arch}     - Go GOARCH                    (e.g. "amd64", "arm64")
        {ext}      - archive extension            (e.g. "zip" or "tar.gz")
        {exe}      - executable suffix            (e.g. ".exe" or "")

    Args:
        template:  Asset filename template string
        ctx:       Provider context
        version:   Version string (without 'v' prefix)
        triple:    Rust target triple (optional)
        go_os:     Go GOOS string (optional)
        go_arch:   Go GOARCH string (optional)

    Returns:
        Expanded asset filename string.

    Example:
        fname = expand_asset("mytool-{vversion}-{triple}.{ext}", ctx, "1.0.0",
                             triple=rust_triple(ctx))
    """
    ext = archive_ext(ctx)
    exe = exe_suffix(ctx)

    s = template
    s = s.replace("{version}",  version)
    s = s.replace("{vversion}", "v" + version)
    s = s.replace("{ext}",      ext)
    s = s.replace("{exe}",      exe)
    if triple != None:
        s = s.replace("{triple}", triple)
    if go_os != None:
        s = s.replace("{os}",   go_os)
    if go_arch != None:
        s = s.replace("{arch}", go_arch)
    return s
