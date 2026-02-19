# provider.star - GitHub CLI (gh) provider
#
# Reuse pattern: Level 2 (partial override)
#   - fetch_versions: fully inherited from github.star
#   - download_url:   overridden — gh uses a unique naming scheme:
#                     "gh_{version}_{os}_{arch}.{ext}"
#                     where macOS is "macOS" (capital M), Windows/Linux are lowercase
#                     and all platforms use zip except Linux (tar.gz)
#
# Archive layout:
#   - Windows .zip:    bin/gh.exe  (no directory prefix)
#   - macOS .zip:      gh_{version}_macOS_{arch}/bin/gh
#   - Linux .tar.gz:   gh_{version}_linux_{arch}/bin/gh

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "gh"

def description():
    return "GitHub CLI - command line tool that brings GitHub to your terminal"

def homepage():
    return "https://cli.github.com/"

def repository():
    return "https://github.com/cli/cli"

def license():
    return "MIT"

def ecosystem():
    return "devtools"

def aliases():
    return ["github-cli", "github"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "gh",
        "executable":  "gh",
        "description": "GitHub CLI",
        "aliases":     ["github-cli", "github"],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "objects.githubusercontent.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — fully inherited
# gh tags are "v2.85.0"; strip_v_prefix handled by releases_to_versions()
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("cli", "cli", include_prereleases = False)

# ---------------------------------------------------------------------------
# Platform mapping helpers
# ---------------------------------------------------------------------------

def _gh_platform(ctx):
    """Map platform to gh's (os_name, arch_name, ext) tuple.

    gh uses:
      - "macOS" (capital M) for macOS
      - "linux" / "windows" (lowercase) for others
      - "amd64" / "arm64" for arch
      - zip for Windows + macOS, tar.gz for Linux
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    arch_map = {
        "x64":   "amd64",
        "arm64": "arm64",
        "x86":   "386",
    }
    arch_name = arch_map.get(arch)
    if not arch_name:
        return None

    if os == "windows":
        return ("windows", arch_name, "zip")
    elif os == "macos":
        return ("macOS", arch_name, "zip")
    elif os == "linux":
        return ("linux", arch_name, "tar.gz")
    else:
        return None

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Asset format: "gh_{version}_{os}_{arch}.{ext}"
# Tag format:   "v{version}"
# URL:          https://github.com/cli/cli/releases/download/v{version}/gh_{version}_{os}_{arch}.{ext}
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build the gh download URL for the given version and platform.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "2.85.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _gh_platform(ctx)
    if not platform:
        return None

    os_name, arch_name, ext = platform[0], platform[1], platform[2]

    # Asset: "gh_2.85.0_linux_amd64.tar.gz"
    asset = "gh_{}_{}_{}.{}".format(version, os_name, arch_name, ext)
    tag   = "v{}".format(version)

    return github_asset_url("cli", "cli", tag, asset)

# ---------------------------------------------------------------------------
# install_layout — gh archive has a subdirectory prefix (except Windows)
#
# Windows .zip:   bin/gh.exe  (no top-level dir)
# macOS .zip:     gh_{version}_macOS_{arch}/bin/gh
# Linux .tar.gz:  gh_{version}_linux_{arch}/bin/gh
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Describe how to extract the downloaded archive.

    Args:
        ctx:     Provider context
        version: Installed version string

    Returns:
        Layout dict consumed by the vx installer
    """
    os = ctx["platform"]["os"]

    if os == "windows":
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": ["bin/gh.exe", "gh.exe"],
        }
    else:
        platform = _gh_platform(ctx)
        if not platform:
            return {"type": "archive", "strip_prefix": "", "executable_paths": ["bin/gh"]}

        os_name, arch_name, _ext = platform[0], platform[1], platform[2]
        # e.g. "gh_2.85.0_macOS_arm64"
        prefix = "gh_{}_{}_{}/".format(version, os_name, arch_name)
        return {
            "type":             "archive",
            "strip_prefix":     prefix,
            "executable_paths": ["bin/gh"],
        }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
