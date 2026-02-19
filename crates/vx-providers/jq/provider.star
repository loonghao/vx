# provider.star - jq provider
#
# jq: Lightweight and flexible command-line JSON processor
# Inheritance pattern: Level 3 (custom download_url, non-standard naming)
#   - fetch_versions: inherited (tag_prefix "jq-" stripped)
#   - download_url:   custom (binary, platform-specific naming: jq-{os}-{arch})
#
# jq releases: https://github.com/jqlang/jq/releases
# Asset format: jq-{os}-{arch}[.exe]  (direct binary, no archive)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "jq"

def description():
    return "Lightweight and flexible command-line JSON processor"

def homepage():
    return "https://jqlang.github.io/jq/"

def repository():
    return "https://github.com/jqlang/jq"

def license():
    return "MIT"

def ecosystem():
    return "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "jq",
        "executable":  "jq",
        "description": "Command-line JSON processor",
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
# fetch_versions — inherited
#
# jq tags use "jq-" prefix: "jq-1.8.1" → version "1.8.1"
# We use a custom fetch that strips the "jq-" prefix.
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch jq versions from GitHub releases.

    jq uses tag format "jq-1.8.1" (not "v1.8.1"), so we strip the "jq-" prefix.
    """
    releases = ctx["http"]["get_json"]("https://api.github.com/repos/jqlang/jq/releases?per_page=30")
    versions = []
    for release in releases:
        if release.get("draft", False):
            continue
        tag = release.get("tag_name", "")
        # Strip "jq-" prefix
        if tag.startswith("jq-"):
            version = tag[3:]
        elif tag.startswith("v"):
            version = tag[1:]
        else:
            version = tag
        if version:
            versions.append({
                "version":    version,
                "lts":        not release.get("prerelease", False),
                "prerelease": release.get("prerelease", False),
            })
    return versions

# ---------------------------------------------------------------------------
# download_url — custom (binary, platform-specific naming)
#
# jq releases are direct binaries (not archives):
#   jq-windows-amd64.exe
#   jq-macos-amd64 / jq-macos-arm64
#   jq-linux-amd64 / jq-linux-arm64
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build the jq download URL.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "1.8.1"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    # Map arch to jq naming
    arch_map = {
        "x64":   "amd64",
        "arm64": "arm64",
        "x86":   "i386",
    }
    jq_arch = arch_map.get(arch)
    if not jq_arch:
        return None

    # Map OS to jq naming
    os_map = {
        "windows": "windows",
        "macos":   "macos",
        "linux":   "linux",
    }
    jq_os = os_map.get(os)
    if not jq_os:
        return None

    # Build asset name
    if os == "windows":
        asset = "jq-{}-{}.exe".format(jq_os, jq_arch)
    else:
        asset = "jq-{}-{}".format(jq_os, jq_arch)

    tag = "jq-{}".format(version)
    return github_asset_url("jqlang", "jq", tag, asset)

# ---------------------------------------------------------------------------
# install_layout — binary (single file)
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]
    exe = "jq.exe" if os == "windows" else "jq"
    return {
        "type":             "binary",
        "target_name":      exe,
        "target_dir":       "bin",
        "target_permissions": "755",
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir + "/bin",
    }
