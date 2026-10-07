load("@vx//stdlib:provider.star", "runtime_def", "github_permissions", "system_install_strategies", "pkg_strategy")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

name = "material-maker"
description = "Material Maker - Free and open-source procedural material authoring application"
homepage = "https://www.materialmaker.org"
repository = "https://github.com/RodZill4/material-maker"
license = "MIT"
ecosystem = "system"
runtimes = [runtime_def("material-maker", executable = "material_maker", aliases = ["material_maker", "materialmaker"],
    # This checks the embedded Godot engine; the package version comes from GitHub.
    test_commands = [{"command": "{executable} --headless --version", "name": "engine_version_check", "expected_output": "\\d+\\.\\d+"}],
    system_paths = ["/Applications/Material Maker.app/Contents/MacOS/Material Maker", "/usr/bin/material_maker", "/usr/local/bin/material_maker"],
)]
permissions = github_permissions(exec_cmds = ["brew"])
fetch_versions = make_fetch_versions("RodZill4", "material-maker")

# Upstream distributes macOS as a DMG; Homebrew installs the app bundle.
system_install = system_install_strategies([
    pkg_strategy("brew", "material-maker", install_args = "--cask", platforms = ["macos"]),
])

def _archive_root(version, os):
    return "material_maker_{}_{}".format(version.replace(".", "_"), os)

def download_url(ctx, version):
    if ctx.platform.arch != "x64" or ctx.platform.os not in ["windows", "linux"]:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return github_asset_url("RodZill4", "material-maker", version, _archive_root(version, ctx.platform.os) + "." + ext)

def install_layout(ctx, version):
    if download_url(ctx, version) == None:
        return None
    exe = "material_maker.exe" if ctx.platform.os == "windows" else "material_maker.x86_64"
    return {"type": "archive", "strip_prefix": _archive_root(version, ctx.platform.os), "executable_paths": [exe]}

def store_root(ctx):
    return ctx.vx_home + "/store/material-maker"

def get_execute_path(ctx, _version):
    return ctx.install_dir + ("/material_maker.exe" if ctx.platform.os == "windows" else "/material_maker.x86_64")

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
