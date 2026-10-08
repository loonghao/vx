load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:install.star", "msi_install")
load("@vx//stdlib:env.star", "env_prepend", "env_set")

name = "tiled"
description = "Tiled - Free and open-source 2D level editor"
homepage = "https://www.mapeditor.org"
repository = "https://github.com/mapeditor/tiled"
license = "GPL-2.0-or-later"
ecosystem = "system"
runtimes = [runtime_def("tiled", test_commands = [
    # Installation checks must not initialize the Qt graphical application.
    {"command": "{executable}", "check_type": "check_file", "name": "installed_application_executable"},
], system_paths = ["C:/Program Files/Tiled/tiled.exe", "/Applications/Tiled.app/Contents/MacOS/Tiled", "/usr/bin/tiled"])]
permissions = github_permissions(exec_cmds = ["msiexec"])
fetch_versions = make_fetch_versions("mapeditor", "tiled")

def _asset(ctx, version):
    parts = version.split(".")
    # The asset naming scheme changed in 1.12.
    if len(parts) != 3:
        return None
    for part in parts:
        if not part.isdigit():
            return None
    if int(parts[0]) < 1 or (parts[0] == "1" and int(parts[1]) < 12):
        return None
    if ctx.platform.os == "windows" and ctx.platform.arch == "x64":
        return "Tiled-{}_Windows-10%2B_x86_64.msi".format(version)
    if ctx.platform.os == "linux" and ctx.platform.arch == "x64":
        return "Tiled-{}_Linux_x86_64.AppImage".format(version)
    if ctx.platform.os == "macos" and ctx.platform.arch in ["x64", "arm64"]:
        minimum = "11" if version == "1.12.0" else "13"
        return "Tiled-{}_macOS-{}%2B.zip".format(version, minimum)
    return None

def download_url(ctx, version):
    asset = _asset(ctx, version)
    if asset == None:
        return None
    return github_asset_url("mapeditor", "tiled", "v" + version, asset)

def install_layout(ctx, version):
    url = download_url(ctx, version)
    if url == None:
        return None
    if ctx.platform.os == "windows":
        # Upstream WiX Directory tree: TARGETDIR/PFiles/Tiled/tiled.exe.
        return msi_install(url, strip_prefix = "PFiles/Tiled", executable_paths = ["tiled.exe"])
    if ctx.platform.os == "linux":
        return {"type": "binary", "target_name": "tiled", "target_dir": "bin", "executable_paths": ["bin/tiled"]}
    # Keep the sole .app directory; an empty prefix requests auto-flattening.
    return {"type": "archive", "executable_paths": ["Tiled.app/Contents/MacOS/Tiled"], "required_paths": ["Tiled.app/Contents/MacOS/Tiled"]}

def store_root(ctx):
    return ctx.vx_home + "/store/tiled"

def get_execute_path(ctx, version):
    if _asset(ctx, version) == None:
        return None
    if ctx.platform.os == "windows":
        return ctx.install_dir + "/tiled.exe"
    if ctx.platform.os == "macos":
        return ctx.install_dir + "/Tiled.app/Contents/MacOS/Tiled"
    return ctx.install_dir + "/bin/tiled"

def environment(ctx, version):
    executable = get_execute_path(ctx, version)
    if executable == None:
        return []
    subdir = "/Tiled.app/Contents/MacOS" if ctx.platform.os == "macos" else ("/bin" if ctx.platform.os == "linux" else "")
    return [
        env_prepend("PATH", ctx.install_dir + subdir),
        env_set("DCC_MCP_TILED_EXECUTABLE", executable),
    ]
