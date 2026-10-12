# @vx//stdlib:rez.star
# Rez package bundle source descriptors for provider.star scripts.

load("@vx//stdlib:github.star", "github_asset_url")

def _rez_platform(ctx):
    return "osx" if ctx.platform.os == "macos" else ctx.platform.os

def _rez_arch(ctx):
    if ctx.platform.arch == "x64":
        return "x86_64"
    if ctx.platform.arch == "arm64":
        return "arm_64"
    return ctx.platform.arch

def _lookup_target(ctx, mapping):
    target = ctx.platform.target
    if target and target in mapping:
        return mapping[target]
    return mapping.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

def _unsupported_reason(ctx, tool, unsupported_targets):
    reason = _lookup_target(ctx, unsupported_targets)
    if reason:
        return reason
    return "No official {} Rez bundle is published for {}/{}.".format(
        tool,
        ctx.platform.os,
        ctx.platform.arch,
    )

def rez_bundle_source(owner, repo, tool, targets, unsupported_targets = {},
                      programs = {}, versions_url = None,
                      bundle_schema_version = 1, published = True):
    """Create provider functions for immutable Rez package bundle releases.

    `targets` maps either a vx `os/arch` pair or an exact target triple to the
    release asset triple. `unsupported_targets` uses the same keys and stores
    user-facing reasons. The returned `rez_bundle` function is consumed by the
    vx runtime adapter bridge; the remaining functions work with the ordinary
    provider download/install pipeline.

    Set `published = False` while no release exists yet. The runtime bridge
    asks every provider for a bundle, so leaving it enabled would make vx try
    to activate one from an ordinary installation and fail during prepare.
    """
    if versions_url == None:
        versions_url = "https://api.github.com/repos/{}/{}/releases?per_page=100".format(
            owner,
            repo,
        )

    def fetch_versions(_ctx):
        return {
            "__type": "fetch_json_versions",
            "url": versions_url,
            "transform": "rez_bundle_versions",
            "tool": tool,
        }

    def download_url(ctx, version):
        unsupported_reason = _lookup_target(ctx, unsupported_targets)
        triple = _lookup_target(ctx, targets)
        platform_key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
        if unsupported_reason or not triple:
            return {
                "__type": "rez_bundle_asset",
                "supported": False,
                "platform": platform_key,
                "rez_platform": _rez_platform(ctx),
                "architecture": _rez_arch(ctx),
                "target": ctx.platform.target,
                "reason": unsupported_reason or _unsupported_reason(ctx, tool, unsupported_targets),
            }

        release_tag = "{}-{}".format(tool, version)
        asset_name = "{}-{}-{}.rez.tar.zst".format(tool, version, triple)
        url = github_asset_url(owner, repo, release_tag, asset_name)
        return {
            "__type": "rez_bundle_asset",
            "supported": True,
            "tool": tool,
            "version": version,
            "platform": platform_key,
            "rez_platform": _rez_platform(ctx),
            "architecture": _rez_arch(ctx),
            "target": ctx.platform.target,
            "triple": triple,
            "release_tag": release_tag,
            "asset_name": asset_name,
            "url": url,
            "checksum_url": url + ".sha256",
            "index_url": github_asset_url(owner, repo, release_tag, "index.json"),
            "bundle_schema_version": bundle_schema_version,
        }

    def install_layout(ctx, version):
        rez_platform = _rez_platform(ctx)
        rez_arch = _rez_arch(ctx)
        return {
            "type": "archive",
            "required_paths": [
                "{}/{}/package.py".format(tool, version),
                "platform/{}/package.py".format(rez_platform),
                "arch/{}/package.py".format(rez_arch),
            ],
        }

    def rez_bundle(ctx, version):
        runtime_name = ctx.runtime_name if ctx.runtime_name else tool
        program = programs.get(runtime_name, runtime_name)
        enabled = published and (len(programs) == 0 or runtime_name in programs)
        rez_platform = _rez_platform(ctx)
        rez_arch = _rez_arch(ctx)
        asset = download_url(ctx, version)
        return {
            "__type": "rez_bundle_request",
            "enabled": enabled,
            "bundle_schema_version": bundle_schema_version,
            "repository": ".",
            "requirements": [
                "{}-{}".format(tool, version),
                "platform-{}".format(rez_platform),
                "arch-{}".format(rez_arch),
            ],
            "program": program,
            "asset": asset,
        }

    return {
        "fetch_versions": fetch_versions,
        "download_url": download_url,
        "install_layout": install_layout,
        "rez_bundle": rez_bundle,
    }
