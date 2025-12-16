# Test: vx invalid-command

Verify that `vx` handles invalid commands gracefully.

```console
$ vx invalid-command-xyz
? failed
[2m2025-12-16T09:08:42.387103Z[0m Executing: invalid-command-xyz
[2m2025-12-16T09:08:42.387174Z[0m Resolving runtime: invalid-command-xyz
[2m2025-12-16T09:08:42.399405Z[0m Auto-installing missing runtimes: ["invalid-command-xyz"]
[2m2025-12-16T09:08:42.399469Z[0m Installing: invalid-command-xyz
Error: Unknown runtime 'invalid-command-xyz'. Cannot auto-install.

```
