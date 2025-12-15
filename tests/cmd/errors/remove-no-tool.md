# Test: vx remove (no tool)

Verify that `vx remove` without tool name shows error (remove is alias for uninstall).

```console
$ vx remove
? failed
error: the following required arguments were not provided:
  <TOOL>

Usage: vx[EXE] uninstall <TOOL> [VERSION]

For more information, try '--help'.

```
