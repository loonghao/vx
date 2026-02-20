# provider.star - make provider
#
# GNU Make - build automation tool (Unix-only, system detection + package manager install)
# Inheritance pattern: Level 1 (fully custom, system tool, Unix-only)
#
# Windows is NOT supported via vx - use 'just' as a modern alternative.
# On macOS/Linux, make is typically pre-installed or available via system package manager.

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "make"

def description():
    return "GNU Make - A build automation tool (Unix only, use 'just' on Windows)"

def homepage():
    return "https://www.gnu.org/software/make/"

def license():
    return "GPL-3.0"

def ecosystem():
    return "system"

# ---------------------------------------------------------------------------
# Platform constraint: Unix-only
# ---------------------------------------------------------------------------

def supported_platforms():
    return [
        {"os": "linux",  "arch": "x64"},
        {"os": "linux",  "arch": "arm64"},
        {"os": "macos",  "arch": "x64"},
        {"os": "macos",  "arch": "arm64"},
    ]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "make",
        "executable":  "make",
        "description": "GNU Make build automation tool",
        "aliases":     ["gmake", "gnumake"],
        "priority":    100,
        "system_paths": [
            "/usr/bin/make",
            "/usr/local/bin/make",
            "/opt/homebrew/bin/make",
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": [],
    "fs":   ["/usr/bin", "/usr/local/bin", "/opt/homebrew/bin"],
    "exec": ["make", "brew", "apt", "dnf", "pacman"],
}

# ---------------------------------------------------------------------------
# fetch_versions — static list (system package manager managed)
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Return known make versions (installed via system package manager)."""
    os = ctx["platform"]["os"]
    if os == "windows":
        return []
    return [
        {"version": "4.4.1", "lts": True,  "prerelease": False},
        {"version": "4.4",   "lts": True,  "prerelease": False},
        {"version": "4.3",   "lts": False, "prerelease": False},
        {"version": "4.2.1", "lts": False, "prerelease": False},
        {"version": "4.2",   "lts": False, "prerelease": False},
        {"version": "4.1",   "lts": False, "prerelease": False},
        {"version": "4.0",   "lts": False, "prerelease": False},
        {"version": "3.82",  "lts": False, "prerelease": False},
        {"version": "3.81",  "lts": False, "prerelease": False},
    ]

# ---------------------------------------------------------------------------
# download_url — not directly downloadable (use system package manager)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """make is installed via system package manager, not direct download."""
    return None

# ---------------------------------------------------------------------------
# system_install — package manager strategies
# ---------------------------------------------------------------------------

def system_install(ctx):
    """Return system install strategies for make."""
    os = ctx["platform"]["os"]
    if os == "windows":
        return []
    return [
        {"manager": "brew",   "package": "make",  "priority": 90},
        {"manager": "apt",    "package": "make",  "priority": 90},
        {"manager": "dnf",    "package": "make",  "priority": 90},
        {"manager": "pacman", "package": "make",  "priority": 90},
    ]

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for make."""
    return "{vx_home}/store/make"

def get_execute_path(ctx, version):
    """Return the executable path for the given version (system tool)."""
    return "{install_dir}/make"

def post_install(ctx, version, install_dir):
    """No post-install steps needed for make."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {}
