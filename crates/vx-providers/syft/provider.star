load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_go_provider")

# Metadata
name = "syft"
description = "Syft - CLI tool and library for generating a Software Bill of Materials (SBOM)"
homepage = "https://syft.cli.anchore.io/"
repository = "https://github.com/anchore/syft"
license = "Apache-2.0"
ecosystem = "security"

# Runtimes
runtimes = [
    runtime_def("syft"),
]

# Permissions
permissions = github_permissions()

# Use github_go_provider template with custom asset naming
# syft uses underscores: syft_1.43.0_windows_amd64.zip
_p = github_go_provider("anchore", "syft",
    asset = "syft_{version}_{os}_{arch}.{ext}",
    executable = "syft",
)

fetch_versions = _p["fetch_versions"]
download_url = _p["download_url"]
install_layout = _p["install_layout"]
store_root = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment = _p["environment"]
