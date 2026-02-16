# VX Provider Creation Checklist

Use this checklist to ensure all steps are completed when creating a new provider.

## License Compliance (MUST DO FIRST)

- [ ] Check the upstream tool's license (GitHub repo → LICENSE file)
- [ ] Verify license is NOT in blocked list (AGPL-3.0, SSPL, CC BY-NC)
- [ ] Determine SPDX license identifier (e.g., MIT, Apache-2.0, GPL-2.0)
- [ ] If GPL/LGPL/BSL: confirm vx only downloads/executes (no linking), add `license_note`
- [ ] If proprietary: confirm tool is free to download/use, add `license_note`

## Pre-Implementation

- [ ] Identify the tool's GitHub repository (owner/repo)
- [ ] Check the release asset naming pattern
- [ ] Identify supported platforms (Windows, macOS, Linux, ARM64, x86_64)
- [ ] Determine archive format (zip, tar.gz, git clone, etc.)
- [ ] Check if archive extracts to subdirectory or directly
- [ ] Identify the executable name and extension pattern:
  - Standard `.exe` on Windows? (default)
  - Uses `.cmd` on Windows? (npm, yarn, npx)
  - Different executable name than runtime name?
- [ ] **Determine if project analyzer integration is needed** (for language/ecosystem tools)

## Files to Create

### Provider Crate (`crates/vx-providers/{name}/`)

- [ ] `Cargo.toml` - Package configuration
- [ ] `src/lib.rs` - Module exports and `create_provider()`
- [ ] `src/provider.rs` - Provider trait implementation
- [ ] `src/runtime.rs` - Runtime trait implementation
- [ ] `src/config.rs` - URL builder and platform config
- [ ] `tests/runtime_tests.rs` - Unit tests

### Project Analyzer (Optional - for language/ecosystem tools)

- [ ] `crates/vx-project-analyzer/src/languages/{lang}/mod.rs` - Module exports
- [ ] `crates/vx-project-analyzer/src/languages/{lang}/analyzer.rs` - Analyzer implementation
- [ ] `crates/vx-project-analyzer/src/languages/{lang}/dependencies.rs` - Dependency parsing
- [ ] `crates/vx-project-analyzer/src/languages/{lang}/rules.rs` - Script detection rules
- [ ] `crates/vx-project-analyzer/src/languages/{lang}/scripts.rs` - Script parsing

## Files to Modify

### Workspace Configuration

- [ ] `Cargo.toml` (root)
  - [ ] Add to `[workspace]` members
  - [ ] Add to `[workspace.dependencies]`

### CLI Integration

- [ ] `crates/vx-cli/Cargo.toml`
  - [ ] Add `vx-provider-{name} = { workspace = true }`

- [ ] `crates/vx-cli/src/registry.rs`
  - [ ] Add `registry.register(vx_provider_{name}::create_provider());`

### Project Analyzer Integration (if applicable)

- [ ] `crates/vx-project-analyzer/src/languages/mod.rs`
  - [ ] Add `mod {lang};`
  - [ ] Add `pub use {lang}::{Lang}Analyzer;`
  - [ ] Add to `all_analyzers()` function

### Snapshot Tests

- [ ] `tests/cmd/plugin/plugin-stats.md`
  - [ ] Increment "Total providers" count
  - [ ] Increment "Total runtimes" count (by number of runtimes in provider)
  - [ ] Add provider line: `{name} (N runtimes)`

- [ ] `tests/cmd/search/search.md`
  - [ ] Add runtime line: `{name} - {Description}`

### Documentation (Required)

- [ ] `docs/tools/{category}.md` - Add tool to appropriate category doc
  - DevOps tools: `docs/tools/devops.md`
  - Cloud CLI: `docs/tools/cloud.md`
  - Build tools: `docs/tools/build-tools.md`
  - AI tools: `docs/tools/ai.md`
  - Scientific/HPC: `docs/tools/scientific.md`
  - Code quality: `docs/tools/quality.md`
  - Other: `docs/tools/other.md`

- [ ] `docs/zh/tools/{category}.md` - Add Chinese version

Documentation should include:
- Tool description
- Installation command: `vx install {name} latest`
- Common usage examples
- Key features (if applicable)
- Platform support notes (if applicable)

## Verification Commands

```bash
# Check provider compiles
cargo check -p vx-provider-{name}

# Run provider tests
cargo test -p vx-provider-{name}

# If analyzer was added
cargo test -p vx-project-analyzer

# Check full workspace
cargo check

# Run all tests including snapshots
cargo test

# Format code
cargo fmt

# Run clippy
cargo clippy -p vx-provider-{name}
```

## Common Issues

### Compilation Errors

1. **Missing import**: Ensure all `use` statements are correct
2. **Trait not implemented**: Check all required trait methods
3. **Type mismatch**: Verify return types match trait definitions

### Test Failures

1. **Snapshot mismatch**: Update counts in `plugin-stats.md` and `search.md`
2. **URL format**: Verify download URL matches actual release assets
3. **Platform support**: Ensure all platform combinations are handled

### Runtime Errors

1. **Version fetch fails**: Check GitHub API URL and response format
2. **Download fails**: Verify URL construction matches release assets
3. **Verification fails**: Check executable path in archive

### Analyzer Errors

1. **Detection fails**: Check config file patterns in `detect()`
2. **Scripts not detected**: Verify rule triggers and excludes
3. **Priority conflicts**: Ensure higher priority rules are listed first

## Manifest Error Troubleshooting

### Common provider.toml Parse Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `unknown variant "cpp"` for ecosystem | Unsupported ecosystem value | Use one of: `nodejs`, `python`, `rust`, `go`, `ruby`, `java`, `dotnet`, `devtools`, `container`, `cloud`, `ai`, `cpp`, `zig`, `system` |
| `unknown variant "git-clone"` for download_type | kebab-case not supported | Use `git_clone` (snake_case) |
| `invalid type: map, expected a string` for `when` | Wrong constraint syntax | Use `when = "*"` with separate `platform` field, not `when = { os = "windows" }` |
| `missing field "name"` | Required field missing | Add the required `name` field to provider or runtime section |
| `invalid type: integer, expected a string` | Unquoted number | Quote version numbers: `version = "1.0"` not `version = 1.0` |

### Build Error Categories

- **`NoFactory` (manifest-only)**: Provider has `provider.toml` but no Rust implementation. Expected during early development.
- **`FactoryFailed`**: Rust factory function failed. Check the provider's `create_provider()` function.
- **Real errors (0 expected)**: Configuration issues that need immediate attention.

### Debug Command

```bash
# See detailed manifest parse errors and build diagnostics
cargo run -- --debug list 2>&1 | grep -E "registered|errors|manifest-only|Failed"
```

## Release Asset Patterns

Common patterns for GitHub release assets:

```
# Standard Rust project
{name}-{version}-{target}.{ext}
{name}-v{version}-{target}.{ext}

# With 'v' prefix
v{version}/{name}-{target}.{ext}

# Platform-specific naming
{name}-{version}-linux-amd64.tar.gz
{name}-{version}-darwin-arm64.tar.gz
{name}-{version}-windows-amd64.zip

# Architecture variations
x86_64, amd64, x64
aarch64, arm64
```

## Target Triple Reference

| OS | Arch | Common Triple |
|----|------|---------------|
| Windows | x86_64 | x86_64-pc-windows-msvc |
| Windows | aarch64 | aarch64-pc-windows-msvc |
| macOS | x86_64 | x86_64-apple-darwin |
| macOS | aarch64 | aarch64-apple-darwin |
| Linux | x86_64 | x86_64-unknown-linux-musl |
| Linux | aarch64 | aarch64-unknown-linux-musl |
| Linux | arm | arm-unknown-linux-musleabihf |

## Executable Path Configuration

The Runtime trait provides a layered approach for configuring executable paths. Most providers only need to override 1-2 methods:

### Method Hierarchy

| Method | Default | When to Override |
|--------|---------|------------------|
| `executable_name()` | `self.name()` | Executable name differs from runtime name |
| `executable_extensions()` | `&[".exe"]` | Tool uses `.cmd`/`.bat` on Windows |
| `executable_dir_path()` | `None` (root) | Executable is in a subdirectory |
| `executable_relative_path()` | Auto-generated | Complex cases only (rarely needed) |

### Common Patterns

**Simple tool (executable in root):**
```rust
// No overrides needed - defaults work
// Result: {name} (Unix) or {name}.exe (Windows)
```

**Tool with subdirectory:**
```rust
fn executable_dir_path(&self, version: &str, _platform: &Platform) -> Option<String> {
    Some(format!("{name}-{}", version))
}
// Result: {name}-{version}/{name} or {name}-{version}/{name}.exe
```

**Node.js ecosystem tools (npm, yarn, npx):**
```rust
fn executable_extensions(&self) -> &[&str] {
    &[".cmd", ".exe"]  // .cmd takes priority
}

fn executable_dir_path(&self, version: &str, platform: &Platform) -> Option<String> {
    let dir = format!("node-v{}-{}", version, platform.as_str());
    if platform.is_windows() {
        Some(dir)
    } else {
        Some(format!("{}/bin", dir))
    }
}
// Windows: node-v22.0.0-win-x64/npm.cmd
// Linux: node-v22.0.0-linux-x64/bin/npm
```

**Different executable name:**
```rust
fn executable_name(&self) -> &str {
    "python3"  // Runtime name is "python"
}
```

## ScriptRule Priority Reference

| Priority | Use Case | Examples |
|----------|----------|----------|
| 100 | Task runners that manage other tools | nox, tox, just, make |
| 90 | Secondary task runners | taskfile |
| 50 | Default/standalone tools | pytest, ruff, cargo, npm |

## Unit Test Requirements

Each provider must have comprehensive unit tests in `tests/runtime_tests.rs`:

### Required Tests

```rust
// Provider tests
#[test] fn test_provider_name()
#[test] fn test_provider_description()
#[test] fn test_provider_supports()
#[test] fn test_provider_runtimes()
#[test] fn test_provider_get_runtime()

// Runtime tests
#[test] fn test_runtime_name()
#[test] fn test_runtime_description()
#[test] fn test_runtime_ecosystem()
#[test] fn test_runtime_metadata()

// URL builder tests (using rstest for parameterized tests)
#[rstest] fn test_target_triple(os, arch, expected)
#[rstest] fn test_archive_extension(os, expected)
#[rstest] fn test_executable_name(os, expected)
#[rstest] fn test_executable_relative_path(os, arch, expected)
#[test] fn test_download_url_format()
```

### Analyzer Tests (if applicable)

```rust
// Detection tests
#[tokio::test] async fn test_{lang}_project_detection()
#[tokio::test] async fn test_{lang}_not_detected_without_config()

// Dependency tests
#[tokio::test] async fn test_{lang}_dependencies()

// Script tests
#[tokio::test] async fn test_{lang}_scripts()
#[tokio::test] async fn test_{lang}_script_priority()

// Required tools tests
#[test] fn test_{lang}_required_tools()
```

### Test Coverage Guidelines

1. Test all supported platforms (Windows, macOS, Linux)
2. Test both x86_64 and ARM64 architectures where applicable
3. Test version string handling (with/without 'v' prefix)
4. Test edge cases (unsupported platforms should return None)
5. Test script rule priority ordering
6. Test explicit vs detected script merging
