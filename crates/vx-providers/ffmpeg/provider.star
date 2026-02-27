# provider.star - FFmpeg provider
#
# FFmpeg - A complete, cross-platform solution to record, convert and stream
# audio and video.
#
# Windows: GyanD/codexffmpeg releases (essentials build)
# Linux:   johnvansickle.com static builds
# macOS:   evermeet.cx binaries
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "post_extract_flatten",
     "path_fns", "path_env_fns")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "ffmpeg"
description = "FFmpeg - A complete, cross-platform solution to record, convert and stream audio and video"
homepage    = "https://ffmpeg.org"
repository  = "https://github.com/FFmpeg/FFmpeg"
license     = "LGPL-2.1"
ecosystem   = "media"
aliases     = ["avconv"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("ffmpeg",
        aliases = ["avconv"],
        test_commands = [
            {"command": "{executable} -version", "name": "version_check",
             "expected_output": "ffmpeg version"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["johnvansickle.com", "evermeet.cx"])

# ---------------------------------------------------------------------------
# fetch_versions — from GyanD/codexffmpeg (Windows) or johnvansickle.com
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("GyanD", "codexffmpeg")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    if os == "windows":
        asset = "ffmpeg-{}-essentials_build.zip".format(version)
        return github_asset_url("GyanD", "codexffmpeg", version, asset)
    elif os == "linux":
        arch_map = {"x64": "amd64", "arm64": "arm64"}
        arch_str = arch_map.get(arch)
        if not arch_str:
            return None
        return "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-{}-static.tar.xz".format(arch_str)
    elif os == "macos":
        return "https://evermeet.cx/ffmpeg/ffmpeg-{}.zip".format(version)
    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx.platform.os
    if os == "windows":
        return {
            "type":             "archive",
            "strip_prefix":     "ffmpeg-{}-essentials_build".format(version),
            "executable_paths": ["bin/ffmpeg.exe", "bin/ffprobe.exe", "bin/ffplay.exe"],
        }
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["ffmpeg", "ffprobe", "ffplay"],
    }

# ---------------------------------------------------------------------------
# post_extract — flatten Linux static build's top-level dir
# ---------------------------------------------------------------------------

post_extract = post_extract_flatten()

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

_paths           = path_fns("ffmpeg")
store_root       = _paths["store_root"]
get_execute_path = _paths["get_execute_path"]

def environment(ctx, _version):
    if ctx.platform.os == "windows":
        return [{"op": "prepend", "name": "PATH", "value": ctx.install_dir + "/bin"}]
    return [{"op": "prepend", "name": "PATH", "value": ctx.install_dir}]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
