# provider.star - Java (Adoptium Temurin) provider
#
# Version source: Adoptium API
#   https://api.adoptium.net/v3/info/available_releases
#
# Uses Eclipse Adoptium (formerly AdoptOpenJDK) Temurin distribution,
# which is the most widely used OpenJDK distribution.
#
# Bundled runtimes: javac, jar (included in every JDK release)
#
# Inheritance pattern: Level 1 (fully custom - uses Adoptium API, not GitHub)

load("@vx//stdlib:install.star", "flatten_dir")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "java"

def description():
    return "Java Development Kit (Eclipse Temurin) - Write once, run anywhere"

def homepage():
    return "https://adoptium.net"

def repository():
    return "https://github.com/adoptium/temurin-build"

def license():
    return "GPL-2.0-with-classpath-exception"

def ecosystem():
    return "java"

def aliases():
    return ["jdk", "temurin"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "java",
        "executable":  "java",
        "description": "Java runtime environment",
        "aliases":     ["jdk", "temurin"],
        "priority":    100,
    },
    {
        "name":        "javac",
        "executable":  "javac",
        "description": "Java compiler (bundled with JDK)",
        "bundled_with": "java",
    },
    {
        "name":        "jar",
        "executable":  "jar",
        "description": "Java archive tool (bundled with JDK)",
        "bundled_with": "java",
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.adoptium.net"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — Adoptium API
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch Java versions from the Adoptium API.

    Uses https://api.adoptium.net which provides:
    - All available Temurin releases
    - LTS/non-LTS information
    - No rate limiting
    """
    # Get available release versions
    info = ctx["http"]["get_json"](
        "https://api.adoptium.net/v3/info/available_releases"
    )

    available = info.get("available_releases", [])
    lts_releases = info.get("available_lts_releases", [])
    lts_set = {}
    for v in lts_releases:
        lts_set[v] = True

    versions = []
    for major in available:
        # Get the latest release for each major version
        releases = ctx["http"]["get_json"](
            "https://api.adoptium.net/v3/assets/latest/{}/hotspot?architecture=x64&image_type=jdk&os=linux&vendor=eclipse".format(major)
        )
        for release in releases:
            version_data = release.get("version", {})
            semver = version_data.get("semver", "")
            if semver:
                versions.append({
                    "version":    semver,
                    "lts":        lts_set.get(major, False),
                    "prerelease": "ea" in semver or "beta" in semver,
                })

    return versions

# ---------------------------------------------------------------------------
# download_url — Adoptium API
# ---------------------------------------------------------------------------

def _adoptium_os(ctx):
    """Map vx platform to Adoptium OS string."""
    os = ctx["platform"]["os"]
    mapping = {
        "windows": "windows",
        "macos":   "mac",
        "linux":   "linux",
    }
    return mapping.get(os)

def _adoptium_arch(ctx):
    """Map vx platform to Adoptium architecture string."""
    arch = ctx["platform"]["arch"]
    mapping = {
        "x64":   "x64",
        "arm64": "aarch64",
        "x86":   "x86",
        "armv7": "arm",
    }
    return mapping.get(arch)

def download_url(ctx, version):
    """Build the Java JDK download URL from Adoptium API.

    Args:
        ctx:     Provider context
        version: Java version string, e.g. "21.0.1+12"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    os_str   = _adoptium_os(ctx)
    arch_str = _adoptium_arch(ctx)
    if not os_str or not arch_str:
        return None

    os = ctx["platform"]["os"]

    # Extract major version from semver (e.g. "21.0.1+12" -> "21")
    major = version.split(".")[0]

    # Query Adoptium API for the specific version
    releases = ctx["http"]["get_json"](
        "https://api.adoptium.net/v3/assets/latest/{}/hotspot?architecture={}&image_type=jdk&os={}&vendor=eclipse".format(
            major, arch_str, os_str
        )
    )

    for release in releases:
        binary = release.get("binary", {})
        package = binary.get("package", {})
        link = package.get("link", "")
        if link:
            return link

    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]

    if os == "windows":
        exe_paths = ["bin/java.exe", "bin/javac.exe", "bin/jar.exe"]
    else:
        exe_paths = ["bin/java", "bin/javac", "bin/jar"]

    return {
        "type":             "archive",
        "strip_prefix":     "",   # Adoptium archives have variable prefix (jdk-21.0.1+12/)
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "JAVA_HOME": install_dir,
        "PATH":      install_dir + "/bin",
    }

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """Java has no external dependencies."""
    return []

# ---------------------------------------------------------------------------
# post_extract — flatten JDK directory structure
#
# Temurin archives extract to a versioned subdirectory:
#   jdk-21.0.1+12/bin/java  →  bin/java
#
# The Rust runtime's flatten_dir action moves all contents one level up
# and removes the now-empty subdirectory.
# ---------------------------------------------------------------------------

def post_extract(ctx, version, install_dir):
    """Flatten the JDK subdirectory into the install root.

    Adoptium Temurin archives extract to a versioned top-level directory
    (e.g. jdk-21.0.1+12/) rather than directly into the install path.
    This hook flattens that structure so executables are at bin/java, etc.

    Args:
        ctx:         Provider context
        version:     Installed version string
        install_dir: Path to the installation directory

    Returns:
        List of post-extract actions
    """
    return [
        flatten_dir(pattern = "jdk-*"),
    ]
