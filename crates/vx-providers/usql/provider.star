load("@vx//stdlib:system_install.star", "cross_platform_install")
# provider.star - usql provider
#
# usql is a universal command-line interface for SQL databases.
# Supports PostgreSQL, MySQL, Oracle, SQLite, MSSQL, and many more.
#
# Release assets (GitHub releases):
#   usql-{version}-{os}-{arch}.tar.bz2  (Linux/macOS)
#   usql-{version}-{os}-{arch}.zip      (Windows)
#
# NOTE: uses dash-separated names with lowercase Go GOOS/GOARCH,
# and .tar.bz2 (not .tar.gz) for Linux/macOS.
#
# OS:   linux, darwin, windows
# Arch: amd64, arm64
#
# Version source: xo/usql releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "path_fns")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "usql"
description = "usql - Universal command-line interface for SQL databases"
homepage    = "https://github.com/xo/usql"
repository  = "https://github.com/xo/usql"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("usql",
        version_cmd     = "{executable} --version",
        version_pattern = "usql \\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("xo", "usql")

# ---------------------------------------------------------------------------
# Platform helpers
#
# usql uses dash-separated format with standard Go GOOS/GOARCH names.
# Linux/macOS: .tar.bz2; Windows: .zip
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "linux/x64":    ("linux",   "amd64", "tar.bz2"),
    "linux/arm64":  ("linux",   "arm64", "tar.bz2"),
    "macos/x64":    ("darwin",  "amd64", "tar.bz2"),
    "macos/arm64":  ("darwin",  "arm64", "tar.bz2"),
    "windows/x64":  ("windows", "amd64", "zip"),
}

def _usql_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _usql_platform(ctx)
    if not platform:
        return None
    os_name, arch_name, ext = platform[0], platform[1], platform[2]
    asset = "usql-{}-{}-{}.{}".format(version, os_name, arch_name, ext)
    return github_asset_url("vx-org", "mirrors", "usql-" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "usql.exe" if ctx.platform.os == "windows" else "usql"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("usql")
store_root       = paths["store_root"]

def get_execute_path(ctx, _version):
    exe = "usql.exe" if ctx.platform.os == "windows" else "usql"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []

system_install = cross_platform_install(
    windows = "usql",
    macos   = "usql",
    linux   = "usql",
)
