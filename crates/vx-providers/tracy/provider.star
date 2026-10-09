# Release archives include the profiler and capture/export command line helpers.
load("@vx//stdlib:provider.star", "runtime_def", "bundled_runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

name = "tracy"
description = "Tracy - Open-source real-time frame profiler"
homepage = "https://github.com/wolfpld/tracy"
repository = "https://github.com/wolfpld/tracy"
license = "BSD-3-Clause"
ecosystem = "system"
runtimes = [
    runtime_def("tracy", executable = "tracy-profiler", aliases = ["tracy-profiler"], test_commands = [
        # The Linux profiler needs host EGL libraries even before parsing --help.
        {"command": "{executable}", "check_type": "check_file", "name": "installed_profiler_executable"},
    ]),
    bundled_runtime_def("tracy-capture", "tracy", description = "Tracy command-line trace capture", test_commands = [
        {"command": "{executable}", "check_type": "check_file", "name": "installed_helper_executable"},
    ]),
    bundled_runtime_def("tracy-csvexport", "tracy", description = "Tracy CSV trace export", test_commands = [
        {"command": "{executable} --version", "name": "cli_version_check", "expected_output": "tracy-csvexport \\d+\\.\\d+\\.\\d+"},
    ]),
    bundled_runtime_def("tracy-update", "tracy", description = "Tracy trace format update", test_commands = [
        {"command": "{executable}", "check_type": "check_file", "name": "installed_helper_executable"},
    ]),
]
permissions = github_permissions()
fetch_versions = make_fetch_versions("wolfpld", "tracy")

def _supported(ctx, version):
    parts = version.split(".")
    if len(parts) != 3:
        return False
    for part in parts:
        if not part.isdigit():
            return False
    # Verified release format and native Apple Silicon bundle start at 0.14.1.
    if parts[0] == "0" and (int(parts[1]) < 14 or (parts[1] == "14" and int(parts[2]) < 1)):
        return False
    if ctx.platform.os in ["windows", "linux"]:
        return ctx.platform.arch == "x64"
    return ctx.platform.os == "macos" and ctx.platform.arch == "arm64"

def download_url(ctx, version):
    if not _supported(ctx, version):
        return None
    return github_asset_url("wolfpld", "tracy", "v" + version, "{}-{}.zip".format(ctx.platform.os, version))

def _profiler(ctx):
    if ctx.platform.os == "windows":
        return "tracy-profiler.exe"
    if ctx.platform.os == "macos":
        return "tracy-profiler.app/Contents/MacOS/tracy-profiler"
    return "tracy-profiler-x86_64.AppImage"

def install_layout(ctx, version):
    if not _supported(ctx, version):
        return None
    suffix = ".exe" if ctx.platform.os == "windows" else ""
    paths = [_profiler(ctx)] + [runtime + suffix for runtime in ["tracy-capture", "tracy-csvexport", "tracy-update"]]
    return {"type": "archive", "strip_prefix": "", "executable_paths": paths, "required_paths": paths}

def store_root(ctx):
    return ctx.vx_home + "/store/tracy"

def get_execute_path(ctx, version):
    if not _supported(ctx, version):
        return None
    runtime = ctx.runtime_name
    if runtime in ["tracy-capture", "tracy-csvexport", "tracy-update"]:
        return ctx.install_dir + "/" + runtime + (".exe" if ctx.platform.os == "windows" else "")
    return ctx.install_dir + "/" + _profiler(ctx)

def environment(ctx, _version):
    paths = [env_prepend("PATH", ctx.install_dir)]
    if ctx.platform.os == "macos":
        paths.append(env_prepend("PATH", ctx.install_dir + "/tracy-profiler.app/Contents/MacOS"))
    return paths
