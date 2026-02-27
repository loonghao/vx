# provider.star - 7-Zip provider
#
# Windows: MSI from GitHub releases (ip7z/7zip)
# macOS/Linux: tar.xz from GitHub releases
# Tags: "24.09" (no 'v' prefix)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def",
     "system_permissions",
     "path_fns",
     "multi_platform_install", "winget_install", "choco_install",
     "brew_install", "apt_install")
load("@vx//stdlib:github.star", "github_releases", "releases_to_versions")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "7zip"
description = "7-Zip - High compression ratio file archiver"
homepage    = "https://www.7-zip.org"
repository  = "https://github.com/ip7z/7zip"
license     = "LGPL-2.1"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("7zip",
        aliases      = ["7z", "7za", "7zz"],
        system_paths = [
            "C:/Program Files/7-Zip",
            "C:/Program Files (x86)/7-Zip",
            "/usr/bin",
            "/usr/local/bin",
            "/opt/homebrew/bin",
        ],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    exec_cmds   = ["winget", "choco", "brew", "apt"],
    extra_hosts = ["api.github.com", "github.com"],
)

# ---------------------------------------------------------------------------
# fetch_versions — tags like "24.09" (no prefix)
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    releases = github_releases(ctx, owner = "ip7z", repo = "7zip",
                               include_prereleases = False)
    return releases_to_versions(releases)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def _ver_compact(version):
    return version.replace(".", "")

def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    ver  = _ver_compact(version)
    base = "https://github.com/ip7z/7zip/releases/download/{}".format(version)
    if os == "windows":
        return "{}/7z{}-x64.msi".format(base, ver) if arch == "x64" else "{}/7z{}.msi".format(base, ver)
    elif os == "macos":
        return "{}/7z{}-mac.tar.xz".format(base, ver)
    elif os == "linux":
        return "{}/7z{}-linux-arm64.tar.xz".format(base, ver) if arch == "arm64" else "{}/7z{}-linux-x64.tar.xz".format(base, ver)
    return None

# ---------------------------------------------------------------------------
# install_layout
# Note: Windows uses MSI which is not supported by vx-installer.
# Users should use system_install on Windows.
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        # MSI is not supported; this will fail at install time
        # Users should use system_install instead
        return {
            "type":             "msi",
            "executable_paths": ["7z.exe"],
            "strip_prefix":     "PFiles\\7-Zip",
        }
    exe = "7zz"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "7z"],
    }

# ---------------------------------------------------------------------------
# system_install
# ---------------------------------------------------------------------------

system_install = multi_platform_install(
    windows_strategies = [
        winget_install("7zip.7zip", priority = 90),
        choco_install("7zip",       priority = 80),
    ],
    macos_strategies = [
        brew_install("sevenzip"),
    ],
    linux_strategies = [
        brew_install("sevenzip", priority = 70),
        apt_install("p7zip-full", priority = 70),
    ],
)

# ---------------------------------------------------------------------------
# Path queries + environment
# Note: 7zip has special executable names:
#   - Windows: 7z.exe
#   - Unix: 7zz (create symlink 7z -> 7zz on macOS)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/7zip"

def get_execute_path(ctx, _version):
    if ctx.platform.os == "windows":
        return ctx.install_dir + "/7z.exe"
    return ctx.install_dir + "/7zz"

def post_install(ctx, _version):
    if ctx.platform.os == "macos":
        return {"type": "symlink", "source": ctx.install_dir + "/7zz",
                "target": ctx.install_dir + "/7z"}
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
