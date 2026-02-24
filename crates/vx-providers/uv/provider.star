# provider.star - uv provider
#
# uv: An extremely fast Python package installer and resolver
# Inheritance pattern: Level 2 (standard Rust triple naming)
#   - fetch_versions: inherited from github.star
#   - download_url:   inherited via make_download_url (standard Rust triple)
#
# uv releases: https://github.com/astral-sh/uv/releases
# Asset format: uv-{triple}.{ext}

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:platform.star", "platform_triple", "platform_ext")
load("@vx//stdlib:install.star", "ensure_dependencies")

load("@vx//stdlib:env.star", "env_prepend")
# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

name        = "uv"
description = "An extremely fast Python package installer and resolver"
homepage    = "https://github.com/astral-sh/uv"
repository  = "https://github.com/astral-sh/uv"
license     = "MIT OR Apache-2.0"
ecosystem   = "python"

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
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "uv \\d+\\.\\d+"},
        ],
    },
    {
        "name":         "uvx",
        "executable":   "uvx",
        "description":  "Python application runner",
        "bundled_with": "uv",
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
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
# fetch_versions + download_url
#
# uv asset naming: uv-{triple}.{ext}
#   uv-x86_64-pc-windows-msvc.zip
#   uv-x86_64-apple-darwin.tar.gz
#   uv-aarch64-apple-darwin.tar.gz
#   uv-x86_64-unknown-linux-gnu.tar.gz
#
# IMPORTANT: uv release tags do NOT have a 'v' prefix (e.g. "0.10.5", not "v0.10.5")
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("astral-sh", "uv")

def download_url(ctx, version):
    """Build the uv download URL.

    uv uses tag format WITHOUT 'v' prefix: e.g. "0.10.5" not "v0.10.5"
    Asset format: uv-{triple}.{ext}
    """
    triple = platform_triple(ctx)
    if not triple:
        return None
    ext = platform_ext(ctx).lstrip(".")   # "zip" or "tar.gz"

    asset = "uv-{}.{}".format(triple, ext)
    # uv tags have NO 'v' prefix — use version directly as tag
    tag = version
    return github_asset_url("astral-sh", "uv", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os = ctx.platform.os
    exe = "uv.exe" if os == "windows" else "uv"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "uv"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for uv."""
    return ctx.vx_home + "/store/uv"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    exe = "uv.exe" if os == "windows" else "uv"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    """No post-install actions needed for uv."""
    return None

# ---------------------------------------------------------------------------
# pre_run — ensure uv sync before `uv run`
# ---------------------------------------------------------------------------

def pre_run(ctx, args, executable):
    """Ensure project dependencies are synced before running uv commands.

    For `uv run` commands, checks if pyproject.toml exists and .venv does not,
    then runs `uv sync` to install project dependencies.

    Args:
        ctx:        Provider context
        args:       Command-line arguments passed to uv
        executable: Path to the uv executable

    Returns:
        List of pre-run actions
    """
    if len(args) > 0 and args[0] == "run":
        return [
            ensure_dependencies(
                "uv",
                check_file  = "pyproject.toml",
                install_dir = ".venv",
            ),
        ]
    return []
