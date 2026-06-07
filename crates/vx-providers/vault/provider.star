# Vault Provider
# Vault is a tool for secrets management, encryption as a service, and privileged access management.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "archive_layout", "path_fns", "path_env_fns",
     "fetch_versions_from_api")
load("@vx//stdlib:system_install.star", "cross_platform_install")

# Provider metadata
name        = "vault"
description = "Vault - secrets management and encryption as a service"
homepage    = "https://www.vaultproject.io/"
repository  = "https://github.com/hashicorp/vault"
license     = "BUSL-1.1"
ecosystem   = "security"

# Runtime definitions
runtimes = [
    runtime_def("vault", aliases=["vault-cli"]),
]

# Permissions: needs GitHub releases access + package managers for system_install
permissions = github_permissions(
    extra_hosts = ["releases.hashicorp.com"],
    exec_cmds   = ["winget", "brew", "apt"],
)

fetch_versions = fetch_versions_from_api(
    "https://api.github.com/repos/hashicorp/vault/tags?per_page=100",
    "github_tags",
)

_VAULT_PLATFORMS = {
    "windows/x64":   ("windows", "amd64"),
    "windows/arm64": ("windows", "arm64"),
    "macos/x64":     ("darwin",  "amd64"),
    "macos/arm64":   ("darwin",  "arm64"),
    "linux/x64":     ("linux",   "amd64"),
    "linux/arm64":   ("linux",   "arm64"),
}

def _vault_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _VAULT_PLATFORMS.get(key)

def download_url(ctx, version):
    platform = _vault_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    asset = "vault_{}_{}_{}.zip".format(version, os_str, arch_str)
    return "https://releases.hashicorp.com/vault/{}/{}".format(version, asset)

install_layout   = archive_layout("vault")
paths            = path_fns("vault")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]
env_fns          = path_env_fns()
environment      = env_fns["environment"]
post_install     = env_fns["post_install"]

# system_install fallback when direct downloads are unavailable
system_install = cross_platform_install(
    windows = "HashiCorp.Vault",
    macos   = "vault",
    linux   = "vault",
)

# No additional dependencies
def deps(_ctx, _version):
    return []
