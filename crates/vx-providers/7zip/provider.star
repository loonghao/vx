# provider.star - 7-Zip provider
#
# 7-Zip is a file archiver with a high compression ratio.
# Tags are date-based like "24.09" (no "v" prefix).
#
# Installation strategy:
#   Windows: winget (7zip.7zip) or choco (7zip)
#   macOS:   brew (sevenzip)
#   Linux:   brew or apt (p7zip-full)
#
# Inheritance pattern: Level 1 (system install via package managers)

load("@vx//stdlib:github.star", "make_fetch_versions")

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
    "http": ["api.github.com"],
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
# download_url — system install only, no direct binary download
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """7-Zip is installed via system package managers."""
    return None

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    return []
