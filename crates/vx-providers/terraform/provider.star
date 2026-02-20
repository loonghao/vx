# provider.star - Terraform provider
#
# Terraform releases are hosted on releases.hashicorp.com (NOT GitHub).
# URL pattern: https://releases.hashicorp.com/terraform/{version}/terraform_{version}_{os}_{arch}.zip
#
# Inheritance pattern: Level 1 (full custom) — custom domain, no GitHub
#
# fetch_versions: custom (HashiCorp releases API)
# download_url:   custom (releases.hashicorp.com)

load("@vx//stdlib:http.star",     "http_get_json")
load("@vx//stdlib:platform.star", "exe_ext")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "terraform"

def description():
    return "Terraform - Infrastructure as Code tool by HashiCorp"

def homepage():
    return "https://www.terraform.io"

def repository():
    return "https://github.com/hashicorp/terraform"

def license():
    return "BUSL-1.1"

def ecosystem():
    return "devtools"

def aliases():
    return ["tf"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "terraform",
        "executable":  "terraform",
        "description": "Terraform - Infrastructure as Code",
        "aliases":     ["tf"],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["releases.hashicorp.com", "checkpoint-api.hashicorp.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

def _terraform_os(ctx):
    """Map vx OS name to Terraform OS string."""
    os_map = {
        "windows": "windows",
        "macos":   "darwin",
        "linux":   "linux",
    }
    return os_map.get(ctx["platform"]["os"], "linux")

def _terraform_arch(ctx):
    """Map vx arch name to Terraform arch string (go-style)."""
    arch_map = {
        "x64":   "amd64",
        "arm64": "arm64",
        "x86":   "386",
        "arm":   "arm",
    }
    return arch_map.get(ctx["platform"]["arch"], "amd64")

# ---------------------------------------------------------------------------
# fetch_versions — HashiCorp Checkpoint API
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch available Terraform versions from HashiCorp releases index.

    Uses the HashiCorp releases API which returns a JSON index of all versions.
    Falls back to checkpoint API for latest version info.

    Args:
        ctx: Provider context

    Returns:
        List of VersionInfo dicts
    """
    url = "https://releases.hashicorp.com/terraform/index.json"
    data = http_get_json(ctx, url)
    if not data:
        return []

    versions = data.get("versions", {})
    result = []
    for v in versions.keys():
        # Skip pre-release versions (alpha, beta, rc)
        if "alpha" in v or "beta" in v or "rc" in v:
            continue
        result.append({
            "version":    v,
            "stable":     True,
            "prerelease": False,
        })

    return result

# ---------------------------------------------------------------------------
# download_url — releases.hashicorp.com
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build the Terraform download URL.

    URL pattern:
        https://releases.hashicorp.com/terraform/{version}/terraform_{version}_{os}_{arch}.zip

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "1.7.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    os_str   = _terraform_os(ctx)
    arch_str = _terraform_arch(ctx)

    if not os_str or not arch_str:
        return None

    # Asset: "terraform_1.7.0_linux_amd64.zip"
    asset = "terraform_{}_{}_{}.zip".format(version, os_str, arch_str)

    return "https://releases.hashicorp.com/terraform/{}/{}".format(version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Terraform archives contain a single binary at the root."""
    os  = ctx["platform"]["os"]
    exe = "terraform.exe" if os == "windows" else "terraform"

    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "terraform"],
    }

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for terraform."""
    return "{vx_home}/store/terraform"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    exe = "terraform.exe" if os == "windows" else "terraform"
    return "{install_dir}/" + exe

def post_install(ctx, version, install_dir):
    """No post-install actions needed for terraform."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
