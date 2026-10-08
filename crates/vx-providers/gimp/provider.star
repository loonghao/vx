# GIMP publishes platform installers rather than a Windows portable archive.
# Delegate installation to the OS manager and discover its real executable.
load("@vx//stdlib:provider.star", "runtime_def", "system_permissions")
load("@vx//stdlib:provider_templates.star", "system_provider")
load("@vx//stdlib:system_install.star", "system_install_strategies", "winget_install", "choco_install", "pkg_strategy", "apt_install", "dnf_install", "pacman_install")

name = "gimp"
description = "GIMP - GNU Image Manipulation Program"
homepage = "https://www.gimp.org"
repository = "https://gitlab.gnome.org/GNOME/gimp"
license = "GPL-3.0-or-later"
ecosystem = "media"

# Keep paths literal: the CLI's static metadata reader does not evaluate helpers.
# Upstream installs major-version aliases plus the release-series binaries.
# Inno's {autopf} supports both all-users and current-user installations.
runtimes = [runtime_def("gimp", system_paths = [
    "C:/Program Files/GIMP 3/bin/gimp-console-3.exe",
    "C:/Program Files (x86)/GIMP 3/bin/gimp-console-3.exe",
    "C:/Users/*/AppData/Local/Programs/GIMP 3/bin/gimp-console-3.exe",
    "C:/Program Files/GIMP 3/bin/gimp-console-3.2.exe",
    "C:/Program Files (x86)/GIMP 3/bin/gimp-console-3.2.exe",
    "C:/Users/*/AppData/Local/Programs/GIMP 3/bin/gimp-console-3.2.exe",
    "C:/Program Files/GIMP 3/bin/gimp-console-3.0.exe",
    "C:/Program Files (x86)/GIMP 3/bin/gimp-console-3.0.exe",
    "C:/Users/*/AppData/Local/Programs/GIMP 3/bin/gimp-console-3.0.exe",
    "C:/Program Files/GIMP 3/bin/gimp-3.exe",
    "C:/Program Files (x86)/GIMP 3/bin/gimp-3.exe",
    "C:/Users/*/AppData/Local/Programs/GIMP 3/bin/gimp-3.exe",
    "C:/Program Files/GIMP 3/bin/gimp-3.2.exe",
    "C:/Program Files (x86)/GIMP 3/bin/gimp-3.2.exe",
    "C:/Users/*/AppData/Local/Programs/GIMP 3/bin/gimp-3.2.exe",
    "C:/Program Files/GIMP 3/bin/gimp-3.0.exe",
    "C:/Program Files (x86)/GIMP 3/bin/gimp-3.0.exe",
    "C:/Users/*/AppData/Local/Programs/GIMP 3/bin/gimp-3.0.exe",
    "C:/Program Files/GIMP 2/bin/gimp-console-2.10.exe",
    "C:/Program Files (x86)/GIMP 2/bin/gimp-console-2.10.exe",
    "C:/Users/*/AppData/Local/Programs/GIMP 2/bin/gimp-console-2.10.exe",
    "C:/Program Files/GIMP 2/bin/gimp-2.10.exe",
    "C:/Program Files (x86)/GIMP 2/bin/gimp-2.10.exe",
    "C:/Users/*/AppData/Local/Programs/GIMP 2/bin/gimp-2.10.exe",
    "/Applications/GIMP.app/Contents/MacOS/gimp",
    "/usr/bin/gimp", "/usr/local/bin/gimp", "/opt/homebrew/bin/gimp",
])]
permissions = system_permissions(exec_cmds = ["winget", "choco", "brew", "apt", "dnf", "pacman"])
_p = system_provider("gimp")

def fetch_versions(_ctx):
    # OS managers choose the package version; do not invent an upstream pin.
    return [{"version": "system", "lts": True, "prerelease": False}]
download_url = _p["download_url"]
install_layout = _p["install_layout"]
store_root = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment = _p["environment"]
system_install = system_install_strategies([
    winget_install("GIMP.GIMP.3"),
    choco_install("gimp"),
    pkg_strategy("brew", "gimp", install_args = "--cask", platforms = ["macos"]),
    apt_install("gimp"), dnf_install("gimp"), pacman_install("gimp"),
])
