## Summary

Add a new VSCode provider to enable installation and management of Visual Studio Code through vx.

## Changes

### New Files

- `crates/vx-providers/vscode/` - New VSCode provider crate
  - `src/provider.rs` - Provider metadata implementation
  - `src/runtime.rs` - Runtime implementation with version fetching
  - `src/config.rs` - URL builder for download URLs
  - `tests/runtime_tests.rs` - 21 unit tests

### Modified Files

- `crates/vx-runtime/src/impls.rs` - Enhanced extract functions with:
  - URL fragment hints (`#.zip`) for file type detection
  - Magic bytes detection (ZIP, GZIP) for archives without extensions
- `crates/vx-cli/src/registry.rs` - Register VSCode provider
- Updated snapshot tests for new provider count

## Features

- **Runtime names**: `code`, `vscode`, `vs-code`, `visual-studio-code`
- **Version source**: Official VSCode SHA API with GitHub releases fallback
- **Platform support**: Windows, macOS, Linux (x64, arm64)
- **Archive handling**: Automatic ZIP extraction with magic bytes detection

## Testing

```bash
# Install VSCode
vx install code 1.107.1

# List installed versions
vx list code

# Locate executable
vx where code
```

## Checklist

- [x] Code compiles without errors
- [x] All 21 unit tests pass
- [x] Integration tests pass
- [x] Manual testing verified
