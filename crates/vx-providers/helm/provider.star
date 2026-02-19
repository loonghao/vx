# provider.star - Helm provider
#
# Helm releases are hosted on get.helm.sh (not GitHub releases).
# URL pattern: https://get.helm.sh/helm-v{version}-{os}-{arch}.{ext}
#
# Platform naming: Go-style (linux/darwin/windows + amd64/arm64/386/arm)
# Archive layout:  contains a subdirectory "{os}-{arch}/helm[.exe]"
#
# fetch_versions: inherited from github.star (helm/helm on GitHub)
# download_url:   custom — uses get.helm.sh domain

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "helm"

def description():
    return "Helm - The Kubernetes Package Manager"

def homepage():
    return "https://helm.sh"

def repository():
    return "https://github.com/helm/helm"

def license():
    return "Apache-2.0"

def ecosystem():
    return "devtools"

def aliases():
    return []

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "helm",
        "executable":  "helm",
        "description": "Helm - The Kubernetes Package Manager",
        "aliases":     [],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "get.helm.sh"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — inherited from github.star
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("helm", "helm")

# ---------------------------------------------------------------------------
# download_url — custom: uses get.helm.sh, not GitHub releases
#
# URL: https://get.helm.sh/helm-v{version}-{os}-{arch}.{ext}
# Examples:
#   https://get.helm.sh/helm-v3.17.0-linux-amd64.tar.gz
#   https://get.helm.sh/helm-v3.17.0-windows-amd64.zip
#   https://get.helm.sh/helm-v3.17.0-darwin-arm64.tar.gz
# ---------------------------------------------------------------------------

def _helm_platform(ctx):
    """Map platform to Helm's Go-style os/arch strings."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    os_map = {
        "windows": "windows",
        "macos":   "darwin",
        "linux":   "linux",
    }
    arch_map = {
        "x64":   "amd64",
        "arm64": "arm64",
        "x86":   "386",
        "arm":   "arm",
    }

    os_str   = os_map.get(os, "linux")
    arch_str = arch_map.get(arch, "amd64")
    return os_str, arch_str

def download_url(ctx, version):
    """Build the Helm download URL from get.helm.sh.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "3.17.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    os_str, arch_str = _helm_platform(ctx)
    os = ctx["platform"]["os"]
    ext = "zip" if os == "windows" else "tar.gz"

    # https://get.helm.sh/helm-v3.17.0-linux-amd64.tar.gz
    return "https://get.helm.sh/helm-v{}-{}-{}.{}".format(
        version, os_str, arch_str, ext
    )

# ---------------------------------------------------------------------------
# install_layout — archive contains a subdirectory "{os}-{arch}/helm[.exe]"
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Describe how to extract the Helm archive.

    Helm archives contain: {os}-{arch}/helm[.exe]
    We need to strip the subdirectory prefix.
    """
    os_str, arch_str = _helm_platform(ctx)
    os = ctx["platform"]["os"]
    exe = "helm.exe" if os == "windows" else "helm"

    return {
        "type":             "archive",
        "strip_prefix":     "{}-{}".format(os_str, arch_str),
        "executable_paths": [exe, "helm"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
