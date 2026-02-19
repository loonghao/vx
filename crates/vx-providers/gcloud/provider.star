# provider.star - Google Cloud CLI provider
#
# Version source: https://cloud.google.com/sdk/docs/release-notes
#   Download index: https://dl.google.com/dl/cloudsdk/channels/rapid/components-2.json
#
# Bundled runtimes: gsutil, bq (included in every gcloud SDK release)
#
# Inheritance pattern: Level 1 (fully custom - uses Google Cloud storage API)

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "gcloud"

def description():
    return "Google Cloud CLI - Command-line interface for Google Cloud Platform"

def homepage():
    return "https://cloud.google.com/sdk/gcloud"

def repository():
    return "https://github.com/GoogleCloudPlatform/google-cloud-sdk"

def license():
    return "Apache-2.0"

def ecosystem():
    return "cloud"

def aliases():
    return ["google-cloud-sdk"]

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
    },
    {
        "name":        "gsutil",
        "executable":  "gsutil",
        "description": "Google Cloud Storage utility (bundled with gcloud)",
        "bundled_with": "gcloud",
    },
    {
        "name":        "bq",
        "executable":  "bq",
        "description": "BigQuery command-line tool (bundled with gcloud)",
        "bundled_with": "gcloud",
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
    manifest = ctx["http"]["get_json"](
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
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

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

def install_layout(ctx, version):
    os = ctx["platform"]["os"]
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

def environment(ctx, version, install_dir):
    return {"PATH": install_dir + "/bin"}

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """gcloud bundles its own Python, no external deps needed."""
    return []
