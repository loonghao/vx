# provider.star - Hadolint provider
#
# Hadolint is a Dockerfile linter that helps you build best practice Docker images.
#
# Inheritance level: 2 (partial override)
#   - fetch_versions: fully inherited from @vx//stdlib:github.star
#   - download_url:   overridden — hadolint uses direct binary downloads (no archive)
#                     with {os}-{arch} naming (not Rust triple), and Windows only
#                     supports x86_64
#
# Release URL format:
#   https://github.com/hadolint/hadolint/releases/download/v{version}/hadolint-{os}-{arch}[.exe]

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "hadolint"

def description():
    return "Hadolint - Dockerfile linter, validate inline bash, written in Haskell"

def homepage():
    return "https://github.com/hadolint/hadolint"

def repository():
    return "https://github.com/hadolint/hadolint"

def license():
    return "GPL-3.0"

def ecosystem():
    return "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "hadolint",
        "executable":  "hadolint",
        "description": "Dockerfile linter",
        "aliases":     [],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — fully inherited from github.star
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("hadolint", "hadolint", include_prereleases = False)

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Why override:
#   - Direct binary download (no archive to extract)
#   - Naming: hadolint-{os}-{arch}[.exe]  (NOT Rust triple)
#   - Windows only supports x86_64 (no arm64)
#   - os strings: "Linux", "Darwin", "Windows" (capitalized)
# ---------------------------------------------------------------------------

def _hadolint_asset(ctx):
    """Return the asset filename for the current platform, or None if unsupported."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    # hadolint uses capitalized OS names
    os_map = {
        "linux":   "Linux",
        "macos":   "Darwin",
        "windows": "Windows",
    }

    # hadolint uses x86_64 / arm64 (not aarch64)
    arch_map = {
        "x64":   "x86_64",
        "arm64": "arm64",
    }

    os_str   = os_map.get(os)
    arch_str = arch_map.get(arch)

    if not os_str or not arch_str:
        return None

    # Windows only supports x86_64
    if os == "windows" and arch != "x64":
        return None

    ext = ".exe" if os == "windows" else ""
    return "hadolint-{}-{}{}".format(os_str, arch_str, ext)

def download_url(ctx, version):
    """Build the hadolint download URL.

    hadolint releases are direct binary downloads (not archives).

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "2.14.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    asset = _hadolint_asset(ctx)
    if not asset:
        return None

    tag = "v{}".format(version)
    return github_asset_url("hadolint", "hadolint", tag, asset)

# ---------------------------------------------------------------------------
# install_layout — direct binary (no archive)
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Hadolint is a direct binary download, no archive extraction needed."""
    os  = ctx["platform"]["os"]
    exe = "hadolint.exe" if os == "windows" else "hadolint"

    return {
        "type":             "binary",
        "executable_name":  exe,
    }

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for hadolint."""
    return "{vx_home}/store/hadolint"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    exe = "hadolint.exe" if os == "windows" else "hadolint"
    return "{install_dir}/" + exe

def post_install(ctx, version, install_dir):
    """No post-install steps needed for hadolint."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
