# provider.star - Azure CLI provider
#
# Linux: direct tar.gz from GitHub releases
# Windows/macOS: system package manager (MSI not supported)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "post_extract_permissions",
     "path_fns",
     "multi_platform_install", "winget_install", "choco_install",
     "brew_install")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star",    "env_prepend")

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
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "azure-cli"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["aka.ms"])

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("Azure", "azure-cli")

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

system_install = multi_platform_install(
    windows_strategies = [
        winget_install("Microsoft.AzureCLI", priority = 100),
        choco_install("azure-cli",            priority = 80),
    ],
    macos_strategies = [
        brew_install("azure-cli"),
    ],
)

# ---------------------------------------------------------------------------
# Path + env functions (from stdlib)
# ---------------------------------------------------------------------------

_paths           = path_fns("azcli", executable = "az")
store_root       = _paths["store_root"]
get_execute_path = _paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
