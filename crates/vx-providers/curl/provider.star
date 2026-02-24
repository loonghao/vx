# provider.star - curl provider
#
# cURL is pre-installed on most modern systems (Windows 10+, macOS, Linux).
# vx only detects the system installation — no download managed.
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star", "runtime_def", "system_permissions", "system_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "curl"
description = "Command-line tool for transferring data with URLs"
homepage    = "https://curl.se"
repository  = "https://github.com/curl/curl"
license     = "MIT"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("curl",
        system_paths = [
            "C:/Windows/System32/curl.exe",
            "C:/Program Files/Git/mingw64/bin/curl.exe",
            "C:/msys64/usr/bin/curl.exe",
            "/usr/bin/curl",
            "/usr/local/bin/curl",
            "/opt/homebrew/bin/curl",
        ],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "curl \\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(exec_cmds = ["curl"])

# ---------------------------------------------------------------------------
# fetch_versions — system detection only
# ---------------------------------------------------------------------------

def fetch_versions(_ctx):
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — system tool, not managed by vx
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/curl"

def get_execute_path(ctx, _version):
    exe = "curl.exe" if ctx.platform.os == "windows" else "curl"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

def deps(_ctx, _version):
    return []
