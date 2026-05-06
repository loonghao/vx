# provider.star - Azure CLI provider
#
# Linux: direct tar.gz from GitHub releases
# Windows/macOS: system package manager preferred
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "post_extract_permissions",
     "system_install_strategies", "winget_install", "choco_install",
     "brew_install")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "azcli"
description = "Azure CLI - Command-line interface for Microsoft Azure"
homepage    = "https://docs.microsoft.com/cli/azure/"
repository  = "https://github.com/Azure/azure-cli"
license     = "MIT"
ecosystem   = "cloud"
aliases     = ["azure-cli"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("az",
        aliases = ["azcli", "azure-cli"],
        system_paths = [
            "C:/Program Files (x86)/Microsoft SDKs/Azure/CLI2/wbin/az.cmd",
            "C:/Program Files/Microsoft SDKs/Azure/CLI2/wbin/az.cmd",
            "/usr/local/bin/az",
            "/usr/bin/az",
        ],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "azure-cli"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(
    extra_hosts = ["aka.ms"],
    exec_cmds   = ["winget", "choco", "brew"],
)

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("vx-org", "mirrors", tag_prefix = "azcli-")

# ---------------------------------------------------------------------------
# download_url — Linux only; Windows/macOS use system_install
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    if ctx.platform.os != "linux":
        return None
    return "https://github.com/Azure/azure-cli/releases/download/{}/azure-cli-{}.tar.gz".format(
        version, version)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["bin/az"],
    }

# ---------------------------------------------------------------------------
# post_extract — set +x on Linux
# ---------------------------------------------------------------------------

post_extract = post_extract_permissions(["bin/az"])

# ---------------------------------------------------------------------------
# system_install — preferred on Windows and macOS
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    winget_install("Microsoft.AzureCLI", priority = 100),
    choco_install("azure-cli", priority = 80),
    brew_install("azure-cli", priority = 70),
])

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/azcli"


def get_execute_path(ctx, _version):
    if ctx.platform.os == "windows":
        return ctx.install_dir + "/bin/az.cmd"
    return ctx.install_dir + "/bin/az"


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]


def post_install(_ctx, _version):
    return None


def deps(_ctx, _version):
    return []
