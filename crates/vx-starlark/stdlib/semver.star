# @vx//stdlib:semver.star
# Semantic version utilities for vx provider scripts
#
# Usage:
#   load("@vx//stdlib:semver.star", "semver_compare", "semver_strip_v")

def semver_strip_v(version):
    """Strip the leading 'v' prefix from a version string.

    Args:
        version: Version string, e.g. "v1.2.3" or "1.2.3"

    Returns:
        Version string without 'v' prefix, e.g. "1.2.3"
    """
    if version.startswith("v") or version.startswith("V"):
        return version[1:]
    return version

def semver_parse(version):
    """Parse a version string into (major, minor, patch) tuple.

    Args:
        version: Version string like "1.2.3" or "v1.2.3"

    Returns:
        List of [major, minor, patch] integers, or [0, 0, 0] on failure
    """
    v = semver_strip_v(version)
    # Remove pre-release suffix (e.g. "1.2.3-rc1" -> "1.2.3")
    dash_idx = v.find("-")
    if dash_idx >= 0:
        v = v[:dash_idx]
    parts = v.split(".")
    result = []
    for part in parts[:3]:
        # Extract leading digits only
        digits = ""
        for ch in part:
            if ch >= "0" and ch <= "9":
                digits += ch
            else:
                break
        result.append(int(digits) if digits else 0)
    # Pad to 3 parts
    while len(result) < 3:
        result.append(0)
    return result

def semver_compare(a, b):
    """Compare two semantic version strings.

    Args:
        a: First version string
        b: Second version string

    Returns:
        -1 if a < b, 0 if a == b, 1 if a > b
    """
    pa = semver_parse(a)
    pb = semver_parse(b)
    for i in range(3):
        if pa[i] < pb[i]:
            return -1
        if pa[i] > pb[i]:
            return 1
    return 0

def semver_gt(a, b):
    """Return True if version a is greater than b."""
    return semver_compare(a, b) > 0

def semver_lt(a, b):
    """Return True if version a is less than b."""
    return semver_compare(a, b) < 0

def semver_gte(a, b):
    """Return True if version a is greater than or equal to b."""
    return semver_compare(a, b) >= 0

def semver_lte(a, b):
    """Return True if version a is less than or equal to b."""
    return semver_compare(a, b) <= 0

def semver_eq(a, b):
    """Return True if version a equals b."""
    return semver_compare(a, b) == 0

def semver_sort(versions, reverse = False):
    """Sort a list of version strings.

    Args:
        versions: List of version strings
        reverse: If True, sort in descending order (newest first)

    Returns:
        Sorted list of version strings
    """
    # Simple insertion sort (Starlark doesn't have sorted() with key)
    result = list(versions)
    for i in range(1, len(result)):
        key = result[i]
        j = i - 1
        if reverse:
            while j >= 0 and semver_compare(result[j], key) < 0:
                result[j + 1] = result[j]
                j -= 1
        else:
            while j >= 0 and semver_compare(result[j], key) > 0:
                result[j + 1] = result[j]
                j -= 1
        result[j + 1] = key
    return result
