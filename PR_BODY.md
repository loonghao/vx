## Problem

The `vx sync` and `vx dev` commands failed when installing tools because they passed version numbers as separate arguments to the install command.

Example error:
```
✗ Failed to install python@3.11
  ✗ Tool '3.11' is not supported by vx
```

## Root Cause

The install command handler expects `tool@version` format (e.g., `node@20`), but sync and dev commands passed two separate arguments: `['install', 'node', '20']`.

This caused the install command's `parse_tool_spec()` function to misinterpret the version number as a tool name.

## Solution

Modified both commands to format the arguments correctly:
- `sync.rs`: Changed from `cmd.args(['install', name, version])` to `cmd.args(['install', &format!('{}@{}', name, version)])`
- `dev/install.rs`: Same fix applied

## Testing

Added unit tests in `crates/vx-cli/tests/install_fix_tests.rs`:
- Test `parse_tool_spec` behavior
- Test various version formats
- Test tool@version format string generation

All tests pass.

## Additional Changes

Created `CLI_COMMAND_REUSE_ANALYSIS.md` documenting areas where CLI command logic can be consolidated but currently isn't. This provides a roadmap for future refactoring efforts.

## Files Changed

- `crates/vx-cli/src/commands/sync.rs` - Fixed install argument format
- `crates/vx-cli/src/commands/dev/install.rs` - Fixed install argument format
- `crates/vx-cli/tests/install_fix_tests.rs` - Added unit tests
- `CLI_COMMAND_REUSE_ANALYSIS.md` - Added analysis documentation
