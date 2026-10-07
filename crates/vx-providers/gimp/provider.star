# GIMP publishes platform installers rather than a Windows portable archive.
# Delegate installation to the OS manager and discover its real executable.
load("@vx//stdlib:provider.star", "runtime_def", "system_permissions")
load("@vx//stdlib:provider_templates.star", "system_provider")
load("@vx//stdlib:system_install.star", "system_install_strategies", "winget_install", "pkg_strategy", "apt_install", "dnf_install", "pacman_install")

name = "gimp"
description = "GIMP - GNU Image Manipulation Program"
homepage = "https://www.gimp.org"
repository = "https://gitlab.gnome.org/GNOME/gimp"
license = "GPL-3.0-or-later"
ecosystem = "media"
runtimes = [runtime_def("gimp", system_paths = [
    "C:/Program Files/GIMP 3/bin/gimp-console-3.0.exe",
    "C:/Program Files/GIMP 3/bin/gimp-3.0.exe",
    "/Applications/GIMP.app/Contents/MacOS/gimp",
    "/usr/bin/gimp", "/usr/local/bin/gimp", "/opt/homebrew/bin/gimp",
])]
permissions = system_permissions(exec_cmds = ["winget", "brew", "apt", "dnf", "pacman"])
_p = system_provider("gimp")
fetch_versions = _p["fetch_versions"]
download_url = _p["download_url"]
install_layout = _p["install_layout"]
store_root = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment = _p["environment"]
system_install = system_install_strategies([
    winget_install("GIMP.GIMP.3"),
    pkg_strategy("brew", "gimp", install_args = "--cask", platforms = ["macos"]),
    apt_install("gimp"), dnf_install("gimp"), pacman_install("gimp"),
])
