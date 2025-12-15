# Test: vx invalid-command

Verify that `vx` handles invalid commands gracefully.

```console
$ vx invalid-command-xyz
? failed
[..]Auto-installing missing runtimes: ["invalid-command-xyz"]
[..]Installing: invalid-command-xyz
[..]Unknown runtime 'invalid-command-xyz'. Cannot auto-install.

```
