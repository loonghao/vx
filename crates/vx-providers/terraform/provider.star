# provider.star - Terraform provider
#
# Terraform releases are hosted on releases.hashicorp.com (NOT GitHub).
# URL: https://releases.hashicorp.com/terraform/{version}/terraform_{version}_{os}_{arch}.zip
#
# Uses fetch_versions_from_api + runtime_def from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "fetch_versions_from_api",
     "system_permissions")
load("@vx//stdlib:http.star", "http_get_json")
load("@vx//stdlib:env.star",  "env_prepend")

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

permissions = system_permissions(
    extra_hosts = ["releases.hashicorp.com", "checkpoint-api.hashicorp.com"],
)

# ---------------------------------------------------------------------------
# fetch_versions — HashiCorp releases API
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_api(
    "https://releases.hashicorp.com/terraform/index.json",
    "hashicorp_releases",
)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

def _terraform_platform(ctx):
    os_map   = {"windows": "windows", "macos": "darwin", "linux": "linux"}
    arch_map = {"x64": "amd64", "arm64": "arm64", "x86": "386", "arm": "arm"}
    return os_map.get(ctx.platform.os, "linux"), arch_map.get(ctx.platform.arch, "amd64")

# ---------------------------------------------------------------------------
# download_url — releases.hashicorp.com
# URL: https://releases.hashicorp.com/terraform/{version}/terraform_{version}_{os}_{arch}.zip
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str, arch_str = _terraform_platform(ctx)
    asset = "terraform_{}_{}_{}.zip".format(version, os_str, arch_str)
    return "https://releases.hashicorp.com/terraform/{}/{}".format(version, asset)

# ---------------------------------------------------------------------------
# install_layout — single binary at archive root
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "terraform.exe" if ctx.platform.os == "windows" else "terraform"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "terraform"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/terraform"

def get_execute_path(ctx, _version):
    exe = "terraform.exe" if ctx.platform.os == "windows" else "terraform"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
