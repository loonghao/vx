# FreeCAD publishes a portable Windows 7z and self-contained Linux AppImages.
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions", "system_install_strategies", "winget_install", "pkg_strategy", "apt_install")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend", "env_set")

name = "freecad"
description = "FreeCAD - Free and open-source parametric 3D modeler"
homepage = "https://www.freecad.org"
repository = "https://github.com/FreeCAD/FreeCAD"
license = "LGPL-2.1-or-later"
ecosystem = "system"
runtimes = [runtime_def("freecad", test_commands = [
    # --version initializes Qt in the GUI application on some upstream builds.
    {"command": "{executable}", "check_type": "check_file", "name": "installed_application_executable"},
], system_paths = [
    "C:/Program Files/FreeCAD */bin/FreeCAD.exe", "C:/Program Files/FreeCAD*/bin/freecad.exe",
    "/Applications/FreeCAD.app/Contents/MacOS/FreeCAD",
    "/usr/bin/freecad", "/usr/local/bin/freecad", "/opt/homebrew/bin/freecad",
]), runtime_def("freecadcmd", bundled_with = "freecad", executable = "FreeCADCmd", aliases = ["FreeCADCmd", "freecad-cmd"],
    description = "FreeCAD console executable for headless automation",
    version_pattern = "FreeCAD \\d+\\.\\d+", platform_constraint = {"os": ["windows"]},
    system_paths = ["C:/Program Files/FreeCAD*/bin/FreeCADCmd.exe"]),
]
permissions = github_permissions(exec_cmds = ["winget", "brew", "apt"])
fetch_versions = make_fetch_versions("FreeCAD", "FreeCAD")

def _release_prefix(version):
    return "FreeCAD_" + version + ("-conda" if version.startswith("1.0.") else "")

def download_url(ctx, version):
    # The 1.x portable builds use Python 3.11. Earlier releases have other names.
    if not version.startswith("1."):
        return None
    if ctx.platform.os == "windows" and ctx.platform.arch == "x64":
        asset = "{}-Windows-x86_64-py311.7z".format(_release_prefix(version))
    elif ctx.platform.os == "linux" and ctx.platform.arch in ["x64", "arm64"]:
        arch = "x86_64" if ctx.platform.arch == "x64" else "aarch64"
        asset = "{}-Linux-{}-py311.AppImage".format(_release_prefix(version), arch)
    else:
        return None
    return github_asset_url("FreeCAD", "FreeCAD", version, asset)

def install_layout(ctx, version):
    if download_url(ctx, version) == None:
        return None
    if ctx.platform.os == "linux":
        return {"type": "binary", "target_name": "freecad", "target_dir": "bin", "executable_paths": ["bin/freecad"]}
    return {
        "type": "archive",
        "strip_prefix": "{}-Windows-x86_64-py311".format(_release_prefix(version)),
        "executable_paths": ["bin/FreeCAD.exe", "bin/freecad.exe", "bin/FreeCADCmd.exe", "bin/freecadcmd.exe"],
        # Both entry points are needed; a GUI-only cache cannot satisfy freecadcmd.
        "required_paths": ["bin/FreeCAD.exe", "bin/FreeCADCmd.exe"],
    }

system_install = system_install_strategies([
    winget_install("FreeCAD.FreeCAD"),
    pkg_strategy("brew", "freecad", install_args = "--cask", platforms = ["macos"]),
    apt_install("freecad"),
])

def store_root(ctx):
    return ctx.vx_home + "/store/freecad"

def get_execute_path(ctx, _version):
    if ctx.runtime_name in ["freecadcmd", "FreeCADCmd", "freecad-cmd"]:
        return ctx.install_dir + "/bin/FreeCADCmd.exe" if ctx.platform.os == "windows" else None
    return ctx.install_dir + ("/bin/FreeCAD.exe" if ctx.platform.os == "windows" else "/bin/freecad")

def environment(ctx, _version):
    ops = [env_prepend("PATH", ctx.install_dir + "/bin")]
    if ctx.platform.os == "windows":
        ops.append(env_set("DCC_MCP_FREECAD_EXECUTABLE", ctx.install_dir + "/bin/FreeCADCmd.exe"))
    return ops
