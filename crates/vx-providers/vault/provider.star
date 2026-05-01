# Vault Provider
# Vault is a tool for secrets management, encryption as a service, and privileged access management.

load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_go_provider")
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
    exec_cmds = ["winget", "brew", "apt"],
)

# Use github_go_provider template (GoReleaser format)
# Asset format: vault_{version}_{os}_{arch}.zip
# Example: vault_2.0.0_linux_amd64.zip
_p = github_go_provider("hashicorp", "vault",
    asset      = "vault_{version}_{os}_{arch}.{ext}",
    executable = "vault",
)

# Export functions from template
fetch_versions   = _p["fetch_versions"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]

# Custom download_url: v2.x has no public assets (BUSL license) → return None
# to trigger system_install fallback.
def download_url(ctx, version):
    if version.startswith("2."):
        return None
    return _p["download_url"](ctx, version)

# system_install: fallback to package managers when GitHub download is unavailable
# (vault ≥2.0.0 has no public assets due to BUSL license change)
system_install = cross_platform_install(
    windows = "HashiCorp.Vault",
    macos   = "vault",
    linux   = "vault",
)

# No additional dependencies
def deps(_ctx, _version):
    return []
