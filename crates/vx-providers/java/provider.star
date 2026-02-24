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
load("@vx//stdlib:env.star", "env_set", "env_prepend")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "java"
description = "Java Development Kit (Eclipse Temurin) - Write once, run anywhere"
homepage    = "https://adoptium.net"
repository  = "https://github.com/adoptium/temurin-build"
license     = "GPL-2.0-with-classpath-exception"
ecosystem   = "java"
aliases     = ["jdk", "temurin"]

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
        "test_commands": [
            {"command": "{executable} -version", "name": "version_check", "expected_output": "version"},
        ],
    },
    {
        "name":        "javac",
        "executable":  "javac",
        "description": "Java compiler (bundled with JDK)",
        "bundled_with": "java",
        "test_commands": [
            {"command": "{executable} -version", "name": "version_check", "expected_output": "javac"},
        ],
    },
    {
        "name":        "jar",
        "executable":  "jar",
        "description": "Java archive tool (bundled with JDK)",
        "bundled_with": "java",
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
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

    Returns a descriptor dict for the Rust runtime to execute.
    """
    return fetch_json_versions(
        ctx,
        "https://api.adoptium.net/v3/info/available_releases",
        "adoptium",
    )

# ---------------------------------------------------------------------------
# download_url — Adoptium API
# ---------------------------------------------------------------------------

def _adoptium_os(ctx):
    """Map vx platform to Adoptium OS string."""
    os = ctx.platform.os
    mapping = {
        "windows": "windows",
        "macos":   "mac",
        "linux":   "linux",
    }
    return mapping.get(os)

def _adoptium_arch(ctx):
    """Map vx platform to Adoptium architecture string."""
    arch = ctx.platform.arch
    mapping = {
        "x64":   "x64",
        "arm64": "aarch64",
        "x86":   "x86",
        "armv7": "arm",
    }
    return mapping.get(arch)

def download_url(ctx, version):
    """Build the Java JDK download URL from Adoptium binary API.

    Uses the Adoptium direct binary download endpoint:
      https://api.adoptium.net/v3/binary/latest/{major}/ga/{os}/{arch}/jdk/hotspot/normal/eclipse

    This avoids any HTTP calls inside Starlark — the URL is constructed
    purely from the version string and platform info.

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

    # Extract major version from semver (e.g. "21.0.1+12" -> "21")
    major = version.split(".")[0]

    # Adoptium direct binary download — no HTTP query needed
    return "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jdk/hotspot/normal/eclipse".format(
        major, os_str, arch_str
    )

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os = ctx.platform.os

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

def environment(ctx, _version):
    return [
        env_set("JAVA_HOME", ctx.install_dir),
        env_prepend("PATH", ctx.install_dir + "/bin"),
    ]

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    """Java has no external dependencies."""
    return []

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for java."""
    return ctx.vx_home + "/store/java"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    exe = "java.exe" if os == "windows" else "java"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    """No post-install steps needed for java."""
    return None

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
