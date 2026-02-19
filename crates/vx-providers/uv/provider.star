# provider.star - uv provider
#
# uv: An extremely fast Python package installer and resolver
# Inheritance pattern: Level 2 (standard Rust triple naming)
#   - fetch_versions: inherited from github.star
#   - download_url:   inherited via make_download_url (standard Rust triple)
#
# uv releases: https://github.com/astral-sh/uv/releases
# Asset format: uv-{triple}.{ext}

load("@vx//stdlib:github.star", "make_github_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "uv"

def description():
    return "An extremely fast Python package installer and resolver"

def homepage():
    return "https://github.com/astral-sh/uv"

def repository():
    return "https://github.com/astral-sh/uv"

def license():
    return "MIT OR Apache-2.0"

def ecosystem():
    return "python"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "uv",
        "executable":  "uv",
        "description": "Extremely fast Python package installer",
        "aliases":     [],
        "priority":    100,
    },
    {
        "name":         "uvx",
        "executable":   "uvx",
        "description":  "Python application runner",
        "bundled_with": "uv",
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
# fetch_versions + download_url — inherited via make_github_provider
#
# uv asset naming: uv-{triple}.{ext}
#   uv-x86_64-pc-windows-msvc.zip
#   uv-x86_64-apple-darwin.tar.gz
#   uv-aarch64-apple-darwin.tar.gz
#   uv-x86_64-unknown-linux-gnu.tar.gz
# ---------------------------------------------------------------------------

_p = make_github_provider("astral-sh", "uv", "uv-{triple}.{ext}")
fetch_versions = _p["fetch_versions"]
download_url   = _p["download_url"]

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]
    exe = "uv.exe" if os == "windows" else "uv"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "uv"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
