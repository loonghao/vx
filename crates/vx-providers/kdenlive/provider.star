# The Windows standalone executable is a 7z SFX, not an installer to execute.
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions", "fetch_versions_from_api", "pkg_strategy", "system_install_strategies")
load("@vx//stdlib:env.star", "env_prepend", "env_set")

name = "kdenlive"
description = "Kdenlive - Free and open-source video editor"
homepage = "https://kdenlive.org"
repository = "https://invent.kde.org/multimedia/kdenlive"
license = "GPL-3.0-only"
ecosystem = "system"
runtimes = [runtime_def("kdenlive", test_commands = [
    # --version initializes QApplication before parsing command-line options.
    {"command": "{executable}", "check_type": "check_file", "name": "installed_application_executable"},
], system_paths = [
    "C:/Program Files/kdenlive/bin/kdenlive.exe",
    "/Applications/kdenlive.app/Contents/MacOS/kdenlive", "/usr/bin/kdenlive",
])]
permissions = github_permissions(extra_hosts = ["cdn.download.kde.org"], exec_cmds = ["brew"])
_fetch_tags = fetch_versions_from_api("https://api.github.com/repos/KDE/kdenlive/tags?per_page=100", "github_tags")

def fetch_versions(ctx):
    descriptor = _fetch_tags(ctx)
    descriptor["version_filter"] = "numeric"
    # KDE uses .80 for beta and .90 for RC tags, without textual suffixes.
    descriptor["exclude_version_suffixes"] = ["." + str(patch) for patch in range(80, 100)]
    return descriptor

def _supported(ctx, version):
    parts = version.split(".")
    if len(parts) != 3:
        return False
    for part in parts:
        if not part.isdigit():
            return False
    if int(parts[0]) < 24 or parts[1] not in ["04", "08", "12"] or int(parts[2]) >= 80:
        return False
    return ctx.platform.arch == "x64" and ctx.platform.os in ["windows", "linux"]

def download_url(ctx, version):
    if not _supported(ctx, version):
        return None
    series = ".".join(version.split(".")[:2])
    asset = "kdenlive-{}_standalone.exe".format(version) if ctx.platform.os == "windows" else "kdenlive-{}-x86_64.AppImage".format(version)
    return "https://cdn.download.kde.org/stable/kdenlive/{}/{}/{}".format(series, ctx.platform.os, asset)

def install_layout(ctx, version):
    if not _supported(ctx, version):
        return None
    if ctx.platform.os == "windows":
        return {"type": "archive", "strip_prefix": "kdenlive-{}_standalone".format(version), "executable_paths": ["bin/kdenlive.exe"], "required_paths": ["bin/kdenlive.exe", "bin/melt.exe", "bin/ffprobe.exe", "bin/ffmpeg.exe"]}
    return {"type": "binary", "target_name": "kdenlive", "target_dir": "bin", "executable_paths": ["bin/kdenlive"]}

def system_install(ctx):
    if ctx.platform.os == "macos" and ctx.platform.arch in ["x64", "arm64"]:
        return system_install_strategies([pkg_strategy("brew", "kdenlive", install_args = "--cask", platforms = ["macos"])])
    return system_install_strategies([])

def store_root(ctx):
    return ctx.vx_home + "/store/kdenlive"

def get_execute_path(ctx, version):
    if ctx.platform.os == "macos" and ctx.platform.arch in ["x64", "arm64"]:
        return "/Applications/kdenlive.app/Contents/MacOS/kdenlive"
    if not _supported(ctx, version):
        return None
    return ctx.install_dir + ("/bin/kdenlive.exe" if ctx.platform.os == "windows" else "/bin/kdenlive")

def environment(ctx, version):
    executable = get_execute_path(ctx, version)
    if executable == None:
        return []
    ops = [env_set("DCC_MCP_KDENLIVE_EXECUTABLE", executable)]
    if ctx.platform.os != "macos":
        ops.insert(0, env_prepend("PATH", ctx.install_dir + "/bin"))
    if ctx.platform.os == "windows":
        # The official standalone archive ships these beside kdenlive.exe.
        for variable, binary in [
            ("DCC_MCP_KDENLIVE_MELT", "melt"),
            ("DCC_MCP_KDENLIVE_FFPROBE", "ffprobe"),
            ("DCC_MCP_KDENLIVE_FFMPEG", "ffmpeg"),
        ]:
            ops.append(env_set(variable, ctx.install_dir + "/bin/" + binary + ".exe"))
    return ops
