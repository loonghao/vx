# provider.star - AWS CLI provider
#
# Version source: https://github.com/aws/aws-cli/releases
#
# AWS CLI v2 uses platform-specific installers (.msi on Windows, .pkg on macOS).
# vx prefers system package managers; Linux supports direct zip download.
#
# Inheritance pattern: Level 2 (custom download_url + system_install)

load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:install.star", "set_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "awscli"

def description():
    return "AWS CLI - Unified command line interface to Amazon Web Services"

def homepage():
    return "https://aws.amazon.com/cli/"

def repository():
    return "https://github.com/aws/aws-cli"

def license():
    return "Apache-2.0"

def ecosystem():
    return "cloud"

def aliases():
    return ["aws-cli"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "aws",
        "executable":  "aws",
        "description": "AWS Command Line Interface v2",
        "aliases":     ["awscli", "aws-cli"],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "awscli.amazonaws.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — aws/aws-cli GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("aws", "aws-cli")

# ---------------------------------------------------------------------------
# download_url — Linux only (Windows/macOS use system install)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build AWS CLI download URL.

    Linux x86_64/arm64: official zip from awscli.amazonaws.com
    Windows/macOS: use system_install (MSI/PKG not supported by vx-installer)
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    if os == "linux":
        arch_map = {"x64": "x86_64", "arm64": "aarch64"}
        arch_str = arch_map.get(arch)
        if not arch_str:
            return None
        # https://awscli.amazonaws.com/awscli-exe-linux-x86_64-2.22.0.zip
        return "https://awscli.amazonaws.com/awscli-exe-linux-{}-{}.zip".format(
            arch_str, version
        )

    # Windows/macOS: no portable archive, use system_install
    return None

# ---------------------------------------------------------------------------
# install_layout — Linux zip layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    return {
        "type":             "archive",
        "strip_prefix":     "aws",
        "executable_paths": ["dist/aws"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {"PATH": install_dir + "/bin"}

# ---------------------------------------------------------------------------
# system_install — preferred on Windows and macOS
# ---------------------------------------------------------------------------

def system_install(ctx):
    os = ctx["platform"]["os"]
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "Amazon.AWSCLI", "priority": 100},
                {"manager": "choco",  "package": "awscli",         "priority": 80},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "awscli", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "brew", "package": "awscli", "priority": 70},
            ],
        }
    return {}

# ---------------------------------------------------------------------------
# post_extract — set executable permissions on Linux
#
# AWS CLI Linux zip extracts to dist/aws (no .exe extension).
# The binary needs +x permissions on Linux/macOS.
# ---------------------------------------------------------------------------

def post_extract(ctx, version, install_dir):
    """Set executable permissions on the AWS CLI binary after extraction.

    The AWS CLI Linux zip places the main executable at dist/aws.
    On Linux/macOS we need to ensure it has execute permissions.

    Args:
        ctx:         Provider context
        version:     Installed version string
        install_dir: Path to the installation directory

    Returns:
        List of post-extract actions
    """
    os = ctx["platform"]["os"]
    if os == "linux" or os == "macos":
        return [
            set_permissions("dist/aws", "755"),
        ]
    return []

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    return []


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for awscli."""
    return "{vx_home}/store/awscli"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/aws.exe"
    else:
        return "{install_dir}/aws"

def post_install(ctx, version, install_dir):
    """Post-install hook (no-op for awscli)."""
    return None

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return "{vx_home}/store/awscli"

def get_execute_path(ctx, version):
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/aws.exe"
    else:
        return "{install_dir}/aws"

def post_install(ctx, version, install_dir):
    return None
