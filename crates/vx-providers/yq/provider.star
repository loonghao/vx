# provider.star - yq provider
#
# yq: portable command-line YAML, JSON, XML, CSV, TOML processor
# Asset: yq_{os}_{arch}[.exe]  (direct binary, no archive)
#
# Uses github_binary_provider template from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "github_binary_provider", "runtime_def", "github_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "yq"
description = "yq - a portable command-line YAML, JSON, XML, CSV, TOML and properties processor"
homepage    = "https://github.com/mikefarah/yq"
repository  = "https://github.com/mikefarah/yq"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("yq",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "yq \\(https://github.com/mikefarah/yq\\) version"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template — github_binary_provider
#
# yq asset naming: yq_{os}_{arch}[.exe]
# - os: windows, darwin, linux
# - arch: amd64, arm64, arm, 386
# ---------------------------------------------------------------------------

_p = github_binary_provider(
    "mikefarah", "yq",
    asset = "yq_{os}_{arch}{exe}",
    tag_prefix = "v",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return []
