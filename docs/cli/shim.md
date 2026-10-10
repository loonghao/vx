# vx shim

Expose a Runtime or package executable as a plain command while vx manages its
execution environment.

```bash
vx codex --version          # install and run in an isolated package environment
vx shim add codex           # explicitly expose the codex command
codex --version             # runs through vx with its managed dependencies
```

Running `vx codex` does not create global command shims. `vx shim add` writes
wrappers that bind the absolute vx executable and the `VX_HOME` used to create
them. The command therefore keeps using that managed environment when invoked
from another shell or directory.

The public interface is `vx shim add/list/remove/sync/path`. Use `add` to create
an entry point; `vx shim codex` and bare `vx shim` are not creation commands.

## Subcommands

| Subcommand | Purpose |
|---|---|
| `add` | Create a shim for a Runtime or package command |
| `list` | Show registered shims (alias `ls`) |
| `remove` | Remove a registered shim (alias `rm`) |
| `sync` | Refresh registered shims against the current vx executable |
| `path` | Show default target directories and their PATH status |

## add

```bash
vx shim add jq
vx shim add git@2.53.0
vx shim add codex --as codex-vx
vx shim add "npm:@openai/codex::codex" --as codex-vx
vx shim add codex --dir "/absolute/path/to/shim-bin"
```

| Flag | Description |
|---|---|
| `--as <NAME>` | Command name to create instead of the default executable name. |
| `--dir <DIR>` | Destination directory, stored as an absolute path. Repeatable; replaces the defaults. |
| `-f, --force` | Permit intentional shadowing of another command found on PATH. |

Default destinations are `$VX_HOME/bin` and the directory containing the running
vx executable, with duplicate directories removed. The vx executable directory
is normally already on PATH after installation. `--dir` is useful when only one
particular shell or editor should see the command.

`add` does not install or run the target. The first command invocation can install
it through vx. Package aliases such as `codex` resolve to their package execution
request; explicit package syntax supports executables whose names differ from
the package name. See [AI coding agents](../tools/ai.md#ai-coding-agents).

### Collision protection

`add` checks command resolution on PATH and validates every destination file,
including both Windows variants. Existing registered wrappers can be refreshed;
unrelated files, modified wrappers and symbolic links are preserved. Use a
different `--as` name or destination to resolve a file conflict. `--force` permits
PATH shadowing, but does not bypass destination ownership checks.

## list

```bash
vx shim list
vx shim list --json
```

The text output reports each registered command and whether its recorded files
match the expected wrappers. JSON exposes the command request, launcher,
`vx_home`, directories and generated file paths. A complete wrapper does not
prove that its target package is installed or that it takes precedence on PATH.

## remove

```bash
vx shim remove codex
```

Removes the registry entry and its unchanged, owned wrapper files. Modified files
and unrelated replacements are reported and left intact. `--force` does not
bypass ownership checks. Removing a shim leaves its package installed; subsequent
`vx codex` execution does not recreate the shim.

## sync

```bash
vx shim sync
```

Refreshes registered shims against the running vx executable in the current
`VX_HOME`. It retains their recorded directories and recreates missing variants.
Conflicting files are preserved and reported. Older records without a bound home
are upgraded when refreshed. `sync` does not install packages, upgrade their
versions or publish commands for packages without a registered shim.

## path

```bash
vx shim path
```

Reports the default directories and their presence on the current process's
PATH. Custom `--dir` locations appear in `vx shim list --json`.

vx prints PATH hints but does not change shell profiles, persistent PATH or the
Windows registry. If a directory is not on PATH, add it to the environment of the
shell or application that needs the command.

## Package installation

Explicit `vx install codex` and `vx pkg install npm:@openai/codex` also publish
package executables for compatibility. Those shims use the same registry and
managed execution path as `vx shim add`.

`vx pkg shim-update` explicitly publishes installed package executables through
that shared lifecycle. Use `vx shim sync` when only existing registered commands
should be refreshed. Package maintenance preserves unrelated command shims and
files. Uninstalling a package removes its registered package entry points.

Older package wrappers that are absent from the command shim registry are left
unchanged. They may cause a collision when creating a new shim. Use a different
`--as` name and, if needed, a private `--dir`; vx does not assume ownership of
those files.

## Platform behaviour

| Platform | Generated files | Callers |
|---|---|---|
| Windows | `<name>.cmd` | cmd.exe and PowerShell |
| Windows | `<name>` | Git Bash, MSYS2 and Cygwin |
| Linux / macOS | `<name>` | POSIX shells |

Windows generates both batch and shell wrappers, without a `.ps1` file.
Arguments and the child exit code are forwarded to the caller. A native process
launcher that cannot execute scripts should invoke vx directly with separate
arguments.

The registry is stored at `$VX_HOME/config/command-shims.json`. An unreadable or
malformed registry is reported rather than replaced with an empty one.

## Related

- [Managed command shims](../guide/managed-command-shims.md)
- [RFC 0042 — Platform Command Shims](../rfcs/0042-platform-command-shims.md)
- [`vx global`](./global) — isolated package management
- [Implicit Package Execution](./implicit-package-execution)
