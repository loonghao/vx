load("@vx//stdlib:provider.star", "runtime_def", "github_permissions", "fetch_versions_from_api", "system_install_strategies", "winget_install", "pkg_strategy", "apt_install")
load("@vx//stdlib:env.star", "env_prepend")

name = "krita"
description = "Krita - Free and open-source digital painting application"
homepage = "https://krita.org"
repository = "https://invent.kde.org/graphics/krita"
license = "GPL-3.0-or-later"
ecosystem = "system"
runtimes = [runtime_def("krita", version_pattern = "krita \\d+\\.\\d+", system_paths = [
    "C:/Program Files/Krita (x64)/bin/krita.exe", "C:/Program Files/Krita/bin/krita.exe",
    "/Applications/krita.app/Contents/MacOS/krita", "/usr/bin/krita", "/usr/local/bin/krita", "/opt/homebrew/bin/krita",
])]
permissions = github_permissions(extra_hosts = ["download.kde.org", "cdn.kde.org", "mirrors.kde.org"], exec_cmds = ["winget", "brew", "apt"])
_fetch_tags = fetch_versions_from_api("https://api.github.com/repos/KDE/krita/tags?per_page=100", "github_tags")

def fetch_versions(ctx):
    descriptor = _fetch_tags(ctx)
    descriptor["version_filter"] = "numeric"
    return descriptor

def download_url(ctx, version):
    if ctx.platform.arch != "x64":
        return None
    if ctx.platform.os == "windows":
        asset = "krita-x64-{}.zip".format(version)
    elif ctx.platform.os == "linux":
        asset = "krita-{}-x86_64.AppImage".format(version)
    else:
        return None
    return "https://download.kde.org/stable/krita/{}/{}".format(version, asset)

def install_layout(ctx, version):
    if download_url(ctx, version) == None:
        return None
    if ctx.platform.os == "linux":
        return {"type": "binary", "target_name": "krita", "target_dir": "bin", "executable_paths": ["bin/krita"]}
    return {"type": "archive", "strip_prefix": "krita-x64-{}".format(version), "executable_paths": ["bin/krita.com", "bin/krita.exe"]}

system_install = system_install_strategies([
    winget_install("KDE.Krita"),
    pkg_strategy("brew", "krita", install_args = "--cask", platforms = ["macos"]),
    apt_install("krita"),
])

def store_root(ctx):
    return ctx.vx_home + "/store/krita"

def get_execute_path(ctx, _version):
    return ctx.install_dir + ("/bin/krita.com" if ctx.platform.os == "windows" else "/bin/krita")

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]
