# Standard Godot editor; .NET builds remain a distinct upstream distribution.
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

name = "godot"
description = "Godot Engine - Free and open-source 2D and 3D game engine"
homepage = "https://godotengine.org"
repository = "https://github.com/godotengine/godot"
license = "MIT"
ecosystem = "system"
runtimes = [runtime_def("godot", version_pattern = "\\d+\\.\\d+", system_paths = [
    "/Applications/Godot.app/Contents/MacOS/Godot", "/usr/bin/godot", "/usr/local/bin/godot", "/opt/homebrew/bin/godot",
])]
permissions = github_permissions()
fetch_versions = make_fetch_versions("godotengine", "godot-builds")

_PLATFORMS = {
    "windows/x64": "win64.exe", "windows/x86": "win32.exe", "windows/arm64": "windows_arm64.exe",
    "linux/x64": "linux.x86_64", "linux/x86": "linux.x86_32", "linux/arm64": "linux.arm64", "linux/armv7": "linux.arm32",
    "macos/x64": "macos.universal", "macos/arm64": "macos.universal",
}

def _tag(version):
    return version if version.endswith("-stable") else version + "-stable"

def _executable(ctx, version):
    platform = _PLATFORMS.get(ctx.platform.os + "/" + ctx.platform.arch)
    if platform == None:
        return None
    if ctx.platform.os == "macos":
        return "Godot.app/Contents/MacOS/Godot"
    base = "Godot_v{}_{platform}".format(_tag(version), platform = platform)
    # Upstream's console wrapper preserves CLI stdout on Windows.
    return base[:-4] + "_console.exe" if ctx.platform.os == "windows" else base

def download_url(ctx, version):
    platform = _PLATFORMS.get(ctx.platform.os + "/" + ctx.platform.arch)
    if platform == None:
        return None
    tag = _tag(version)
    return github_asset_url("godotengine", "godot-builds", tag, "Godot_v{}_{}.zip".format(tag, platform))

def install_layout(ctx, version):
    exe = _executable(ctx, version)
    if exe == None:
        return None
    layout = {"type": "archive", "strip_prefix": "", "executable_paths": [exe]}
    if ctx.platform.os == "macos":
        # An empty strip_prefix would flatten the sole Godot.app directory.
        return {"type": "archive", "executable_paths": [exe], "required_paths": [exe]}
    if ctx.platform.os == "windows":
        main_exe = "Godot_v{}_{}".format(_tag(version), _PLATFORMS[ctx.platform.os + "/" + ctx.platform.arch])
        # The console executable delegates to the editor in the same directory.
        layout["required_paths"] = [exe, main_exe]
    return layout

def store_root(ctx):
    return ctx.vx_home + "/store/godot"

def get_execute_path(ctx, version):
    exe = _executable(ctx, version)
    return ctx.install_dir + "/" + exe if exe != None else None

def environment(ctx, _version):
    path = ctx.install_dir + "/Godot.app/Contents/MacOS" if ctx.platform.os == "macos" else ctx.install_dir
    return [env_prepend("PATH", path)]
