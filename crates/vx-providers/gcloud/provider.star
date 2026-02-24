# provider.star - Google Cloud CLI provider
#
# Version source: Google Cloud SDK release manifest
# Bundled runtimes: gsutil, bq
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "fetch_versions_from_api",
     "system_permissions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "gcloud"
description = "Google Cloud CLI - Command-line interface for Google Cloud Platform"
homepage    = "https://cloud.google.com/sdk/gcloud"
repository  = "https://github.com/GoogleCloudPlatform/google-cloud-sdk"
license     = "Apache-2.0"
ecosystem   = "cloud"
aliases     = ["google-cloud-sdk"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("gcloud",
        aliases = ["google-cloud-sdk"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "Google Cloud SDK"},
        ],
    ),
    bundled_runtime_def("gsutil", bundled_with = "gcloud"),
    bundled_runtime_def("bq",     bundled_with = "gcloud"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    extra_hosts = ["dl.google.com", "storage.googleapis.com"],
)

# ---------------------------------------------------------------------------
# fetch_versions — Google Cloud SDK release manifest
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_api(
    "https://dl.google.com/dl/cloudsdk/channels/rapid/components-2.json",
    "gcloud_manifest",
)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_GCLOUD_PLATFORMS = {
    "windows/x64":  ("windows", "x86_64", "zip"),
    "windows/x86":  ("windows", "x86",    "zip"),
    "macos/x64":    ("darwin",  "x86_64", "tar.gz"),
    "macos/arm64":  ("darwin",  "arm",    "tar.gz"),
    "linux/x64":    ("linux",   "x86_64", "tar.gz"),
    "linux/arm64":  ("linux",   "arm",    "tar.gz"),
}

def _gcloud_platform(ctx):
    return _GCLOUD_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _gcloud_platform(ctx)
    if not platform:
        return None
    os_str, arch_str, ext = platform[0], platform[1], platform[2]
    filename = "google-cloud-cli-{}-{}-{}.{}".format(version, os_str, arch_str, ext)
    return "https://dl.google.com/dl/cloudsdk/channels/rapid/downloads/{}".format(filename)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    if ctx.platform.os == "windows":
        exe_paths = ["bin/gcloud.cmd", "bin/gsutil.cmd", "bin/bq.cmd"]
    else:
        exe_paths = ["bin/gcloud", "bin/gsutil", "bin/bq"]
    return {
        "type":             "archive",
        "strip_prefix":     "google-cloud-sdk",
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/gcloud"

def get_execute_path(ctx, _version):
    exe = "gcloud.exe" if ctx.platform.os == "windows" else "gcloud"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []
