# Official Electron distribution: NSIS on Windows, ZIP on macOS, AppImage
# on Linux. The NSIS installer runs silently into the selected vx store.
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:install.star", "archive_install", "run_nsis_installer", "run_command")
load("@vx//stdlib:env.star", "env_prepend")

name = "openscreen"
description = "OpenScreen - Open-source screen recorder and video editor"
homepage = "https://getopenscreen.com"
repository = "https://github.com/getopenscreen/openscreen"
license = "MIT"
ecosystem = "media"
runtimes = [runtime_def("openscreen", system_paths = [
    "C:/Users/*/AppData/Local/Programs/Openscreen/Openscreen.exe",
    "/Applications/Openscreen.app/Contents/MacOS/Openscreen",
], test_commands = [{"command": "{executable} --help", "name": "cli_help", "expected_output": "openscreen"}])]
permissions = github_permissions()
fetch_versions = make_fetch_versions("getopenscreen", "openscreen", include_prereleases = False)

def _uses_arch_suffixed_installer(version):
    """Windows NSIS assets carry an architecture suffix from 2.1.0 onward.

    The 2.0.x line shipped a single arch-less x64 installer
    (``Openscreen.Setup.<version>.exe``); starting with 2.1.0 upstream publishes
    per-architecture installers (``Openscreen.Setup.<version>-<arch>.exe``) for
    both x64 and arm64. Requesting the arch-less name on 2.1.0+ returns HTTP 404.
    """
    parts = version.split(".")
    major = int(parts[0])
    minor = int(parts[1]) if len(parts) > 1 and parts[1].isdigit() else 0
    return major > 2 or (major == 2 and minor >= 1)

def download_url(ctx, version):
    # The headless CLI and these package names are available from 2.0.0.
    if int(version.split(".")[0]) < 2:
        return None
    os = ctx.platform.os
    arch = ctx.platform.arch
    if os == "windows" and arch in ["x64", "arm64"]:
        if _uses_arch_suffixed_installer(version):
            asset = "Openscreen.Setup.{}-{}.exe".format(version, arch)
        elif arch == "x64":
            # 2.0.x only ever published the single arch-less x64 installer.
            asset = "Openscreen.Setup.{}.exe".format(version)
        else:
            return None
    elif os == "macos" and arch in ["x64", "arm64"]:
        asset = "Openscreen-Mac-{}-{}.zip".format(arch, version)
    elif os == "linux" and arch == "x64":
        asset = "Openscreen-Linux-{}.AppImage".format(version)
    else:
        return None
    return github_asset_url("getopenscreen", "openscreen", "v" + version, asset)

def install_layout(ctx, version):
    url = download_url(ctx, version)
    if url == None:
        return None
    if ctx.platform.os == "windows":
        return {"type": "binary", "target_name": "openscreen-installer.exe", "target_dir": "bin", "executable_paths": ["bin/openscreen-installer.exe"], "required_paths": ["Openscreen.exe"]}
    if ctx.platform.os == "linux":
        return {"type": "binary", "target_name": "openscreen", "target_dir": "bin", "executable_paths": ["bin/openscreen"], "required_paths": ["squashfs-root/AppRun"]}
    return archive_install(url, executable_paths = ["Openscreen.app/Contents/MacOS/Openscreen"])

def store_root(ctx):
    return ctx.vx_home + "/store/openscreen"

def get_execute_path(ctx, _version):
    if ctx.platform.os == "windows":
        return ctx.install_dir + "/Openscreen.exe"
    if ctx.platform.os == "macos":
        return ctx.install_dir + "/Openscreen.app/Contents/MacOS/Openscreen"
    return ctx.install_dir + "/squashfs-root/AppRun"

def post_extract(ctx, _version, install_dir):
    if ctx.platform.os == "windows":
        return [run_nsis_installer(install_dir + "/bin/openscreen-installer.exe", install_dir)]
    if ctx.platform.os == "linux":
        # AppImage extraction needs no FUSE mount or running desktop session.
        return [run_command(install_dir + "/bin/openscreen", ["--appimage-extract"], working_dir = install_dir, on_failure = "error")]
    return []

def environment(ctx, _version):
    path = ctx.install_dir + "/Openscreen.app/Contents/MacOS" if ctx.platform.os == "macos" else ctx.install_dir
    return [env_prepend("PATH", path)]
