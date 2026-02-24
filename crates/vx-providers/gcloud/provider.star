# provider.star - Google Cloud CLI provider
#
# Version source: https://cloud.google.com/sdk/docs/release-notes
#   Download index: https://dl.google.com/dl/cloudsdk/channels/rapid/components-2.json
#
# Bundled runtimes: gsutil, bq (included in every gcloud SDK release)
#
# Inheritance pattern: Level 1 (fully custom - uses Google Cloud storage API)
load("@vx//stdlib:env.star", "env_set", "env_prepend")

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
    {
        "name":        "gcloud",
        "executable":  "gcloud",
        "description": "Google Cloud SDK CLI",
        "aliases":     ["google-cloud-sdk"],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "Google Cloud SDK"},
        ],
    },
    {
        "name":        "gsutil",
        "executable":  "gsutil",
        "description": "Google Cloud Storage utility (bundled with gcloud)",
        "bundled_with": "gcloud",
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":        "bq",
        "executable":  "bq",
        "description": "BigQuery command-line tool (bundled with gcloud)",
        "bundled_with": "gcloud",
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["dl.google.com", "storage.googleapis.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — Google Cloud SDK release manifest
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch gcloud SDK versions from Google's release manifest."""
    manifest = ctx.http.get_json(
        "https://dl.google.com/dl/cloudsdk/channels/rapid/components-2.json"
    )

    version = manifest.get("version", "")
    if not version:
        return []

    return [{"version": version, "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — Google Cloud SDK official download
# ---------------------------------------------------------------------------

def _gcloud_platform(ctx):
    """Map vx platform to gcloud SDK platform string."""
    os   = ctx.platform.os
    arch = ctx.platform.arch

    platforms = {
        "windows/x64":   ("windows",  "x86_64",  "zip"),
        "windows/x86":   ("windows",  "x86",     "zip"),
        "macos/x64":     ("darwin",   "x86_64",  "tar.gz"),
        "macos/arm64":   ("darwin",   "arm",     "tar.gz"),
        "linux/x64":     ("linux",    "x86_64",  "tar.gz"),
        "linux/arm64":   ("linux",    "arm",     "tar.gz"),
    }
    return platforms.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the Google Cloud SDK download URL.

    Args:
        ctx:     Provider context
        version: gcloud SDK version string, e.g. "502.0.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _gcloud_platform(ctx)
    if not platform:
        return None

    os_str, arch_str, ext = platform[0], platform[1], platform[2]

    # e.g. https://dl.google.com/dl/cloudsdk/channels/rapid/downloads/
    #        google-cloud-cli-502.0.0-linux-x86_64.tar.gz
    filename = "google-cloud-cli-{}-{}-{}.{}".format(version, os_str, arch_str, ext)
    return "https://dl.google.com/dl/cloudsdk/channels/rapid/downloads/{}".format(filename)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        exe_paths = ["bin/gcloud.cmd", "bin/gsutil.cmd", "bin/bq.cmd"]
    else:
        exe_paths = ["bin/gcloud", "bin/gsutil", "bin/bq"]

    return {
        "type":             "archive",
        "strip_prefix":     "google-cloud-sdk",
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    """gcloud bundles its own Python, no external deps needed."""
    return []


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/gcloud"

def get_execute_path(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        return ctx.install_dir + "/gcloud.exe"
    else:
        return ctx.install_dir + "/gcloud"

def post_install(_ctx, _version):
    return None
