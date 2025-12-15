# Test: vx switch (no tool)

Verify that `vx switch` without tool@version shows error.

```console
$ vx switch
? failed
error: the following required arguments were not provided:
  <TOOL_VERSION>

Usage: vx[EXE] switch <TOOL_VERSION>

For more information, try '--help'.

```
