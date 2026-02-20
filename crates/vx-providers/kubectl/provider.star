# provider.star - kubectl provider
#
# Inheritance pattern:
#   - fetch_versions: inherited from @vx//stdlib:github.star
#   - download_url:   fully custom — kubectl uses dl.k8s.io (not GitHub releases)
#
# URL format: https://dl.k8s.io/release/v{version}/bin/{os}/{arch}/kubectl[.exe]

load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "kubectl"

def description():
    return "kubectl - The Kubernetes command-line tool"

def homepage():
    return "https://kubernetes.io/docs/reference/kubectl/"

def repository():
    return "https://github.com/kubernetes/kubectl"

def license():
    return "Apache-2.0"

def ecosystem():
    return "devtools"

def aliases():
    return ["kube", "k"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "kubectl",
        "executable":  "kubectl",
        "description": "Kubernetes command-line tool",
        "aliases":     ["kube", "k"],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["dl.k8s.io", "api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — inherited from github.star
# kubectl tags are "v1.32.0"; strip_v_prefix handled by releases_to_versions()
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("kubernetes", "kubectl")

# ---------------------------------------------------------------------------
# download_url — fully custom
#
# kubectl is distributed via dl.k8s.io, NOT GitHub release assets.
# URL: https://dl.k8s.io/release/v{version}/bin/{os}/{arch}/kubectl[.exe]
# ---------------------------------------------------------------------------

def _kubectl_platform(ctx):
    """Map platform to kubectl's os/arch strings."""
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

    os_str   = os_map.get(os)
    arch_str = arch_map.get(arch, "amd64")

    if not os_str:
        return None, None
    return os_str, arch_str

def download_url(ctx, version):
    """Build the kubectl download URL.

    kubectl is a single binary (no archive), downloaded directly from dl.k8s.io.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "1.32.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    os_str, arch_str = _kubectl_platform(ctx)
    if not os_str:
        return None

    os = ctx["platform"]["os"]
    exe = ".exe" if os == "windows" else ""

    return "https://dl.k8s.io/release/v{}/bin/{}/{}/kubectl{}".format(
        version, os_str, arch_str, exe
    )

# ---------------------------------------------------------------------------
# install_layout — single binary, no archive
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os  = ctx["platform"]["os"]
    exe = "kubectl.exe" if os == "windows" else "kubectl"
    return {
        "type":             "binary",
        "executable_paths": [exe, "kubectl"],
    }

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for kubectl."""
    return "{vx_home}/store/kubectl"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    exe = "kubectl.exe" if os == "windows" else "kubectl"
    return "{install_dir}/" + exe

def post_install(ctx, version, install_dir):
    """No post-install steps needed for kubectl."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
