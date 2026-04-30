load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_go_provider")

# Metadata
name = "grype"
description = "Grype - A vulnerability scanner for container images and filesystems"
homepage = "https://github.com/anchore/grype"
repository = "https://github.com/anchore/grype"
license = "Apache-2.0"
ecosystem = "security"

# Runtimes
runtimes = [
    runtime_def("grype"),
]

# Permissions
permissions = github_permissions()

# Use github_go_provider template with custom asset naming
# grype uses underscores: grype_v0.111.1_darwin_amd64.tar.gz
_p = github_go_provider("anchore", "grype",
    asset = "grype_v{version}_{os}_{arch}.{ext}",
    executable = "grype",
)

fetch_versions = _p["fetch_versions"]
download_url = _p["download_url"]
install_layout = _p["install_layout"]
store_root = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment = _p["environment"]
