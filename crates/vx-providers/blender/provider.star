# Blender portable archives keep their versioned top-level directory.
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions", "fetch_versions_from_api", "system_install_strategies", "winget_install", "pkg_strategy", "apt_install")
load("@vx//stdlib:env.star", "env_prepend")

name = "blender"
description = "Blender - Free and open-source 3D creation suite"
homepage = "https://www.blender.org"
repository = "https://github.com/blender/blender"
license = "GPL-3.0-or-later"
ecosystem = "system"
runtimes = [runtime_def("blender", version_pattern = "Blender \\d+\\.\\d+", system_paths = [
    "C:/Program Files/Blender Foundation/Blender */blender.exe",
    "/Applications/Blender.app/Contents/MacOS/Blender",
    "/usr/bin/blender", "/usr/local/bin/blender", "/opt/homebrew/bin/blender",
])]
permissions = github_permissions(extra_hosts = ["download.blender.org"], exec_cmds = ["winget", "brew", "apt"])

_fetch_tags = fetch_versions_from_api("https://api.github.com/repos/blender/blender/tags?per_page=100", "github_tags")

def fetch_versions(ctx):
    descriptor = _fetch_tags(ctx)
    descriptor["version_filter"] = "numeric"
    return descriptor

def _platform(ctx, version):
    parts = version.split(".")
    if len(parts) < 2 or not parts[0].isdigit() or not parts[1].isdigit():
        return None
    major, minor = int(parts[0]), int(parts[1])
    # Older archives use different platform names; do not fabricate those URLs.
    if major < 3:
        return None
    if ctx.platform.os == "windows" and ctx.platform.arch == "x64":
        return "windows-x64"
    if ctx.platform.os == "windows" and ctx.platform.arch == "arm64" and (major > 4 or (major == 4 and minor >= 4)):
        return "windows-arm64"
    if ctx.platform.os == "linux" and ctx.platform.arch == "x64":
        return "linux-x64"
    # Upstream macOS distributions are DMGs; use the Homebrew cask instead.
    return None

def download_url(ctx, version):
    platform = _platform(ctx, version)
    if platform == None:
        return None
    series = ".".join(version.split(".")[:2])
    ext = "zip" if ctx.platform.os == "windows" else "tar.xz"
    return "https://download.blender.org/release/Blender{}/blender-{}-{}.{}".format(series, version, platform, ext)

def install_layout(ctx, version):
    platform = _platform(ctx, version)
    if platform == None:
        return None
    exe = "blender.exe" if ctx.platform.os == "windows" else "blender"
    return {"type": "archive", "strip_prefix": "blender-{}-{}".format(version, platform), "executable_paths": [exe]}

system_install = system_install_strategies([
    winget_install("BlenderFoundation.Blender"),
    pkg_strategy("brew", "blender", install_args = "--cask", platforms = ["macos"]),
    apt_install("blender"),
])

def store_root(ctx):
    return ctx.vx_home + "/store/blender"

def get_execute_path(ctx, _version):
    return ctx.install_dir + ("/blender.exe" if ctx.platform.os == "windows" else "/blender")

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
