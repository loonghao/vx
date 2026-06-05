# @vx//stdlib:smart_detect.star
# Asset scoring and smart detection for GitHub releases
#
# Pure computation — no I/O. The Rust runtime fetches release assets via
# GitHub API, then calls into this module to score and select the best match.
#
# Usage:
#   load("@vx//stdlib:smart_detect.star", "score_asset", "detect_best_asset")

# ── Hard exclusion keywords (checked case-insensitive in filename) ──
_EXCLUDE_KEYWORDS = [
    "checksum", "sha256", "sha512", "md5", "sha1",
    "sbom", "attestation", "spdx",
    "source", "src.tar", "-src.",
    ".deb", ".rpm", ".apk", ".msi", ".pkg", ".dmg", ".appimage",
    ".sig", ".asc", ".pem",
]

# ── OS alias mapping: canonical OS → list of filename substrings ──
_OS_ALIASES = {
    "windows": ["windows", "win64", "win32", "windows-x64", "pc-windows", "win"],
    "darwin":  ["darwin", "macos", "macosx", "mac", "apple-darwin", "osx"],
    "linux":   ["linux", "unknown-linux", "linux-gnu", "linux-musl"],
}

# ── Arch alias mapping: canonical arch → list of filename substrings ──
_ARCH_ALIASES = {
    "x86_64":  ["x86_64", "x64", "amd64", "x86-64", "win64"],
    "aarch64": ["arm64", "aarch64", "armv8", "arm64-v8a"],
    "i686":    ["x86", "i686", "i386", "386", "win32"],
}

# ── Universal / multi-arch markers ──
_UNIVERSAL_MARKERS = ["all", "any", "portable", "multi", "universal", "fat"]

# ── Libc detection markers ──
_LIBC_MUSL = ["musl", "static", "alpine"]
_LIBC_GNU  = ["gnu", "glibc", "gnueabihf"]

# ── Platform-aware format preferences: {os: {extension: points, ...}} ──
_FORMAT_PREFS = {
    "linux":   {".tar.gz": 15, ".tar.xz": 10, ".tar.bz2": 5, ".tgz": 5, ".zip": 2},
    "darwin":  {".tar.gz": 15, ".tar.xz": 10, ".zip": 5, ".tar.bz2": 2},
    "windows": {".zip": 15, ".tar.gz": 10, ".7z": 5, ".tar.xz": 2, ".tar.bz2": 2},
}

# ── Keyword bonus table ──
_KEYWORD_BONUS = {
    "static": 3,
    "portable": 3,
    "standalone": 3,
}

# ── Vx platform → canonical OS for asset matching ──
def _canonical_os(ctx):
    """Map vx platform.os to the canonical OS used in asset filenames."""
    os = ctx.platform.os
    if os == "macos":
        return "darwin"
    return os

# ── Vx arch → canonical arch for asset matching ──
def _canonical_arch(ctx):
    """Map vx platform.arch to canonical arch used in asset filenames."""
    arch = ctx.platform.arch
    if arch == "x64":
        return "x86_64"
    if arch == "arm64":
        return "aarch64"
    if arch == "x86":
        return "i686"
    return arch

# ── Helpers ──

def _lc(name):
    return name.lower()

def _contains(name_lower, markers):
    """Check if a lowercased name contains any of the given marker substrings."""
    for m in markers:
        if m in name_lower:
            return True
    return False

def _find_alias(name_lower, alias_map):
    """Find the canonical key whose alias list matches the name.

    Returns (canonical_key, is_full_match) where is_full_match means
    the longest alias matched (preferring full token matches over partial).
    """
    best_key = None
    best_score = 0
    for key, aliases in alias_map.items():
        for alias in aliases:
            if alias in name_lower:
                # Score: prefer longer alias matches (more specific)
                score = len(alias)
                if score > best_score:
                    best_score = score
                    best_key = key
    return best_key

# ── Scoring sub-functions ──

def _score_os(name_lower, ctx, target_os):
    """Score OS match. Returns (points, is_mismatch) where mismatch means
    the asset matches a different OS entirely."""
    detected_os = _find_alias(name_lower, _OS_ALIASES)

    if detected_os == None:
        return (0, False)  # No OS detected, neutral

    if detected_os == target_os:
        # Full alias match
        # Check if it's a substring match within a compound name
        return (35, False)

    # Detected a different OS — this asset is for another platform
    return (0, True)


def _score_arch(name_lower, ctx, target_arch):
    """Score arch match."""
    # Check universal/multi-arch markers first
    if _contains(name_lower, _UNIVERSAL_MARKERS):
        return 15  # Universal binary, neutral score

    detected_arch = _find_alias(name_lower, _ARCH_ALIASES)

    if detected_arch == None:
        return 0

    if detected_arch == target_arch:
        return 30  # Full match

    return 0  # Different arch, doesn't match


def _score_libc(name_lower, ctx, linux_libc):
    """Score libc preference on Linux. Non-Linux always +15."""
    if ctx.platform.os != "linux":
        return 15

    has_musl = _contains(name_lower, _LIBC_MUSL)
    has_gnu  = _contains(name_lower, _LIBC_GNU)

    if linux_libc == "musl":
        if has_musl:
            return 15
        if has_gnu:
            return 5  # gnu asset on musl-preferred system
        return 8  # No libc marker detected

    if linux_libc == "gnu":
        if has_gnu:
            return 15
        if has_musl:
            return 5
        return 8

    return 8  # Unknown libc preference


def _score_format(name_lower, ctx, target_os):
    """Score archive format preference."""
    prefs = _FORMAT_PREFS.get(target_os, {})

    for ext, points in prefs.items():
        if name_lower.endswith(ext):
            return points

    # Acceptable format not in preference list
    if name_lower.endswith(".tar.gz"):
        return 2
    if name_lower.endswith(".tar.xz"):
        return 2
    if name_lower.endswith(".tar.bz2"):
        return 2
    if name_lower.endswith(".tgz"):
        return 2
    if name_lower.endswith(".zip"):
        return 2
    if name_lower.endswith(".7z"):
        return 2

    return 0


def _score_keywords(name_lower):
    """Score keyword bonuses, capped at 5 total."""
    bonus = 0
    for keyword, points in _KEYWORD_BONUS.items():
        if keyword in name_lower:
            bonus = bonus + points
    if bonus > 5:
        bonus = 5
    return bonus


def _version_appears(name_lower, version):
    """Check that the version appears in the filename (after stripping 'v' prefix)."""
    v = version
    if v.startswith("v"):
        v = v[1:]
    return v in name_lower


# ── Public API ──

def score_asset(name, ctx, version, linux_libc = "musl", extra_excludes = None):
    """Score a single asset filename against the current platform.

    Args:
        name:          Asset filename (e.g. "ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz")
        ctx:           Provider context with .platform.os and .platform.arch
        version:       Version string (e.g. "14.1.1" or "v14.1.1")
        linux_libc:    "musl" (default) or "gnu"
        extra_excludes: Optional list of extra exclusion substrings

    Returns:
        None if the asset is excluded (hard-excluded, version missing, OS mismatch).
        Otherwise a dict: {"os_score": int, "arch_score": int, "libc_score": int,
                           "format_score": int, "keyword_score": int, "total": int}
    """
    name_lower = _lc(name)

    # ── Hard exclusion check ──
    for kw in _EXCLUDE_KEYWORDS:
        if kw in name_lower:
            return None

    if extra_excludes != None:
        for ex in extra_excludes:
            if ex.lower() in name_lower:
                return None

    # ── Version must appear ──
    if not _version_appears(name_lower, version):
        return None

    target_os = _canonical_os(ctx)
    target_arch = _canonical_arch(ctx)

    # ── OS scoring ──
    os_score, os_mismatch = _score_os(name_lower, ctx, target_os)
    if os_mismatch:
        return None  # Exclude assets for a different OS

    # ── Arch scoring ──
    arch_score = _score_arch(name_lower, ctx, target_arch)

    # ── Libc scoring ──
    libc_score = _score_libc(name_lower, ctx, linux_libc)

    # ── Format scoring ──
    format_score = _score_format(name_lower, ctx, target_os)

    # ── Keyword bonus ──
    keyword_score = _score_keywords(name_lower)

    total = os_score + arch_score + libc_score + format_score + keyword_score

    return {
        "os_score":      os_score,
        "arch_score":    arch_score,
        "libc_score":    libc_score,
        "format_score":  format_score,
        "keyword_score": keyword_score,
        "total":         total,
    }


def _tie_break_key(item):
    """Sort key for tie-breaking: (score desc, size asc, name_length asc, name asc)."""
    score = item.get("total", 0)
    size  = item.get("size", 0)
    name  = item.get("name", "")
    # Higher score first, smaller size first, shorter name first, alphabetical
    return (-score, size, len(name), name)


def detect_best_asset(assets, ctx, version,
                      threshold = 40,
                      linux_libc = "musl",
                      extra_excludes = None):
    """Score all assets and return the best match.

    Args:
        assets:         List of asset dicts, each with "name", "size", "browser_download_url"
        ctx:            Provider context with .platform.os and .platform.arch
        version:        Version string
        threshold:      Minimum total score to accept (default: 40)
        linux_libc:     "musl" (default) or "gnu"
        extra_excludes: Optional list of extra exclusion substrings

    Returns:
        The best asset dict (with "score" and "scores" fields added) if score >= threshold.
        None if no asset meets the threshold.
    """
    candidates = []

    for asset in assets:
        # Assets are Starlark structs (converted from JSON by the Rust layer).
        # Use getattr for safe field access with defaults.
        name = getattr(asset, "name", "")
        if not name:
            continue

        scores = score_asset(name, ctx, version, linux_libc, extra_excludes)
        if scores == None:
            continue

        total = scores.get("total", 0)
        if total < threshold:
            continue

        candidates.append({
            "name":                 name,
            "size":                 getattr(asset, "size", 0),
            "browser_download_url": getattr(asset, "browser_download_url", ""),
            "score":                total,
            "scores":               scores,
        })

    if len(candidates) == 0:
        return None

    # Sort: highest score first, tie-break by size → name length → alphabetical
    candidates = sorted(candidates, key = _tie_break_key)
    return candidates[0]
