# Official Windows portable ZIPs; macOS/Linux use their system packages.
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions", "pkg_strategy", "apt_install", "system_install_strategies")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

name = "obs"
description = "OBS Studio - Free and open-source recording and live streaming"
homepage = "https://obsproject.com"
repository = "https://github.com/obsproject/obs-studio"
license = "GPL-2.0-or-later"
ecosystem = "system"
runtimes = [runtime_def("obs", aliases = ["obs-studio"], system_paths = [
    "C:/Program Files/obs-studio/bin/64bit/obs64.exe",
    "/Applications/OBS.app/Contents/MacOS/OBS",
    "/usr/bin/obs", "/usr/local/bin/obs",
])]
permissions = github_permissions(exec_cmds = ["brew", "apt"])
fetch_versions = make_fetch_versions("obsproject", "obs-studio")

def download_url(ctx, version):
    if ctx.platform.os != "windows" or ctx.platform.arch not in ["x64", "arm64"]:
        return None
    # Architecture-suffixed Windows archives first shipped in OBS 31.1.0; older
    # releases only publish an unsuffixed "-Windows.zip", which this provider
    # does not install. Pre-release tags are refused as a whole because their
    # asset sets vary per tag (31.1.0-beta1 ships no x64 archive).
    parts = version.split("-")[0].split(".")
    if "-" in version or len(parts) < 2 or not parts[0].isdigit() or not parts[1].isdigit():
        return None
    major, minor = int(parts[0]), int(parts[1])
    if major < 31 or (major == 31 and minor < 1):
        return None
    asset = "OBS-Studio-{}-Windows-{}.zip".format(version, ctx.platform.arch)
    return github_asset_url("obsproject", "obs-studio", version, asset)

def install_layout(ctx, version):
    if download_url(ctx, version) == None:
        return None
    return {"type": "archive", "strip_prefix": "", "executable_paths": ["bin/64bit/obs64.exe"]}

# The runtime reads this descriptor directly and filters strategies by OS.
system_install = system_install_strategies([
    pkg_strategy("brew", "obs", install_args = "--cask", platforms = ["macos"]),
    apt_install("obs-studio"),
])

def store_root(ctx):
    return ctx.vx_home + "/store/obs"

def get_execute_path(ctx, version):
    if download_url(ctx, version) != None:
        return ctx.install_dir + "/bin/64bit/obs64.exe"
    if ctx.platform.arch not in ["x64", "arm64"]:
        return None
    if ctx.platform.os == "macos":
        return "/Applications/OBS.app/Contents/MacOS/OBS"
    if ctx.platform.os == "linux":
        return "/usr/bin/obs"
    return None

def environment(ctx, _version):
    if ctx.platform.os == "windows":
        return [env_prepend("PATH", ctx.install_dir + "/bin/64bit")]
    return []
