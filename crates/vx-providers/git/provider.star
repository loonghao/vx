# provider.star - Git provider
#
# Git - Distributed version control system
# Windows: portable Git from git-for-windows (7z.exe self-extracting)
# macOS/Linux: system package manager
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def",
     "github_permissions",
     "path_fns",
     "system_install_strategies", "pkg_strategy")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "git"
description = "Git - Distributed version control system"
homepage    = "https://git-scm.com"
repository  = "https://github.com/git-for-windows/git"
license     = "GPL-2.0"
ecosystem   = "git"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("git",
        description = "Git version control",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("git-for-windows", "git")

# ---------------------------------------------------------------------------
# download_url — Windows-only portable Git (MinGit ZIP)
# macOS/Linux use system package manager
#
# Version format: "{base}.windows.{N}" (e.g. "2.53.0.windows.2")
# Tag:   "v{base}.windows.{N}"  (e.g. "v2.53.0.windows.2")
# Asset version:
#   N=1 → "{base}"       (e.g. "2.53.0")
#   N>1 → "{base}.{N}"   (e.g. "2.53.0.2")
#
# We use MinGit (minimal portable git) rather than PortableGit because:
# - MinGit ships as a plain ZIP archive (reliable extraction, no 7z/SFX needed)
# - PortableGit ships as a .7z.exe self-extracting archive (7z SFX extraction
#   is fragile: `sevenz_rust` may fail to find the 7z archive inside the PE stub)
# - MinGit contains cmd/git.exe + mingw64/bin/git.exe — identical layout to
#   PortableGit, just without GUI tools (git-gui, gitk) and bash interactivity
# - For vx use-cases (scripted git execution), MinGit is fully sufficient
# ---------------------------------------------------------------------------

def _parse_git_version(version):
    """Parse git-for-windows version string.

    Returns (base, windows_n) where base is the semver part and
    windows_n is the integer patch suffix (1, 2, …).

    Accepts both:
      "2.53.0.windows.2"  → ("2.53.0", 2)
      "2.53.0"            → ("2.53.0", 1)  # treat plain version as .windows.1
    """
    marker = ".windows."
    idx = version.find(marker)
    if idx >= 0:
        base = version[:idx]
        n_str = version[idx + len(marker):]
        n = int(n_str) if n_str.isdigit() else 1
    else:
        base = version
        n = 1
    return base, n

def download_url(ctx, version):
    if ctx.platform.os != "windows":
        return None

    base, n = _parse_git_version(version)

    # The GitHub tag is always "v{base}.windows.{N}"
    tag = "v{}.windows.{}".format(base, n)

    # Asset filename uses "{base}" for .windows.1, "{base}.{N}" for .windows.N>1
    asset_ver = "{}.{}".format(base, n) if n > 1 else base

    # Use MinGit ZIP (standard ZIP, reliably extractable) instead of PortableGit
    # .7z.exe (self-extracting archive requiring 7z SFX support).
    if ctx.platform.arch == "x64":
        asset = "MinGit-{}-64-bit.zip".format(asset_ver)
    elif ctx.platform.arch == "arm64":
        asset = "MinGit-{}-arm64.zip".format(asset_ver)
    elif ctx.platform.arch == "x86":
        asset = "MinGit-{}-32-bit.zip".format(asset_ver)
    else:
        return None

    return github_asset_url("git-for-windows", "git", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

# MinGit ZIP extracted layout:
#   <install_dir>/
#     cmd/git.exe          ← cmd-style wrapper (primary entry point)
#     mingw64/bin/git.exe  ← real MinGW git binary
#     mingw64/bin/...      ← other git tools
#
# The ZIP is extracted directly into install_dir with no top-level directory,
# so strip_prefix="" (auto-detect) will NOT attempt to strip any prefix.
# We list candidate paths so the installer can verify the correct one.
def install_layout(ctx, _version):
    if ctx.platform.os == "windows":
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": ["cmd/git.exe", "mingw64/bin/git.exe", "git.exe"],
        }
    # Non-Windows: plain archive (or system install — download_url returns None)
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["bin/git", "git"],
    }

# ---------------------------------------------------------------------------
# system_install — package manager strategies
# ---------------------------------------------------------------------------

# git is cross-platform: all managers, no platform restriction
system_install = system_install_strategies([
    pkg_strategy("winget", "Git.Git", priority = 70),
    pkg_strategy("choco",  "git",     priority = 80),
    pkg_strategy("brew",   "git",     priority = 90),
    pkg_strategy("apt",    "git",     priority = 90),
    pkg_strategy("dnf",    "git",     priority = 90),
    pkg_strategy("pacman", "git",     priority = 90),
])

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

_paths     = path_fns("git")
store_root = _paths["store_root"]

def get_execute_path(ctx, _version):
    """Return the path to the git executable inside the install dir.

    PortableGit for Windows extracts to a directory tree; the canonical
    entry point is cmd/git.exe (the cmd-shell wrapper) which is on PATH.
    The real MinGW binary lives at mingw64/bin/git.exe.

    On non-Windows the tool is managed by the system package manager,
    so install_dir points to the vx store where we placed the binary.
    """
    if ctx.platform.os == "windows":
        return ctx.install_dir + "/cmd/git.exe"
    return ctx.install_dir + "/bin/git"

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    if ctx.platform.os == "windows":
        # Prepend each directory separately so env_prepend uses the correct
        # OS-specific PATH separator (';' on Windows, ':' on Unix).
        # Order matters: later prepends appear earlier in PATH, so list the
        # most specific path last (it ends up first in PATH).
        return [
            env_prepend("PATH", ctx.install_dir + "/usr/bin"),
            env_prepend("PATH", ctx.install_dir + "/bin"),
            env_prepend("PATH", ctx.install_dir + "/mingw64/bin"),
            env_prepend("PATH", ctx.install_dir + "/cmd"),
        ]
    return []

def deps(_ctx, _version):
    return []
