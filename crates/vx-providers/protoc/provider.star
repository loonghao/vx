# provider.star - Protocol Buffers compiler (protoc)
#
# Reuse pattern: Level 2 (partial override)
#   - fetch_versions: fully inherited from github.star
#   - download_url:   overridden — protoc uses non-standard platform naming:
#                     win64 / win32 / linux-x86_64 / linux-aarch_64 / osx-universal_binary
#
# Release URL format:
#   https://github.com/protocolbuffers/protobuf/releases/download/v{version}/protoc-{version}-{platform}.zip
# Examples:
#   protoc-29.2-win64.zip
#   protoc-29.2-linux-x86_64.zip
#   protoc-29.2-osx-universal_binary.zip

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "protoc"

def description():
    return "Protocol Buffers compiler"

def homepage():
    return "https://protobuf.dev"

def repository():
    return "https://github.com/protocolbuffers/protobuf"

def license():
    return "BSD-3-Clause"

def ecosystem():
    return "devtools"

def aliases():
    return ["protobuf"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "protoc",
        "executable":  "protoc",
        "description": "Protocol Buffers compiler",
        "aliases":     ["protobuf"],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "objects.githubusercontent.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — fully inherited from github.star
#
# protoc tags are "v29.2"; strip_v_prefix handled by releases_to_versions()
# so versions are stored as "29.2".
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("protocolbuffers", "protobuf", include_prereleases = False)

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Why override instead of make_download_url()?
#   - macOS uses universal binary regardless of arch (osx-universal_binary)
#   - Windows uses "win64"/"win32" instead of Rust triple
#   - Linux uses "linux-x86_64" / "linux-aarch_64" (note: aarch_64 not aarch64)
#   - Asset is always .zip
#   - Executable lives in bin/ subdirectory inside the archive
# ---------------------------------------------------------------------------

def _protoc_platform(ctx):
    """Map platform to protoc's platform suffix."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    platforms = {
        "windows/x64":  "win64",
        "windows/x86":  "win32",
        "macos/x64":    "osx-universal_binary",   # universal binary
        "macos/arm64":  "osx-universal_binary",   # universal binary
        "linux/x64":    "linux-x86_64",
        "linux/arm64":  "linux-aarch_64",          # note: aarch_64 (underscore)
        "linux/x86":    "linux-x86_32",
    }
    return platforms.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the protoc download URL for the given version and platform.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "29.2"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _protoc_platform(ctx)
    if not platform:
        return None

    # Asset: "protoc-29.2-linux-x86_64.zip"
    asset = "protoc-{}-{}.zip".format(version, platform)

    # Tag: "v29.2"
    tag = "v{}".format(version)

    return github_asset_url("protocolbuffers", "protobuf", tag, asset)

# ---------------------------------------------------------------------------
# install_layout — protoc archives have a bin/ subdirectory
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Describe how to extract the downloaded archive.

    protoc archives contain:
      bin/protoc[.exe]
      include/google/protobuf/*.proto

    Returns:
        Layout dict consumed by the vx installer
    """
    os  = ctx["platform"]["os"]
    exe = "bin/protoc.exe" if os == "windows" else "bin/protoc"

    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "bin/protoc"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir + "/bin",
    }

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for protoc."""
    return "{vx_home}/store/protoc"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    exe = "bin/protoc.exe" if os == "windows" else "bin/protoc"
    return "{install_dir}/" + exe

def post_install(ctx, version, install_dir):
    """No post-install steps needed for protoc."""
    return None

# ---------------------------------------------------------------------------
# constraints
# ---------------------------------------------------------------------------

constraints = [
    {
        "when":       "*",
        "recommends": [],
    },
]
