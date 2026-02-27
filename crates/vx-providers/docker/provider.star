# provider.star - Docker CLI provider
#
# Docker CLI only (not Docker Engine)
# Downloads from docker.com static binaries
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "archive_layout", "path_fns", "path_env_fns",
     "multi_platform_install", "winget_install", "choco_install",
     "brew_install")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "docker"
description = "Docker CLI - The command-line interface for Docker"
homepage    = "https://www.docker.com"
repository  = "https://github.com/docker/cli"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("docker",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "Docker version"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform helpers
# Docker uses static downloads from download.docker.com
# ---------------------------------------------------------------------------

_DOCKER_ARCH = {"x64": "x86_64", "arm64": "aarch64", "armv7": "armv7"}

def download_url(ctx, version):
    arch_str = _DOCKER_ARCH.get(ctx.platform.arch)
    if not arch_str:
        return None
    os = ctx.platform.os
    if os == "linux":
        return "https://download.docker.com/linux/static/stable/{}/docker-{}.tgz".format(
            arch_str, version)
    elif os == "macos":
        return "https://download.docker.com/mac/static/stable/{}/docker-{}.tgz".format(
            arch_str, version)
    return None

# ---------------------------------------------------------------------------
# install_layout — strip top-level "docker/" dir
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    return {
        "type":             "archive",
        "strip_prefix":     "docker",
        "executable_paths": ["docker"],
    }

# ---------------------------------------------------------------------------
# system_install — Windows/macOS Docker Desktop
# ---------------------------------------------------------------------------

system_install = multi_platform_install(
    windows_strategies = [
        winget_install("Docker.DockerDesktop", priority = 90),
        choco_install("docker-desktop",         priority = 80),
    ],
    macos_strategies = [
        brew_install("docker", priority = 80),
    ],
)

# ---------------------------------------------------------------------------
# Path + env functions (from stdlib)
# ---------------------------------------------------------------------------

_paths           = path_fns("docker")
store_root       = _paths["store_root"]
get_execute_path = _paths["get_execute_path"]

_env             = path_env_fns()
post_install     = _env["post_install"]
environment      = _env["environment"]

def deps(_ctx, _version):
    return []
