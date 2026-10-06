# vx shim

Expose any Runtime as a plain command, so you can type `jq --version` instead of
`vx jq --version`.

`vx shim` writes a small wrapper script into a directory on your `PATH`. The
wrapper forwards every argument to `vx <runtime>`, which installs the runtime on
first use. This is the same wrapper people write by hand into
`~/.local/bin`, generated for you and per platform.

```bash
vx shim add jq
jq --version          # -> jq-1.8.1
```

> **RFC**: [RFC 0042 — Platform Command Shims](../rfcs/0042-platform-command-shims.md)

## Subcommands

| Subcommand | Purpose |
|---|---|
| `add` | Create a shim for a runtime |
| `list` | Show shims created by vx (alias `ls`) |
| `remove` | Delete a shim created by vx (alias `rm`) |
| `sync` | Rewrite every shim against the current `vx` executable |
| `path` | Show target directories and whether they are on `PATH` |

## add

```bash
vx shim add jq                       # create `jq`
vx shim add git@2.53.0               # pin a version behind the runtime name
vx shim add jq --as jqp              # different command name
vx shim add jq --dir ~/.local/bin    # choose the directory (repeatable)
vx shim add git --force              # allow shadowing the system git
```

| Flag | Description |
|---|---|
| `--as <NAME>` | Command name to create. Defaults to the runtime name without its `@version`. |
| `--dir <DIR>` | Directory to write into. Repeatable. Defaults to the vx bin directory and the directory holding the `vx` executable. |
| `-f, --force` | Overwrite an existing command that vx did not create. |

### Shadowing protection

If the name already resolves to a binary on `PATH` that vx did not create, `add`
refuses:

```text
$ vx shim add git
✗ 'git' already resolves to C:\Program Files\Git\mingw64\bin\git.exe on PATH.
  Re-run with --force to shadow it, or pick another name with --as.
```

Use `--as` to pick a different name, or `--force` if shadowing is what you want.
Shims vx created itself are always safe to overwrite.

## list

```bash
vx shim list
vx shim list --json      # machine-readable
```

```text
Command shims (2)
  git              -> vx git                  [ok]
    C:\Users\me\.vx\bin
  jq               -> vx jq                   [ok]
    C:\Users\me\.vx\bin
```

`incomplete` means one of the recorded files is missing — run `vx shim sync`.

## remove

```bash
vx shim remove jq
```

Only files that vx created are deleted. A hand-written wrapper sharing the name
is reported and left alone.

## sync

```bash
vx shim sync
```

Shims bake in the absolute path of the `vx` executable. After upgrading or
moving vx, `sync` rewrites every registered shim against the new location and
recreates any file that went missing.

## path

```bash
vx shim path
```

Prints each target directory, whether it is on `PATH`, and the exact command to
add it:

```text
Command shim directories
  C:\Users\me\.vx\bin      not on PATH
    Platform variants: <name>.cmd (cmd.exe, PowerShell), <name> (sh, Git Bash, MSYS2)
💡 $env:PATH = "C:\Users\me\.vx\bin;$env:PATH"
```

vx never edits your shell configuration or the Windows registry. Adding the
directory to `PATH` stays under your control.

## Platform behaviour

One `add` produces every file the platform needs:

| Platform | Files | Reachable from |
|---|---|---|
| Windows | `jq.cmd` | cmd.exe, PowerShell (`PATHEXT` resolves `jq` → `jq.cmd`) |
| Windows | `jq` | Git Bash, MSYS2, Cygwin |
| Linux / macOS | `jq` | every POSIX shell |

Windows needs two files because a POSIX shell script is invisible to cmd.exe and
PowerShell, while a batch file is unusable from Git Bash. Unix terminals all
speak `/bin/sh`, so one script is enough.

Generated scripts carry a `vx-shim` marker line and propagate the exit code of
the wrapped command.

## Safety

- `add` never overwrites a file that is not marked as vx-generated unless you
  pass `--force`.
- `remove` only deletes files recorded in the registry **and** carrying the
  `vx-shim` marker.
- The registry lives at `$VX_HOME/config/command-shims.json` and honours
  `VX_HOME`.

## Related

- [`vx global`](./global) — global package management, which uses the same
  shim-stacking layout.
- [Implicit Package Execution](./implicit-package-execution) — running packages
  without installing them first.
