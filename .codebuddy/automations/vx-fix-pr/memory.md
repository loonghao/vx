# vx-fix-pr Automation Memory

## 2026-04-01 (第一次修复 - 错误目标分支)

**Problem**: CI failed for watchexec provider with HTTP 404 on Windows.
- URL attempted: `watchexec-2.5.1-x86_64-pc-windows-msvc.tar.xz` → 404
- Root cause: watchexec releases use `.zip` on Windows but `.tar.xz` on Linux/macOS. The provider.star had the extension hardcoded as `.tar.xz` for all platforms.

**Fix**: Added a custom `download_url` function in `crates/vx-providers/watchexec/provider.star` that selects `.zip` on Windows and `.tar.xz` on Linux/macOS. All other functions continue to use the `github_rust_provider` template.

**Commit**: `aeb781a6` - `fix(watchexec): use .zip on Windows, .tar.xz on Linux/macOS`
**Branch**: `pr-734` pushed to `origin/pr-734`

❌ **Error**: This fix was applied to the wrong branch. PR #734's actual HEAD is `new-providers`, not `pr-734`. The `pr-734` local branch is a separate tracking branch.

---

## 2026-04-01 (第二次修复 - 应用到正确分支)

**Problem**: CI still failing with the same 404 error because the fix was on `pr-734` branch but PR #734 uses the `new-providers` branch as HEAD.

**Investigation**:
- PR #734 head: `new-providers` branch (SHA: `25ea8695`)
- `origin/new-providers` watchexec/provider.star was the unfixed version (`.tar.xz` hardcoded, no custom `download_url`)
- The worktree for `new-providers` is at `C:/github/vx-new-providers`
- Confirmed `watchexec-2.5.1-x86_64-pc-windows-msvc.zip` EXISTS at GitHub (verified via official download page https://watchexec.github.io/downloads/watchexec/2.5.1/)
- GitHub API only returns first 30 assets, Windows ZIP appears later in the list

**Fix**:
1. Updated `C:/github/vx-new-providers` worktree via `git pull origin new-providers`
2. Copied the fixed `provider.star` to the worktree
3. Committed and pushed: `aeb12c0b` on `new-providers` → `origin/new-providers`

**Key insight**: Always check which branch is the PR HEAD before applying fixes. In this repo, `pr-734` is a local tracking branch, while the actual PR #734 uses `new-providers` as its HEAD branch.

**Commit**: `aeb12c0b` - `fix(watchexec): use .zip on Windows, .tar.xz on Linux/macOS`
**Branch**: `new-providers` pushed to `origin/new-providers` ✅
