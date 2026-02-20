# provider.star - Task (go-task) provider
#
# Reuse level: Level 2 — fetch_versions inherited, download_url overridden
#
# Why override download_url?
#   - Uses Go-style platform naming: {os}_{arch} (darwin/linux/windows + amd64/arm64/arm/386)
#   - Asset: "task_{os}_{arch}.{ext}"  (not Rust triple)
#   - URL:   ".../v{version}/task_{os}_{arch}.{ext}"
#
# Equivalent Rust: TaskUrlBuilder (config.rs, ~128 lines) → ~40 lines Starlark

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "task"

def description():
    return "Task - A task runner / simpler Make alternative written in Go"

def homepage():
    return "https://taskfile.dev"

def repository():
    return "https://github.com/go-task/task"

def license():
    return "MIT"

def ecosystem():
    return "devtools"

def aliases():
    return ["go-task", "taskfile"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "task",
        "executable":  "task",
        "description": "Task runner / simpler Make alternative",
        "aliases":     ["go-task"],
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

fetch_versions = make_fetch_versions("go-task", "task", include_prereleases = False)

# ---------------------------------------------------------------------------
# download_url — custom override (Go-style platform naming)
#
# OS mapping:   windows → windows, macos → darwin, linux → linux
# Arch mapping: x64 → amd64, arm64 → arm64, arm → arm, x86 → 386
# Asset:        "task_{os}_{arch}.{ext}"
# ---------------------------------------------------------------------------

def _task_platform(ctx):
    """Map vx platform to task's Go-style os/arch pair."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    os_map = {
        "windows": "windows",
        "macos":   "darwin",
        "linux":   "linux",
    }
    arch_map = {
        "x64":   "amd64",
        "arm64": "arm64",
        "arm":   "arm",
        "x86":   "386",
    }

    go_os   = os_map.get(os)
    go_arch = arch_map.get(arch)

    if not go_os or not go_arch:
        return None, None
    return go_os, go_arch

def download_url(ctx, version):
    """Build the task download URL.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "3.40.1"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    go_os, go_arch = _task_platform(ctx)
    if not go_os:
        return None

    os  = ctx["platform"]["os"]
    ext = "zip" if os == "windows" else "tar.gz"

    # Asset: "task_linux_amd64.tar.gz"
    asset = "task_{}_{}.{}".format(go_os, go_arch, ext)
    tag   = "v{}".format(version)

    return github_asset_url("go-task", "task", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os  = ctx["platform"]["os"]
    exe = "task.exe" if os == "windows" else "task"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "task"],
    }

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for task."""
    return "{vx_home}/store/task"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    exe = "task.exe" if os == "windows" else "task"
    return "{install_dir}/" + exe

def post_install(ctx, version, install_dir):
    """No post-install actions needed for task."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
