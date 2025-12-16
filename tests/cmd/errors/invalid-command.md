# Test: vx invalid-command

Verify that `vx` handles invalid commands gracefully.

```console
$ vx invalid-command-xyz
? failed
...
Error: Unknown runtime 'invalid-command-xyz'. Cannot auto-install.

```
