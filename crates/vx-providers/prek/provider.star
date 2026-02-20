# provider.star - prek provider
#
# prek: Better pre-commit, re-engineered in Rust
# Inheritance pattern: Level 3 (standard Rust triple naming)
#   - fetch_versions: inherited from github.star
#   - download_url:   inherited via make_github_provider (standard Rust triple)
#
# prek releases: https://github.com/j178/prek/releases
# Asset format: prek-{triple}.{ext}

load("@vx//stdlib:github.star", "make_github_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "prek"

def description():
    return "Better pre-commit, re-engineered in Rust"

def homepage():
    return "https://prek.j178.dev"

def repository():
    return "https://github.com/j178/prek"

def license():
    return "MIT"

def ecosystem():
    return "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "prek",
        "executable":  "prek",
        "description": "prek - better pre-commit framework",
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
# fetch_versions + download_url — inherited via make_github_provider
#
# prek asset naming: prek-{triple}.{ext}
#   prek-x86_64-pc-windows-msvc.zip
#   prek-x86_64-apple-darwin.tar.gz
#   prek-aarch64-apple-darwin.tar.gz
#   prek-x86_64-unknown-linux-musl.tar.gz
# ---------------------------------------------------------------------------

_p = make_github_provider("j178", "prek", "prek-{triple}.{ext}")
fetch_versions = _p["fetch_versions"]
download_url   = _p["download_url"]

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]
    exe = "prek.exe" if os == "windows" else "prek"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "prek"],
    }

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for prek."""
    return "{vx_home}/store/prek"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    exe = "prek.exe" if os == "windows" else "prek"
    return "{install_dir}/" + exe

def post_install(ctx, version, install_dir):
    """No post-install steps needed for prek."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
