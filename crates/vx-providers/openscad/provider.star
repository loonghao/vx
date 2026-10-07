load("@vx//stdlib:provider.star", "runtime_def", "github_permissions", "system_install_strategies", "winget_install", "pkg_strategy", "apt_install")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

name = "openscad"
description = "OpenSCAD - Free and open-source programmable solid 3D CAD modeler"
homepage = "https://openscad.org"
repository = "https://github.com/openscad/openscad"
license = "GPL-2.0-or-later"
ecosystem = "system"
runtimes = [runtime_def("openscad", version_pattern = "OpenSCAD version \\d+\\.\\d+", system_paths = [
    "C:/Program Files/OpenSCAD/openscad.exe", "C:/Program Files (x86)/OpenSCAD/openscad.exe",
    "/Applications/OpenSCAD.app/Contents/MacOS/OpenSCAD", "/usr/bin/openscad", "/usr/local/bin/openscad", "/opt/homebrew/bin/openscad",
])]
permissions = github_permissions(exec_cmds = ["winget", "brew", "apt"])
fetch_versions = make_fetch_versions("openscad", "openscad", tag_prefix = "openscad-")

def download_url(ctx, version):
    if ctx.platform.os == "windows" and ctx.platform.arch in ["x64", "x86"]:
        arch = "x86-64" if ctx.platform.arch == "x64" else "x86-32"
        asset = "OpenSCAD-{}-{}.zip".format(version, arch)
    elif ctx.platform.os == "linux" and ctx.platform.arch == "x64":
        asset = "OpenSCAD-{}-x86_64.AppImage".format(version)
    else:
        return None
    return github_asset_url("openscad", "openscad", "openscad-" + version, asset)

def install_layout(ctx, version):
    if download_url(ctx, version) == None:
        return None
    if ctx.platform.os == "linux":
        return {"type": "binary", "target_name": "openscad", "target_dir": "bin", "executable_paths": ["bin/openscad"]}
    return {"type": "archive", "strip_prefix": "openscad-{}".format(version), "executable_paths": ["openscad.exe", "openscad.com"]}

system_install = system_install_strategies([
    winget_install("OpenSCAD.OpenSCAD"),
    pkg_strategy("brew", "openscad", install_args = "--cask", platforms = ["macos"]),
    apt_install("openscad"),
])

def store_root(ctx):
    return ctx.vx_home + "/store/openscad"

def get_execute_path(ctx, _version):
    return ctx.install_dir + ("/openscad.com" if ctx.platform.os == "windows" else "/bin/openscad")

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir if ctx.platform.os == "windows" else ctx.install_dir + "/bin")]
