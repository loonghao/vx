# Managed command shims

Use a command shim when a shell, editor or other application needs a plain
command such as `codex`, while vx should prepare its execution environment.

## Run an agent, then expose its command

```bash
vx codex --version
vx shim add codex
codex --version
```

`vx codex` installs and runs the package in vx's isolated environment without
publishing a global command. `vx shim add codex` explicitly creates the entry
point. The wrapper invokes the absolute vx executable and restores the `VX_HOME`
used at creation, so managed dependencies remain available outside a vx shell.

The supported commands are `vx shim add`, `list`, `remove`, `sync` and `path`.
The `add` subcommand is required; neither `vx shim codex` nor bare `vx shim`
creates wrappers. Use `vx shim --help` to check the installed interface.

Creating a shim does not run or install its target. If you create it before
running the agent, the first invocation can install the package and dependencies.

## Choose the command and destination

```bash
vx shim add codex --as codex-vx
vx shim add "npm:@openai/codex::codex" --as codex-vx
vx shim add codex --dir "/absolute/path/to/shim-bin"
vx shim list --json
vx shim path
```

Provider aliases resolve to their package request. Explicit package syntax can
select a different executable through `::executable`; `--as` chooses the name
that callers use. Package versions follow the package execution rules: creating
a shim alone does not switch an already installed package to another version.

By default vx writes to `$VX_HOME/bin` and the directory holding the running vx
executable. The latter is normally on PATH after installation. `--dir` is
repeatable and replaces those defaults. Relative values are resolved when `add`
runs and stored as absolute paths. For an isolated
smoke test, use both a private `VX_HOME` and a private `--dir`, because `VX_HOME`
alone does not isolate writes beside the vx executable.

`add` checks PATH collisions and every prospective destination, including both
Windows variants. Unrelated files, edited wrappers and symbolic links are
preserved. Use a distinct alias or directory if a name is occupied. `--force`
permits intentional PATH shadowing; it does not override file ownership.

## Check shell resolution

| Caller | Generated file | Inspect resolution |
|---|---|---|
| Windows PowerShell | `<name>.cmd` | `Get-Command codex -All`; `where.exe codex` |
| Windows cmd.exe | `<name>.cmd` | `where codex` |
| Windows Git Bash / MSYS2 | `<name>` | `type -a codex`; `command -v codex` |
| Linux / macOS | `<name>` | `command -v codex` |

Windows generates batch and POSIX wrappers together, without a `.ps1` file.
Both forward arguments and the exit code. Native process launchers that cannot
execute scripts should invoke vx with separate arguments instead.

PATH membership alone does not prove which command wins: aliases, functions and
earlier directories can take precedence. `vx shim path` reports default
directories; use `vx shim list --json` for custom destinations. After changing
PATH, reopen the shell or application; Bash users can also clear cached paths
with `hash -r`.

vx does not change persistent PATH or shell configuration. To expose a custom
directory for the current session, use the appropriate shell syntax:

```powershell
$env:PATH = "<absolute-shim-directory>;$env:PATH"
```

```sh
export PATH="<absolute-shim-directory>:$PATH"
```

## Refresh and remove

```bash
vx shim list --json
vx shim sync
vx shim remove codex
```

`sync` refreshes registered wrappers against the running vx executable in the
current `VX_HOME`. It keeps their recorded directories and recreates missing
variants, while rejecting file conflicts. Older records without a bound home
are upgraded during refresh. It does not upgrade the target packages or publish
unregistered commands.

`list` checks whether recorded files match their expected wrappers. This reports
wrapper state, not target execution or PATH precedence. The registry lives at
`$VX_HOME/config/command-shims.json`; unreadable or malformed content produces an
error instead of silently discarding records.

`remove` removes the entry and its unchanged, owned files. Modified files,
symlinks and unrelated replacements are left intact and reported; `--force`
does not bypass that protection. The package stays installed, and running
`vx codex` afterwards does not recreate the command shim.

## Explicit package installation

For compatibility, `vx install codex` and `vx pkg install npm:@openai/codex`
install the package and publish its executables. They use the same command shim
registry, so `vx shim list`, `sync` and `remove` manage those entries too.

`vx pkg shim-update` explicitly publishes installed package executables. It
preserves unrelated command shims and files instead of treating the vx binary
directory as a disposable shim directory. Use `vx shim sync` to refresh only
commands that are already registered. Uninstalling a package removes its
registered package entry points.

Older package wrappers without a command shim registry entry are preserved.
They can cause a collision during `add` or explicit installation. Use a different
`--as` name and, if needed, a private `--dir` for the new shim. vx does not automatically adopt
or remove an unregistered wrapper based on its name or contents.

If a registered package's executable has gone missing, vx reports the package
and a `vx pkg install <ecosystem>:<package> --force` repair command. It does not
fall back to a global wrapper and recursively call itself.

## Related

- [Command shim CLI reference](../cli/shim.md)
- [AI coding agents](../tools/ai.md#ai-coding-agents)
- [Implicit package execution](../cli/implicit-package-execution.md)
- [RFC 0042](../rfcs/0042-platform-command-shims.md)
