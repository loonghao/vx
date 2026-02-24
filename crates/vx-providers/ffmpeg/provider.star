# provider.star - FFmpeg provider
#
# Windows: GyanD pre-built binaries (GitHub)
# Linux:   johnvansickle.com static builds (needs flatten)
# macOS:   evermeet.cx static builds
# Bundled runtimes: ffprobe, ffplay
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def",
     "github_permissions", "post_extract_flatten")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "ffmpeg"
description = "FFmpeg - Complete solution for recording, converting and streaming audio and video"
homepage    = "https://ffmpeg.org"
repository  = "https://github.com/FFmpeg/FFmpeg"
license     = "LGPL-2.1-or-later"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("ffmpeg",
        version_pattern = "ffmpeg version",
        version_cmd     = "{executable} -version",
    ),
    bundled_runtime_def("ffprobe", bundled_with = "ffmpeg",
        version_pattern = "ffprobe version",
        test_commands   = [{"command": "{executable} -version", "name": "version_check",
                            "expected_output": "ffprobe version"}]),
    bundled_runtime_def("ffplay", bundled_with = "ffmpeg"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — GyanD pre-built releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("GyanD", "codexffmpeg")

# ---------------------------------------------------------------------------
# download_url
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
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/ffmpeg"

def get_execute_path(ctx, _version):
    exe = "ffmpeg.exe" if ctx.platform.os == "windows" else "ffmpeg"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    if ctx.platform.os == "windows":
        return [env_prepend("PATH", ctx.install_dir + "/bin")]
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
