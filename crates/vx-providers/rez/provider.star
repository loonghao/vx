# provider.star - rez provider
#
# Rez: Cross-platform package manager for deterministic environments
# Inheritance pattern: Level 3 (package alias - runs via uvx)
#   - fetch_versions: inherited from github.star
#   - download_url:   None (installed via uvx, not direct download)
#   - deps:           requires python
#
# Rez is a Python tool, installed via `uvx rez` or `pip install rez`

load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "rez"

def description():
    return "Cross-platform package manager for deterministic environments"

def homepage():
    return "https://rez.readthedocs.io"

def repository():
    return "https://github.com/AcademySoftwareFoundation/rez"

def license():
    return "Apache-2.0"

def ecosystem():
    return "python"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "rez",
        "executable":  "rez",
        "description": "Rez package manager",
        "aliases":     [],
        "priority":    100,
    },
    {
        "name":         "rez-env",
        "executable":   "rez-env",
        "description":  "Rez environment resolver",
        "bundled_with": "rez",
    },
    {
        "name":         "rez-build",
        "executable":   "rez-build",
        "description":  "Rez package builder",
        "bundled_with": "rez",
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
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("AcademySoftwareFoundation", "rez")

# ---------------------------------------------------------------------------
# download_url — None (installed via uvx/pip)
#
# Rez is a Python package, not a direct binary download.
# It should be installed via: uvx rez or pip install rez
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Rez is installed via uvx/pip, not direct download."""
    return None

# ---------------------------------------------------------------------------
# deps — requires python
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """Rez requires Python 3.8+."""
    return [{"runtime": "python", "version": ">=3.8"}]

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for rez."""
    return "{vx_home}/store/rez"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    exe = "rez.exe" if os == "windows" else "rez"
    return "{install_dir}/" + exe

def post_install(ctx, version, install_dir):
    """No post-install steps needed for rez."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {}
