# provider.star - Docker provider
#
# Linux/macOS: static binary from download.docker.com
# Windows: Docker Desktop via system package manager
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "multi_platform_install", "winget_install", "choco_install",
     "brew_install")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "docker"
description = "Docker - Container platform for building, sharing, and running applications"
homepage    = "https://www.docker.com"
repository  = "https://github.com/docker/cli"
license     = "Apache-2.0"
ecosystem   = "container"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("docker",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "Docker version"},
            {"command": "{executable} info", "name": "daemon_check",
             "expect_success": True},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["download.docker.com"])

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("docker", "cli")

# ---------------------------------------------------------------------------
# download_url — Linux/macOS static binary; Windows uses system_install
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
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/docker"

def get_execute_path(ctx, _version):
    exe = "docker.exe" if ctx.platform.os == "windows" else "docker"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
