# Vault Provider
# Vault is a tool for secrets management, encryption as a service, and privileged access management.
# The vault binary includes both server and CLI functionality.

load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_go_provider")

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

# Permissions: needs GitHub releases access
permissions = github_permissions()

# Use github_go_provider template (GoReleaser format)
# Asset format: vault_{version}_{os}_{arch}.zip
# Example: vault_2.0.0_linux_amd64.zip
_p = github_go_provider("hashicorp", "vault",
    asset      = "vault_{version}_{os}_{arch}.{ext}",
    executable = "vault",
)

# Export functions from template
fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]

# No additional dependencies
def deps(_ctx, _version):
    return []
