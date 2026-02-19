# provider.star - 7-Zip provider
#
# 7-Zip is a file archiver with a high compression ratio.
# Tags are date-based like "24.09" (no "v" prefix).
#
# Installation strategy:
#   Windows: MSI install (7z{compact}-x64.msi) via msiexec /a
#   macOS:   tar.xz archive (7z{compact}-mac.tar.xz)
#   Linux:   tar.xz archive (7z{compact}-linux-{arch}.tar.xz)
#
# Fallback (system package managers):
#   Windows: winget (7zip.7zip) or choco (7zip)
#   macOS:   brew (sevenzip)
#   Linux:   brew or apt (p7zip-full)
#
# Inheritance pattern: Level 2 (custom download_url + install_layout)

load("@vx//stdlib:install.star", "msi_install", "archive_install")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "7zip"

def description():
    return "7-Zip file archiver with high compression ratio"

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
# fetch_versions — GitHub releases (tags like "24.09", no "v" prefix)
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch 7-Zip versions from GitHub releases (ip7z/7zip)."""
    releases = ctx["http"]["get_json"](
        "https://api.github.com/repos/ip7z/7zip/releases?per_page=20"
    )
    versions = []
    for release in releases:
        if release.get("draft") or release.get("prerelease"):
            continue
        tag = release.get("tag_name", "")
        if tag:
            versions.append({
                "version":    tag,
                "lts":        False,
                "prerelease": False,
            })
    return versions

# ---------------------------------------------------------------------------
# _compact_version — "24.09" -> "2409"
# ---------------------------------------------------------------------------

def _compact_version(version):
    """Convert dotted version to compact form: '24.09' -> '2409'."""
    return version.replace(".", "")

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Return the download URL for the given platform.

    Windows: MSI installer (7z{compact}-x64.msi)
    macOS:   Universal tar.xz (7z{compact}-mac.tar.xz)
    Linux:   Platform-specific tar.xz
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]
    ver  = _compact_version(version)
    base = "https://github.com/ip7z/7zip/releases/download/{}/".format(version)

    if os == "windows":
        if arch == "x64":
            return base + "7z{}-x64.msi".format(ver)
        elif arch == "x86":
            return base + "7z{}.msi".format(ver)
        # arm64 has no MSI, fall back to system install
        return None
    elif os == "macos":
        # Universal binary (supports both x64 and arm64)
        return base + "7z{}-mac.tar.xz".format(ver)
    elif os == "linux":
        if arch == "x64":
            return base + "7z{}-linux-x64.tar.xz".format(ver)
        elif arch == "arm64":
            return base + "7z{}-linux-arm64.tar.xz".format(ver)
        elif arch == "arm":
            return base + "7z{}-linux-arm.tar.xz".format(ver)
        return None

    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Return the install layout descriptor for the Rust runtime.

    Windows: MSI install via msiexec /a (no registry changes)
    macOS/Linux: Archive extraction
    """
    os  = ctx["platform"]["os"]
    url = download_url(ctx, version)

    if url == None:
        return None

    if os == "windows":
        # msiexec /a extracts to TARGETDIR; 7-Zip MSI places files in root
        return msi_install(
            url,
            executable_paths = ["7z.exe", "7za.exe", "7zr.exe"],
        )
    else:
        # Linux and macOS: tar.xz with executables in root (no subdirectory)
        exe = "7zz" if os == "macos" else "7zzs"
        return archive_install(
            url,
            strip_prefix = "",
            executable_paths = [exe, "7z", "7za", "7zr"],
        )

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
