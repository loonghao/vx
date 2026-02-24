#!/usr/bin/env python3
"""
Starlark (.star) syntax checker for vx provider scripts.

Checks for:
1. Syntax errors via Python ast.parse() — Starlark is a Python subset, so
   Python's parser catches indentation errors, unexpected tokens, etc.
2. Deprecated ctx["key"]["subkey"] dict access (should use ctx.key.subkey)
3. Double ctx access bug (ctx.ctx.xxx)
4. Common Starlark anti-patterns (print() usage, global mutable state, etc.)

Usage:
    python scripts/check_star_syntax.py [files...]
    python scripts/check_star_syntax.py           # checks all .star files
    python scripts/check_star_syntax.py --fix     # auto-fix fixable issues

Exit code:
    0 - no issues found
    1 - issues found
"""

import ast
import os
import re
import sys
import argparse
from typing import List, Tuple


# ---------------------------------------------------------------------------
# Issue types
# ---------------------------------------------------------------------------

class Issue:
    def __init__(self, path: str, line: int, col: int, code: str, message: str, fixable: bool = False):
        self.path = path
        self.line = line
        self.col = col
        self.code = code
        self.message = message
        self.fixable = fixable

    def __str__(self):
        fix_hint = " [fixable with --fix]" if self.fixable else ""
        return f"{self.path}:{self.line}:{self.col}: {self.code}: {self.message}{fix_hint}"


# ---------------------------------------------------------------------------
# Check 1: Real syntax check via Python ast.parse()
# Starlark is a strict subset of Python, so Python's parser catches:
#   - IndentationError (unexpected indentation blocks)
#   - SyntaxError (unexpected tokens, missing colons, etc.)
# ---------------------------------------------------------------------------

def check_syntax(path: str, content: str) -> List[Issue]:
    """Use Python's ast.parse() to detect real syntax/indentation errors."""
    issues = []
    # Check for BOM (U+FEFF) — Starlark parser rejects it
    if content.startswith('\ufeff'):
        issues.append(Issue(
            path=path,
            line=1,
            col=1,
            code="E001",
            message="BOM (U+FEFF) detected at start of file, remove it (UTF-8 without BOM required)",
            fixable=True,
        ))
        # Strip BOM before further parsing so we get real errors, not BOM noise
        content = content.lstrip('\ufeff')
    try:
        ast.parse(content, filename=path)
    except IndentationError as e:
        issues.append(Issue(
            path=path,
            line=e.lineno or 0,
            col=e.offset or 0,
            code="E001",
            message=f"indentation error: {e.msg}",
        ))
    except SyntaxError as e:
        issues.append(Issue(
            path=path,
            line=e.lineno or 0,
            col=e.offset or 0,
            code="E001",
            message=f"syntax error: {e.msg}",
        ))
    return issues


# ---------------------------------------------------------------------------
# Check 2: Deprecated ctx["key"]["subkey"] dict access
# ---------------------------------------------------------------------------

_CTX_DICT_PATTERN = re.compile(r'ctx\["([^"]+)"\]\["([^"]+)"\]')


def check_ctx_dict_access(path: str, lines: List[str]) -> List[Issue]:
    """Detect ctx['key']['subkey'] dict access that should be ctx.key.subkey."""
    issues = []
    for i, line in enumerate(lines, 1):
        if line.strip().startswith("#"):
            continue
        for m in _CTX_DICT_PATTERN.finditer(line):
            key1, key2 = m.group(1), m.group(2)
            issues.append(Issue(
                path=path,
                line=i,
                col=m.start() + 1,
                code="E002",
                message=f'deprecated ctx dict access ctx["{key1}"]["{key2}"], use ctx.{key1}.{key2}',
                fixable=True,
            ))
    return issues


# ---------------------------------------------------------------------------
# Check 3: Double ctx access bug (ctx.ctx.xxx)
# ---------------------------------------------------------------------------

_CTX_DOUBLE_PATTERN = re.compile(r'\bctx\.ctx\.')


def check_ctx_double_access(path: str, lines: List[str]) -> List[Issue]:
    """Detect ctx.ctx.xxx double access bug."""
    issues = []
    for i, line in enumerate(lines, 1):
        if line.strip().startswith("#"):
            continue
        for m in _CTX_DOUBLE_PATTERN.finditer(line):
            issues.append(Issue(
                path=path,
                line=i,
                col=m.start() + 1,
                code="E003",
                message="double ctx access ctx.ctx.xxx, should be ctx.xxx",
                fixable=True,
            ))
    return issues


# ---------------------------------------------------------------------------
# Check 4: Starlark anti-patterns
# ---------------------------------------------------------------------------

# Patterns that are valid Python but not valid/recommended Starlark
_ANTI_PATTERNS = [
    # import statements are not allowed in Starlark
    (re.compile(r'^\s*import\s+\w'), "E004", "import statement not allowed in Starlark, use load()", False),
    # from ... import ... not allowed
    (re.compile(r'^\s*from\s+\w.*\s+import\s+'), "E004", "from...import not allowed in Starlark, use load()", False),
    # class definitions not allowed in Starlark
    (re.compile(r'^\s*class\s+\w'), "E005", "class definition not allowed in Starlark", False),
    # global/nonlocal not allowed
    (re.compile(r'^\s*(global|nonlocal)\s+'), "E006", "global/nonlocal not allowed in Starlark", False),
    # yield not allowed
    (re.compile(r'\byield\b'), "E007", "yield not allowed in Starlark (no generators)", False),
    # lambda is allowed but warn about complex lambdas
    # print() is allowed in Starlark but should use fail() for errors
    # Only match print( at the start of a statement (not inside strings)
    (re.compile(r'^\s*print\s*\('), "W001", "prefer fail() over print() for error reporting in Starlark", False),
]


def check_anti_patterns(path: str, lines: List[str]) -> List[Issue]:
    """Detect Starlark anti-patterns."""
    issues = []
    for i, line in enumerate(lines, 1):
        stripped = line.strip()
        if stripped.startswith("#"):
            continue
        for pattern, code, message, fixable in _ANTI_PATTERNS:
            if pattern.search(line):
                issues.append(Issue(
                    path=path,
                    line=i,
                    col=1,
                    code=code,
                    message=message,
                    fixable=fixable,
                ))
    return issues


# ---------------------------------------------------------------------------
# Auto-fix
# ---------------------------------------------------------------------------

def fix_file(path: str, content: str) -> Tuple[str, int]:
    """Apply auto-fixes to file content. Returns (new_content, fix_count)."""
    fix_count = 0

    # Fix BOM (U+FEFF)
    if content.startswith('\ufeff'):
        content = content.lstrip('\ufeff')
        fix_count += 1

    def replace_ctx_dict(m):
        nonlocal fix_count
        fix_count += 1
        return f"ctx.{m.group(1)}.{m.group(2)}"

    content = _CTX_DICT_PATTERN.sub(replace_ctx_dict, content)

    def replace_ctx_double(m):
        nonlocal fix_count
        fix_count += 1
        return "ctx."

    content = _CTX_DOUBLE_PATTERN.sub(replace_ctx_double, content)

    return content, fix_count


# ---------------------------------------------------------------------------
# File checker
# ---------------------------------------------------------------------------

def check_file(path: str) -> List[Issue]:
    """Run all checks on a single file."""
    try:
        with open(path, encoding="utf-8") as f:
            content = f.read()
        lines = content.splitlines(keepends=True)
    except Exception as e:
        return [Issue(path=path, line=0, col=0, code="E000", message=f"cannot read file: {e}")]

    issues = []
    # E001: Real syntax check (catches indentation errors, unexpected tokens, etc.)
    issues.extend(check_syntax(path, content))
    # E002: Deprecated ctx dict access
    issues.extend(check_ctx_dict_access(path, lines))
    # E003: Double ctx access
    issues.extend(check_ctx_double_access(path, lines))
    # E004-W001: Starlark anti-patterns
    issues.extend(check_anti_patterns(path, lines))
    return issues


# ---------------------------------------------------------------------------
# File finder
# ---------------------------------------------------------------------------

def find_star_files(root: str) -> List[str]:
    """Find all .star files under root, excluding target/ and node_modules/."""
    result = []
    skip_dirs = {"target", "node_modules", ".git"}
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in skip_dirs]
        for fname in filenames:
            if fname.endswith(".star"):
                result.append(os.path.join(dirpath, fname))
    return sorted(result)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="Check Starlark (.star) files for syntax issues")
    parser.add_argument("files", nargs="*", help="Files to check (default: all .star files)")
    parser.add_argument("--fix", action="store_true", help="Auto-fix fixable issues (E002, E003)")
    parser.add_argument("--root", default=".", help="Root directory to search for .star files")
    args = parser.parse_args()

    files = args.files if args.files else find_star_files(args.root)

    if not files:
        print("No .star files found.")
        return 0

    total_issues = 0
    total_fixed = 0
    files_with_issues = []

    for fpath in files:
        if args.fix:
            try:
                with open(fpath, encoding="utf-8") as f:
                    content = f.read()
                new_content, fix_count = fix_file(fpath, content)
                if fix_count > 0:
                    with open(fpath, "w", encoding="utf-8") as f:
                        f.write(new_content)
                    total_fixed += fix_count
                    rel = os.path.relpath(fpath, args.root)
                    print(f"  fixed {fix_count} issue(s) in {rel}")
            except Exception as e:
                print(f"  ERROR fixing {fpath}: {e}", file=sys.stderr)

        issues = check_file(fpath)
        if issues:
            files_with_issues.append(fpath)
            for issue in issues:
                issue.path = os.path.relpath(issue.path, args.root)
                print(str(issue))
            total_issues += len(issues)

    print()
    if total_fixed > 0:
        print(f"Auto-fixed {total_fixed} issue(s).")
    if total_issues > 0:
        print(f"Found {total_issues} issue(s) in {len(files_with_issues)} file(s).")
        return 1
    else:
        print(f"Checked {len(files)} file(s). No issues found.")
        return 0


if __name__ == "__main__":
    sys.exit(main())
