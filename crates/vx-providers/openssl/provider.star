# provider.star - openssl provider
#
# OpenSSL is pre-installed on most systems.
# vx only detects the system installation — no download managed.
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star", "runtime_def", "system_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "openssl"
description = "Cryptography and SSL/TLS toolkit"
homepage    = "https://www.openssl.org"
repository  = "https://github.com/openssl/openssl"
license     = "Apache-2.0"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("openssl",
        system_paths = [
            "C:/Program Files/Git/mingw64/bin/openssl.exe",
            "C:/Program Files/Git/usr/bin/openssl.exe",
            "C:/ProgramData/chocolatey/lib/openssl/tools/openssl/bin/openssl.exe",
            "C:/OpenSSL-Win64/bin/openssl.exe",
            "C:/OpenSSL-Win32/bin/openssl.exe",
            "C:/msys64/usr/bin/openssl.exe",
            "/opt/homebrew/bin/openssl",
            "/usr/local/bin/openssl",
            "/usr/bin/openssl",
            "/bin/openssl",
        ],
        test_commands = [
            {"command": "{executable} version", "name": "version_check",
             "expected_output": "OpenSSL \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(exec_cmds = ["openssl"])

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
    return ctx.vx_home + "/store/openssl"

def get_execute_path(_ctx, _version):
    return None

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

def deps(_ctx, _version):
    return []
