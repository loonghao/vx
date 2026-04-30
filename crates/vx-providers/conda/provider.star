# provider.star - Conda provider
#
# Conda ecosystem: micromamba (recommended), conda, and mamba
# for scientific computing, ML/AI, and cross-platform package management.
#
# Micromamba source: https://github.com/mamba-org/micromamba-releases
# Miniforge source:  https://github.com/conda-forge/miniforge
#
# Inheritance pattern: Level 2 (custom download_url for multiple runtimes)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "conda"

def description():
    return "Conda package, dependency and environment management"

def homepage():
    return "https://conda.io/"

def repository():
    return "https://github.com/conda-forge/miniforge"

def license():
    return "BSD-3-Clause"

def ecosystem():
    return "python"

def aliases():
    return ["miniforge", "miniconda"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "micromamba",
        "executable":  "micromamba",
        "description": "Fast, minimal conda package manager (single binary)",
        "aliases":     ["umamba"],
        "priority":    100,
    },
    {
        "name":        "conda",
        "executable":  "conda",
        "description": "Conda package and environment manager (via Miniforge)",
        "aliases":     ["miniforge", "miniconda"],
        "priority":    90,
    },
    {
        "name":        "mamba",
        "executable":  "mamba",
        "description": "Fast conda-compatible package manager (bundled with Miniforge)",
        "aliases":     [],
        "priority":    80,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

# Micromamba has its own release repo
_fetch_micromamba = make_fetch_versions("mamba-org", "micromamba-releases")

# Conda and Mamba come from Miniforge
_fetch_miniforge = make_fetch_versions("conda-forge", "miniforge")

def fetch_versions(ctx, runtime_name = "micromamba"):
    """Fetch versions for the specified runtime."""
    if runtime_name == "micromamba":
        return _fetch_micromamba(ctx)
    return _fetch_miniforge(ctx)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def _micromamba_platform(ctx):
    """Map platform to micromamba platform string."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]
    platforms = {
        "windows/x64":  "win-64",
        "macos/x64":    "osx-64",
        "macos/arm64":  "osx-arm64",
        "linux/x64":    "linux-64",
        "linux/arm64":  "linux-aarch64",
    }
    return platforms.get("{}/{}".format(os, arch))

def _miniforge_filename(ctx, version):
    """Build Miniforge filename for the platform."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]
    files = {
        "windows/x64":  "Miniforge3-{}-Windows-x86_64.exe".format(version),
        "macos/x64":    "Miniforge3-{}-MacOSX-x86_64.sh".format(version),
        "macos/arm64":  "Miniforge3-{}-MacOSX-arm64.sh".format(version),
        "linux/x64":    "Miniforge3-{}-Linux-x86_64.sh".format(version),
        "linux/arm64":  "Miniforge3-{}-Linux-aarch64.sh".format(version),
    }
    return files.get("{}/{}".format(os, arch))

def download_url(ctx, version, runtime_name = "micromamba"):
    """Build download URL for conda ecosystem tools.

    Args:
        ctx:          Provider context
        version:      Version string, e.g. "2.0.0-0"
        runtime_name: Which runtime ("micromamba", "conda", "mamba")

    Returns:
        Download URL string, or None if platform is unsupported
    """
    if runtime_name == "micromamba":
        plat = _micromamba_platform(ctx)
        if not plat:
            return None
        return github_asset_url(
            "mamba-org", "micromamba-releases", version,
            "micromamba-{}.tar.bz2".format(plat),
        )

    # conda and mamba both come from Miniforge
    filename = _miniforge_filename(ctx, version)
    if not filename:
        return None
    return github_asset_url("conda-forge", "miniforge", version, filename)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version, runtime_name = "micromamba"):
    """Describe how to extract the downloaded archive."""
    os = ctx["platform"]["os"]

    if runtime_name == "micromamba":
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": [
                "Library/bin/micromamba.exe" if os == "windows" else "bin/micromamba",
            ],
        }

    # Miniforge: conda and mamba
    if runtime_name == "conda":
        exe = "Scripts\\conda.exe" if os == "windows" else "bin/conda"
    else:
        exe = "Scripts\\mamba.exe" if os == "windows" else "bin/mamba"

    return {
        "type":             "archive",
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version, install_dir, runtime_name = "micromamba"):
    """Return environment variables to set for this runtime."""
    if runtime_name == "micromamba":
        return {
            "MAMBA_ROOT_PREFIX": install_dir,
            "PATH":             install_dir + ("/Library/bin" if ctx["platform"]["os"] == "windows" else "/bin"),
        }

    return {
        "CONDA_PREFIX": install_dir,
        "PATH":         install_dir + ("/Scripts" if ctx["platform"]["os"] == "windows" else "/bin"),
    }

# ---------------------------------------------------------------------------
# constraints
# ---------------------------------------------------------------------------

constraints = [
    {
        "when":       "*",
        "recommends": [
            {
                "runtime": "python",
                "version": ">=3.9",
                "reason":  "Conda environments commonly require Python",
            },
        ],
    },
]
