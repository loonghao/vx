# provider.star - Podman CLI provider
#
# Podman is primarily installed via system package managers.
# vx detects an existing system installation and can suggest platform-native
# installation strategies when Podman is missing.

load("@vx//stdlib:provider.star",
     "runtime_def", "system_permissions",
     "system_install_strategies", "winget_install", "brew_install", "apt_install", "dnf_install")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "podman"
description = "Podman CLI - Daemonless container engine and container tooling"
homepage    = "https://podman.io"
repository  = "https://github.com/containers/podman"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("podman",
        system_paths = [
            "C:/Program Files/RedHat/Podman/podman.exe",
            "C:/Program Files/Podman/podman.exe",
            "/usr/bin/podman",
            "/usr/local/bin/podman",
            "/opt/homebrew/bin/podman",
        ],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "podman version"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(exec_cmds = ["winget", "brew", "apt", "dnf"])

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
# system_install
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    winget_install("RedHat.Podman", priority = 90),
    brew_install("podman",          priority = 80),
    apt_install("podman",           priority = 70),
    dnf_install("podman",           priority = 60),
])

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/podman"


def get_execute_path(ctx, _version):
    exe = "podman.exe" if ctx.platform.os == "windows" else "podman"
    return ctx.install_dir + "/" + exe


def post_install(_ctx, _version):
    return None


def environment(_ctx, _version):
    return []


def deps(_ctx, _version):
    return []
