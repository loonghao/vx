# provider.star - Node.js provider
#
# Version source: https://nodejs.org/dist/index.json (official API, no rate limiting)
# Bundled runtimes: npm, npx (included in every Node.js release)
#
# Inheritance pattern: Level 1 (fully custom - uses nodejs.org API, not GitHub)
#
# Node.js releases: https://nodejs.org/en/download/releases

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "node"

def description():
    return "Node.js - JavaScript runtime built on Chrome's V8 engine"

def homepage():
    return "https://nodejs.org"

def repository():
    return "https://github.com/nodejs/node"

def license():
    return "MIT"

def ecosystem():
    return "nodejs"

def aliases():
    return ["nodejs"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "node",
        "executable":  "node",
        "description": "Node.js JavaScript runtime",
        "aliases":     ["nodejs"],
        "priority":    100,
    },
    {
        "name":        "npm",
        "executable":  "npm",
        "description": "Node Package Manager (bundled with Node.js)",
        "bundled_with": "node",
    },
    {
        "name":        "npx",
        "executable":  "npx",
        "description": "Node Package Execute (bundled with Node.js)",
        "bundled_with": "node",
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["nodejs.org"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — uses nodejs.org official API (no GitHub rate limiting)
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch Node.js versions from the official nodejs.org API.

    Uses https://nodejs.org/dist/index.json which provides:
    - Full version list with LTS status
    - No rate limiting (unlike GitHub API)
    - Official release metadata
    """
    releases = ctx["http"]["get_json"]("https://nodejs.org/dist/index.json")

    versions = []
    for release in releases:
        v = release["version"]
        # Strip leading 'v': "v20.0.0" -> "20.0.0"
        if v.startswith("v"):
            v = v[1:]

        lts = release.get("lts", False)
        # lts field is either False or a codename string like "Iron"
        is_lts = lts != False and lts != None

        versions.append({
            "version":    v,
            "lts":        is_lts,
            "prerelease": False,
        })

    return versions

# ---------------------------------------------------------------------------
# download_url — nodejs.org official download
# ---------------------------------------------------------------------------

def _node_platform(ctx):
    """Map vx platform to Node.js platform string."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    platforms = {
        "windows/x64":   ("win",    "x64"),
        "windows/x86":   ("win",    "x86"),
        "macos/x64":     ("darwin", "x64"),
        "macos/arm64":   ("darwin", "arm64"),
        "linux/x64":     ("linux",  "x64"),
        "linux/arm64":   ("linux",  "arm64"),
        "linux/armv7":   ("linux",  "armv7l"),
    }
    key = "{}/{}".format(os, arch)
    return platforms.get(key)

def download_url(ctx, version):
    """Build the Node.js download URL from nodejs.org.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "20.11.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _node_platform(ctx)
    if not platform:
        return None

    os_str, arch_str = platform[0], platform[1]
    os = ctx["platform"]["os"]

    if os == "windows":
        # Windows: zip archive
        # e.g. https://nodejs.org/dist/v20.11.0/node-v20.11.0-win-x64.zip
        filename = "node-v{}-{}-{}.zip".format(version, os_str, arch_str)
    else:
        # Unix: tar.xz archive
        # e.g. https://nodejs.org/dist/v20.11.0/node-v20.11.0-linux-x64.tar.xz
        filename = "node-v{}-{}-{}.tar.xz".format(version, os_str, arch_str)

    return "https://nodejs.org/dist/v{}/{}".format(version, filename)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    platform = _node_platform(ctx)
    os = ctx["platform"]["os"]

    if not platform:
        return {"type": "archive", "strip_prefix": "", "executable_paths": ["node"]}

    os_str, arch_str = platform[0], platform[1]

    if os == "windows":
        # Windows Node.js has flat layout: node-v20.11.0-win-x64/node.exe
        strip_prefix = "node-v{}-{}-{}".format(version, os_str, arch_str)
        exe_paths = ["node.exe", "npm.cmd", "npx.cmd"]
    else:
        # Unix Node.js has bin/ layout: node-v20.11.0-linux-x64/bin/node
        strip_prefix = "node-v{}-{}-{}".format(version, os_str, arch_str)
        exe_paths = ["bin/node", "bin/npm", "bin/npx"]

    return {
        "type":             "archive",
        "strip_prefix":     strip_prefix,
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    os = ctx["platform"]["os"]
    if os == "windows":
        # Windows: executables are in root dir
        return {"PATH": install_dir}
    else:
        # Unix: executables are in bin/
        return {"PATH": install_dir + "/bin"}

# ---------------------------------------------------------------------------
# deps — explicit dependency declarations (Buck2 style)
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """Node.js has no external dependencies."""
    return []
