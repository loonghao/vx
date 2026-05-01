# Step CLI Provider
# Step CLI is a zero trust swiss army knife for working with X509 certificates,
# OAuth, JWT, OATH OTP, etc.

load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_go_provider")
load("@vx//stdlib:system_install.star", "cross_platform_install")

# Provider metadata
name        = "step-cli"
description = "Step CLI - zero trust swiss army knife for certificates"
homepage    = "https://smallstep.com/docs/cli/"
repository  = "https://github.com/smallstep/cli"
license     = "Apache-2.0"
ecosystem   = "security"

# Runtime definitions
runtimes = [
    runtime_def("step", aliases=["step-cli"]),
]

# Permissions: needs GitHub releases access
permissions = github_permissions()

# Use github_go_provider template (GoReleaser format)
# Asset format: step_{os}_{version}_{arch}.{ext}
# Example: step_linux_0.30.2_amd64.tar.gz
# Archive has top-level dir: step_{os}_{version}_{arch}/ → need strip_prefix
_p = github_go_provider("smallstep", "cli",
    asset        = "step_{os}_{version}_{arch}.{ext}",
    executable   = "step",
    strip_prefix = "step_{os}_{version}_{arch}",
)

# Export functions from template
fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]

# system_install fallback when GitHub download is unavailable
system_install = cross_platform_install(
    windows = "step-cli",
    macos   = "step",
    linux   = "step",
)

# No additional dependencies
def deps(_ctx, _version):
    return []
