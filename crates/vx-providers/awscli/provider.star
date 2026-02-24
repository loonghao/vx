# provider.star - AWS CLI provider
#
# Linux: direct zip from awscli.amazonaws.com
# Windows/macOS: system package manager (MSI/PKG not supported)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "post_extract_permissions",
     "multi_platform_install", "winget_install", "choco_install",
     "brew_install")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "awscli"
description = "AWS CLI - Unified command line interface to Amazon Web Services"
homepage    = "https://aws.amazon.com/cli/"
repository  = "https://github.com/aws/aws-cli"
license     = "Apache-2.0"
ecosystem   = "cloud"
aliases     = ["aws-cli"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("aws",
        aliases         = ["awscli", "aws-cli"],
        version_pattern = "aws-cli",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["awscli.amazonaws.com"])

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("aws", "aws-cli")

# ---------------------------------------------------------------------------
# download_url — Linux only; Windows/macOS use system_install
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    if ctx.platform.os != "linux":
        return None
    arch_map = {"x64": "x86_64", "arm64": "aarch64"}
    arch_str = arch_map.get(ctx.platform.arch)
    if not arch_str:
        return None
    return "https://awscli.amazonaws.com/awscli-exe-linux-{}-{}.zip".format(arch_str, version)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    return {
        "type":             "archive",
        "strip_prefix":     "aws",
        "executable_paths": ["dist/aws"],
    }

# ---------------------------------------------------------------------------
# post_extract — set +x on Linux/macOS
# ---------------------------------------------------------------------------

post_extract = post_extract_permissions(["dist/aws"])

# ---------------------------------------------------------------------------
# system_install — preferred on Windows and macOS
# ---------------------------------------------------------------------------

system_install = multi_platform_install(
    windows_strategies = [
        winget_install("Amazon.AWSCLI", priority = 100),
        choco_install("awscli",         priority = 80),
    ],
    macos_strategies = [
        brew_install("awscli"),
    ],
    linux_strategies = [
        brew_install("awscli", priority = 70),
    ],
)

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/awscli"

def get_execute_path(ctx, _version):
    exe = "aws.exe" if ctx.platform.os == "windows" else "aws"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []
