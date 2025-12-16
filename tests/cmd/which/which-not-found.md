# Test: vx which (tool not installed)

Verify that `vx which` shows error when tool is not installed.

```console
$ vx which nonexistent-tool-xyz
? failed
✗ Tool 'nonexistent-tool-xyz' not found in vx-managed installations or system PATH
...

```
