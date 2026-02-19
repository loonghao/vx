# provider.star - NASM provider
#
# Version source: https://www.nasm.us/pub/nasm/releasebuilds/
# NASM (Netwide Assembler) - portable 80x86 and x86-64 assembler
#
# Inheritance pattern: Level 1 (fully custom - uses nasm.us, not GitHub)

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "nasm"

def description():
    return "NASM - Netwide Assembler, portable 80x86 and x86-64 assembler"

def homepage():
    return "https://www.nasm.us"

def repository():
    return "https://github.com/netwide-assembler/nasm"

def license():
    return "BSD-2-Clause"

def ecosystem():
    return "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "nasm",
        "executable":  "nasm",
        "description": "Netwide Assembler",
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["www.nasm.us"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — nasm.us release directory listing
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch NASM versions from the official nasm.us release directory.

    Uses GitHub releases as a reliable alternative to directory scraping.
    """
    releases = ctx["http"]["get_json"](
        "https://api.github.com/repos/netwide-assembler/nasm/releases?per_page=30"
    )

    versions = []
    for release in releases:
        if release.get("draft"):
            continue
        tag = release.get("tag_name", "")
        # Tags are like "nasm-2.16.03" or "2.16.03"
        v = tag
        if v.startswith("nasm-"):
            v = v[5:]
        if v:
            prerelease = release.get("prerelease", False)
            versions.append({
                "version":    v,
                "lts":        not prerelease,
                "prerelease": prerelease,
            })

    return versions

# ---------------------------------------------------------------------------
# download_url — nasm.us official download
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build the NASM download URL from nasm.us.

    Args:
        ctx:     Provider context
        version: NASM version string, e.g. "2.16.03"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    if os == "windows":
        if arch == "x64":
            # e.g. https://www.nasm.us/pub/nasm/releasebuilds/2.16.03/win64/nasm-2.16.03-win64.zip
            filename = "nasm-{}-win64.zip".format(version)
            return "https://www.nasm.us/pub/nasm/releasebuilds/{}/win64/{}".format(version, filename)
        else:
            filename = "nasm-{}-win32.zip".format(version)
            return "https://www.nasm.us/pub/nasm/releasebuilds/{}/win32/{}".format(version, filename)

    elif os == "macos":
        # macOS: use Homebrew or build from source
        # nasm.us provides macOS binaries in macosx/
        filename = "nasm-{}-macosx.zip".format(version)
        return "https://www.nasm.us/pub/nasm/releasebuilds/{}/macosx/{}".format(version, filename)

    elif os == "linux":
        # Linux: source tarball (most distros have nasm in package manager)
        filename = "nasm-{}.tar.xz".format(version)
        return "https://www.nasm.us/pub/nasm/releasebuilds/{}/{}".format(version, filename)

    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]

    if os == "windows":
        return {
            "type":             "archive",
            "strip_prefix":     "nasm-{}".format(version),
            "executable_paths": ["nasm.exe", "ndisasm.exe"],
        }
    elif os == "macos":
        return {
            "type":             "archive",
            "strip_prefix":     "nasm-{}".format(version),
            "executable_paths": ["nasm", "ndisasm"],
        }
    else:
        # Linux: source tarball, needs compilation
        return {
            "type":             "archive",
            "strip_prefix":     "nasm-{}".format(version),
            "executable_paths": ["nasm", "ndisasm"],
        }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {"PATH": install_dir}

# ---------------------------------------------------------------------------
# system_install — preferred on Linux
# ---------------------------------------------------------------------------

def system_install(ctx):
    os = ctx["platform"]["os"]
    if os == "windows":
        return {
            "strategies": [
                {"manager": "choco",  "package": "nasm", "priority": 80},
                {"manager": "winget", "package": "NASM.NASM", "priority": 70},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "nasm", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "apt", "package": "nasm", "priority": 80},
                {"manager": "dnf", "package": "nasm", "priority": 80},
            ],
        }
    return {}

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    return []
