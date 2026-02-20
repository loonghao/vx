# provider.star - Docker provider
#
# Version source: https://github.com/docker/cli/releases
#
# Docker CLI binary download is only available for Linux and macOS.
# Windows requires Docker Desktop (system install only).
#
# Inheritance pattern: Level 2 (custom download_url for platform-specific asset naming)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "docker"

def description():
    return "Docker - Container platform for building, sharing, and running applications"

def homepage():
    return "https://www.docker.com"

def repository():
    return "https://github.com/docker/cli"

def license():
    return "Apache-2.0"

def ecosystem():
    return "container"

def aliases():
    return []

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "docker",
        "executable":  "docker",
        "description": "Docker CLI",
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "download.docker.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — docker/cli GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("docker", "cli")

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def _docker_asset(ctx, version):
    """Map vx platform to Docker CLI asset name."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    arch_map = {
        "x64":   "x86_64",
        "arm64": "aarch64",
        "armv7": "armv7",
    }
    arch_str = arch_map.get(arch)
    if not arch_str:
        return None

    if os == "linux":
        # e.g. docker-27.5.1.tgz (Linux static binary)
        return "docker-{}.tgz".format(version)
    elif os == "macos":
        # macOS uses the same static binary format
        return "docker-{}.tgz".format(version)
    else:
        # Windows: no direct binary download
        return None

def download_url(ctx, version):
    """Build Docker CLI download URL.

    Linux/macOS: static binary from download.docker.com
    Windows: not supported (use Docker Desktop)
    """
    os = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    arch_map = {
        "x64":   "x86_64",
        "arm64": "aarch64",
        "armv7": "armv7",
    }
    arch_str = arch_map.get(arch)
    if not arch_str:
        return None

    if os == "linux":
        # https://download.docker.com/linux/static/stable/x86_64/docker-27.5.1.tgz
        return "https://download.docker.com/linux/static/stable/{}/docker-{}.tgz".format(
            arch_str, version
        )
    elif os == "macos":
        # macOS uses the same static binary
        return "https://download.docker.com/mac/static/stable/{}/docker-{}.tgz".format(
            arch_str, version
        )
    else:
        # Windows: no direct binary, use Docker Desktop
        return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    return {
        "type":             "archive",
        "strip_prefix":     "docker",
        "executable_paths": ["docker"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {"PATH": install_dir}

# ---------------------------------------------------------------------------
# system_install — Windows uses Docker Desktop
# ---------------------------------------------------------------------------

def system_install(ctx):
    os = ctx["platform"]["os"]
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "Docker.DockerDesktop", "priority": 90},
                {"manager": "choco",  "package": "docker-desktop",        "priority": 80},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "docker", "priority": 80},
            ],
        }
    return {}

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    return []


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return "{vx_home}/store/docker"

def get_execute_path(ctx, version):
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/docker.exe"
    else:
        return "{install_dir}/docker"

def post_install(ctx, version, install_dir):
    return None
