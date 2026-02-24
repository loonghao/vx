# provider.star - Visual Studio Code provider
#
# Version source: VS Code update API
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "fetch_versions_from_api",
     "system_permissions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "vscode"
description = "Visual Studio Code - Code editing. Redefined."
homepage    = "https://code.visualstudio.com"
repository  = "https://github.com/microsoft/vscode"
license     = "MIT"
ecosystem   = "devtools"
aliases     = ["code", "vs-code"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("code",
        aliases = ["vscode", "vs-code"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    extra_hosts = ["update.code.visualstudio.com", "az764295.vo.msecnd.net"],
)

# ---------------------------------------------------------------------------
# fetch_versions — VS Code update API
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_api(
    "https://update.code.visualstudio.com/api/releases/stable",
    "vscode_releases",
)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_VSCODE_PLATFORMS = {
    "windows/x64":   ("win32-x64-archive",  "zip"),
    "windows/arm64": ("win32-arm64-archive", "zip"),
    "macos/x64":     ("darwin",              "zip"),
    "macos/arm64":   ("darwin-arm64",        "zip"),
    "linux/x64":     ("linux-x64",           "tar.gz"),
    "linux/arm64":   ("linux-arm64",         "tar.gz"),
    "linux/armv7":   ("linux-armhf",         "tar.gz"),
}

def download_url(ctx, version):
    platform = _VSCODE_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))
    if not platform:
        return None
    platform_id, _ext = platform[0], platform[1]
    return "https://update.code.visualstudio.com/{}/{}/stable".format(version, platform_id)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        exe_paths = ["bin/code.cmd", "Code.exe"]
    elif os == "macos":
        exe_paths = ["Visual Studio Code.app/Contents/Resources/app/bin/code"]
    else:
        exe_paths = ["bin/code"]
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/vscode"

def get_execute_path(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        return ctx.install_dir + "/bin/code.cmd"
    elif os == "macos":
        return ctx.install_dir + "/Visual Studio Code.app/Contents/Resources/app/bin/code"
    return ctx.install_dir + "/bin/code"

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    os = ctx.platform.os
    if os == "macos":
        return [env_prepend("PATH", ctx.install_dir + "/Visual Studio Code.app/Contents/Resources/app/bin")]
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []
