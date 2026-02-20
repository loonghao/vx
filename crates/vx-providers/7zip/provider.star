# 7-Zip provider for vx
#
# Replaces the Rust runtime.rs implementation entirely.
# All logic is pure computation — no real I/O happens here.
# The Rust runtime interprets the returned descriptors to perform actual work.
#
# Design follows Buck2/Bazel's descriptor pattern:
#   Starlark = pure computation (what to do)
#   Rust     = actual execution  (how to do it)

load("@vx//stdlib:github.star", "github_releases", "releases_to_versions")
load("@vx//stdlib:install.star", "archive_install", "msi_install", "system_find")
load("@vx//stdlib:platform.star", "is_windows", "is_macos", "is_linux")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "7zip"

def description():
    return "7-Zip - High compression ratio file archiver supporting 7z, ZIP, TAR, GZ, XZ and more"

def homepage():
    return "https://www.7-zip.org"

def repository():
    return "https://github.com/ip7z/7zip"

def license():
    return "LGPL-2.1"

def ecosystem():
    return "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "7zip",
        "executable":  "7z",
        "description": "7-Zip - High compression ratio file archiver",
        "aliases":     ["7z", "7za", "7zz"],
        "priority":    100,
        "system_paths": [
            "C:/Program Files/7-Zip",
            "C:/Program Files (x86)/7-Zip",
            "/usr/bin",
            "/usr/local/bin",
            "/opt/homebrew/bin",
        ],
        "system_install": [
            {"manager": "winget", "package": "7zip.7zip",  "priority": 90, "platforms": ["windows"]},
            {"manager": "choco",  "package": "7zip",       "priority": 80, "platforms": ["windows"]},
            {"manager": "brew",   "package": "sevenzip",   "priority": 90, "platforms": ["macos"]},
            {"manager": "brew",   "package": "sevenzip",   "priority": 70, "platforms": ["linux"]},
            {"manager": "apt",    "package": "p7zip-full", "priority": 70, "platforms": ["linux"]},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": ["winget", "choco", "brew", "apt"],
}

# ---------------------------------------------------------------------------
# Version fetching
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch available 7-Zip versions from GitHub releases.

    7-Zip uses tags like "24.09", "23.01" (no 'v' prefix).
    """
    releases = github_releases(
        owner = "ip7z",
        repo = "7zip",
        include_prereleases = False,
    )
    return releases_to_versions(releases, strip_v_prefix = False)

# ---------------------------------------------------------------------------
# Download URL
# ---------------------------------------------------------------------------

def _ver_compact(version):
    """Convert "24.09" -> "2409" for 7-Zip asset naming."""
    return version.replace(".", "")

def download_url(ctx, version):
    """Return the download URL for the given version and platform.

    7-Zip asset naming convention:
      Windows x64:  7z{compact}-x64.msi   (e.g. 7z2409-x64.msi)
      Windows x86:  7z{compact}.msi        (e.g. 7z2409.msi)
      macOS:        7z{compact}-mac.tar.xz
      Linux x64:    7z{compact}-linux-x64.tar.xz
      Linux arm64:  7z{compact}-linux-arm64.tar.xz
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]
    ver  = _ver_compact(version)
    base = "https://github.com/ip7z/7zip/releases/download/{}".format(version)

    if os == "windows":
        if arch == "x64":
            return "{}/7z{}-x64.msi".format(base, ver)
        else:
            return "{}/7z{}.msi".format(base, ver)
    elif os == "macos":
        return "{}/7z{}-mac.tar.xz".format(base, ver)
    elif os == "linux":
        if arch == "arm64":
            return "{}/7z{}-linux-arm64.tar.xz".format(base, ver)
        else:
            return "{}/7z{}-linux-x64.tar.xz".format(base, ver)
    return None

# ---------------------------------------------------------------------------
# Install layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Return the install descriptor for the given version and platform.

    Windows uses MSI (extracted via msiexec /a, no registry changes).
    macOS and Linux use tar.xz archives.
    The 7z executable lives at the root of the extracted archive.
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]
    url  = download_url(ctx, version)

    if url == None:
        return None

    if os == "windows":
        # msiexec /a extracts to target dir; 7z.exe is at the root
        return msi_install(
            url,
            executable_paths = ["7z.exe"],
            strip_prefix = "PFiles\\7-Zip",
        )
    else:
        # tar.xz: 7z binary is at the root (no top-level directory)
        exe = "7zz" if os == "macos" else "7zz"
        return archive_install(
            url,
            executable_paths = [exe, "7z"],
        )

# ---------------------------------------------------------------------------
# Prepare execution  (system tool detection)
# ---------------------------------------------------------------------------

def prepare_execution(ctx, version):
    """Find 7z on the system before falling back to vx-managed installation.

    Follows the same pattern as Buck2's ctx.actions.run():
    Starlark declares *what to search for*, Rust performs the actual search.

    Search order (handled by Rust runtime):
      1. PATH lookup for "7z" / "7zz"
      2. Known Windows system paths
      3. Fall back to vx-managed installation
    """
    os = ctx["platform"]["os"]

    if os == "windows":
        return system_find(
            "7z",
            system_paths = [
                "C:\\Program Files\\7-Zip\\7z.exe",
                "C:\\Program Files (x86)\\7-Zip\\7z.exe",
            ],
            hint = "Install via: winget install 7zip.7zip  |  choco install 7zip",
        )
    elif os == "macos":
        return system_find(
            "7zz",
            hint = "Install via: brew install sevenzip",
        )
    else:
        return system_find(
            "7zz",
            hint = "Install via: sudo apt install 7zip  |  sudo dnf install 7zip",
        )

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for 7zip."""
    return "{vx_home}/store/7zip"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/7z.exe"
    else:
        # macOS and Linux both use 7zz
        return "{install_dir}/7zz"

def post_install(ctx, version, install_dir):
    """Post-install: on macOS create a 7z -> 7zz symlink for compatibility."""
    os = ctx["platform"]["os"]
    if os == "macos":
        return {
            "type": "symlink",
            "source": install_dir + "/7zz",
            "target": install_dir + "/7z",
        }
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {"PATH": install_dir}

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    return []
