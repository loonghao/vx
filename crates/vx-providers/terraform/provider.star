# provider.star - Terraform provider
#
# Terraform releases are hosted on releases.hashicorp.com (NOT GitHub).
# URL: https://releases.hashicorp.com/terraform/{version}/terraform_{version}_{os}_{arch}.zip
#
# Uses stdlib templates to minimize boilerplate.

load("@vx//stdlib:provider.star",
     "runtime_def", "fetch_versions_with_tag_prefix",
     "github_permissions",
     "archive_layout", "path_fns", "path_env_fns")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "terraform"
description = "Terraform - Infrastructure as Code tool by HashiCorp"
homepage    = "https://www.terraform.io"
repository  = "https://github.com/hashicorp/terraform"
license     = "BUSL-1.1"
ecosystem   = "devtools"
aliases     = ["tf"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("terraform",
        aliases         = ["tf"],
        version_cmd     = "{executable} version",
        version_pattern = "Terraform v\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(
    extra_hosts = ["releases.hashicorp.com", "checkpoint-api.hashicorp.com"],
)

# ---------------------------------------------------------------------------
# fetch_versions — from GitHub releases (hashicorp/terraform)
# Using GitHub releases to avoid HashiCorp API 406 Not Acceptable errors
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("hashicorp", "terraform", tag_prefix = "v")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_TERRAFORM_PLATFORMS = {
    "windows/x64":   ("windows", "amd64"),
    "windows/arm64": ("windows", "arm64"),
    "windows/x86":   ("windows", "386"),
    "macos/x64":     ("darwin",  "amd64"),
    "macos/arm64":   ("darwin",  "arm64"),
    "linux/x64":     ("linux",   "amd64"),
    "linux/arm64":   ("linux",   "arm64"),
    "linux/x86":     ("linux",   "386"),
    "linux/arm":     ("linux",   "arm"),
}

def _terraform_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _TERRAFORM_PLATFORMS.get(key, ("linux", "amd64"))

# ---------------------------------------------------------------------------
# download_url — releases.hashicorp.com
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str, arch_str = _terraform_platform(ctx)
    asset = "terraform_{}_{}_{}.zip".format(version, os_str, arch_str)
    return "https://releases.hashicorp.com/terraform/{}/{}".format(version, asset)

# ---------------------------------------------------------------------------
# Layout + path functions (from stdlib)
# ---------------------------------------------------------------------------

install_layout   = archive_layout("terraform")
paths            = path_fns("terraform")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]
env_fns          = path_env_fns()
environment      = env_fns["environment"]
post_install     = env_fns["post_install"]

system_install = cross_platform_install(
    windows = "terraform",
    macos   = "terraform",
    linux   = "terraform",
)
