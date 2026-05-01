# provider.star - Java (Adoptium Temurin) provider
#
# Version source: Adoptium API
# Bundled runtimes: javac, jar
# Archive has versioned top-level dir (jdk-21.0.1+12/) — flattened via post_extract
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def",
     "fetch_versions_from_api",
     "system_permissions",
     "post_extract_flatten")
load("@vx//stdlib:env.star", "env_set", "env_prepend")

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

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx jbang:<package>` for Java package execution via jbang
# Note: jbang must be installed separately or added as a runtime
package_prefixes = ["jbang", "java"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("java",
        aliases = ["jdk", "temurin"],
        version_pattern = "version",
        version_cmd     = "{executable} -version",
    ),
    bundled_runtime_def("javac", bundled_with = "java",
        version_pattern = "javac",
        test_commands   = [{"command": "{executable} -version", "name": "version_check",
                            "expected_output": "javac"}]),
    bundled_runtime_def("jar",   bundled_with = "java"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(extra_hosts = ["api.adoptium.net"])

# ---------------------------------------------------------------------------
# fetch_versions — Adoptium API
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_api(
    "https://api.adoptium.net/v3/info/available_releases",
    "adoptium",
)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_ADOPTIUM_OS   = {"windows": "windows", "macos": "mac",     "linux": "linux"}
_ADOPTIUM_ARCH = {"x64": "x64", "arm64": "aarch64", "x86": "x86", "armv7": "arm"}

def _adoptium_os(ctx):
    return _ADOPTIUM_OS.get(ctx.platform.os)

def _adoptium_arch(ctx):
    return _ADOPTIUM_ARCH.get(ctx.platform.arch)

# ---------------------------------------------------------------------------
# download_url — Adoptium binary API
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str   = _adoptium_os(ctx)
    arch_str = _adoptium_arch(ctx)
    if not os_str or not arch_str:
        return None
    major = version.split(".")[0]
    return "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jdk/hotspot/normal/eclipse".format(
        major, os_str, arch_str,
    )

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    if ctx.platform.os == "windows":
        exe_paths = ["bin/java.exe", "bin/javac.exe", "bin/jar.exe"]
    else:
        exe_paths = ["bin/java", "bin/javac", "bin/jar"]
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# post_extract — flatten jdk-{version}/ top-level dir
# ---------------------------------------------------------------------------

post_extract = post_extract_flatten(pattern = "jdk-*")

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/java"

def get_execute_path(ctx, _version):
    exe = "java.exe" if ctx.platform.os == "windows" else "java"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [
        env_set("JAVA_HOME", ctx.install_dir),
        env_prepend("PATH", ctx.install_dir + "/bin"),
    ]

def deps(_ctx, _version):
    return []

system_install = cross_platform_install(
    windows = "java",
    macos   = "java",
    linux   = "java",
)
