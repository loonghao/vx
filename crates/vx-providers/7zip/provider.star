# provider.star - 7-Zip provider
#
# Windows prefers system package managers.
# macOS/Linux use archives from GitHub releases.
# Tags: "24.09" (no 'v' prefix)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def",
     "system_permissions",
     "system_install_strategies", "winget_install", "choco_install",
     "brew_install", "apt_install")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

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
        executable   = "7z",
        aliases      = ["7z", "7za", "7zz"],
        system_paths = [
            "C:/Program Files/7-Zip/7z.exe",
            "C:/Program Files (x86)/7-Zip/7z.exe",
            "/usr/bin/7zz",
            "/usr/bin/7z",
            "/usr/local/bin/7zz",
            "/usr/local/bin/7z",
            "/opt/homebrew/bin/7zz",
            "/opt/homebrew/bin/7z",
        ],
        test_commands = [
            {"command": "{executable} -h", "name": "version_check", "expected_output": "7-Zip"},
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

fetch_versions = make_fetch_versions("vx-org", "mirrors", tag_prefix = "7zip-")

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def _ver_compact(version):
    return version.replace(".", "")


def download_url(ctx, version):
    os = ctx.platform.os
    if os == "windows":
        return None
    arch = ctx.platform.arch
    ver  = _ver_compact(version)
    if os == "macos":
        asset = "7z{}-mac.tar.xz".format(ver)
    elif os == "linux":
        asset = "7z{}-linux-arm64.tar.xz".format(ver) if arch == "arm64" else "7z{}-linux-x64.tar.xz".format(ver)
    else:
        return None
    return github_asset_url("vx-org", "mirrors", "7zip-" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["7zz", "7z"],
    }

# ---------------------------------------------------------------------------
# system_install
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    winget_install("7zip.7zip", priority = 90),
    choco_install("7zip",       priority = 80),
    brew_install("sevenzip",    priority = 70),
    apt_install("p7zip-full",   priority = 70),
])

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/7zip"


def get_execute_path(ctx, _version):
    if ctx.platform.os == "windows":
        return ctx.install_dir + "/bin/7z.exe"
    return ctx.install_dir + "/7zz"


def post_install(ctx, _version):
    if ctx.platform.os == "macos":
        return {"type": "symlink", "source": ctx.install_dir + "/7zz",
                "target": ctx.install_dir + "/7z"}
    return None


def environment(ctx, _version):
    if ctx.platform.os == "windows":
        return [env_prepend("PATH", ctx.install_dir + "/bin")]
    return [env_prepend("PATH", ctx.install_dir)]


def deps(_ctx, _version):
    return []
