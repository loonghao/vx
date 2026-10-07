# Managed command shims

Use a managed command shim when a shell, editor, hook, or third-party program
needs a plain command such as `codex`, while vx should remain responsible for
execution. Keep `vx <runtime>` in scripts you control when a plain command is
unnecessary. A shim forwards to the absolute path of the vx executable that
created it; it does not copy or install the target runtime.

## Check the installed capability

```sh
vx --version
vx --help
```

Only if the top-level help lists `shim`, inspect its subcommands:

```sh
vx shim --help
vx shim add --help
vx shim list --json
vx shim path
```

The interface introduced by [PR #1180](https://github.com/loonghao/vx/pull/1180)
is `add`, `list` (alias `ls`), `remove` (alias `rm`), `sync`, and `path`.
It does not accept `vx shim codex` or bare `vx shim` as creation commands.

As checked on 2026-10-07, the latest stable release was
[v0.9.34](https://github.com/loonghao/vx/releases/tag/v0.9.34). Its commit
precedes the PR's merge commit `18194435d990267fd9be6e9c0977866758c9cd5a`.
The feature therefore needs a build containing that change or a later release
whose help confirms the interface. RFC 0042's proposed v0.10.0 target is not a
release guarantee. Version output alone cannot distinguish a development build
that retains the previous version number. On an older build, use the existing
explicit vx execution path until a suitable version is available.

## Create only the entry point you need

```sh
vx shim add jq --as jq-vx --dir "<absolute-owned-bin-directory>"
vx shim list --json
```

Replace the directory placeholder with an absolute directory you own. `--dir`
is repeatable and replaces the default directory set. Relative paths are stored
as supplied, so later `sync` or `remove` from another working directory could
address different files. Without `--dir`, vx writes to
both `$VX_HOME/bin` and the directory containing the running vx executable,
deduplicating matching directories. An isolated `VX_HOME` alone does **not**
isolate all default writes: always pass a private `--dir` for a smoke test.
For package commands, choose a dedicated `--dir` outside both `$VX_HOME/shims`
and the directory containing vx: global package shim maintenance scans those
locations too (see the Codex section below).

Before adding a shim, inspect the command that each relevant shell currently
resolves and every prospective output file. On Windows inspect both `<name>`
and `<name>.cmd`, including files outside PATH. Choose a different `--as` name
or directory if either is occupied. Do not add `--force` as a routine retry.

At the PR #1180 implementation, the collision guard checks the command found
by `which`; it does not inspect every destination file or shell alias/function.
The underlying writer replaces existing destination files. Consequently, the
guard is not a guarantee that every unmanaged file is protected. `--force`
allows the detected collision; it neither removes another installation nor
changes PATH priority. Changing directories by re-adding the same name replaces
its registry record without cleaning up the old directory's files. Inspect the
existing entry before relocating it.

`add` records a forwarding specification without checking that the target
runtime or package exists. First execution may install the target and its
dependencies. Treat generation and target execution as separate operations;
do not run an AI agent merely to verify that a wrapper was generated.

## Codex: use the explicit package route

The existing [package execution interface](../cli/implicit-package-execution.md)
supports the official scoped package:

```sh
vx npm:@openai/codex::codex --version
```

Run this only when installing/running that package is intended; it can install
dependencies. It is not required for inspecting shim capability. There is no
dedicated Codex Provider in the PR #1180 merge tree. `vx codex` can resolve an
already registered global package executable, but should not be assumed to be
an on-demand alias for the official scoped package.

After checking the chosen name and directory, create a command shim with an
explicit alias:

```sh
vx shim add "npm:@openai/codex::codex" --as codex-vx --dir "<absolute-owned-bin-directory>"
```

Use `--as codex` instead only when that exact command name is needed and its
existing resolution has been reviewed. `--as` is required for this package
specification: the implementation derives a default name by splitting at the
first `@`, which would produce `npm:` here, not `codex`.

Prefer a distinct alias such as `codex-vx` and a dedicated directory. Global
package installation also creates package shims beside vx; a command shim
named `codex` in that same directory can be replaced by package shim generation.
The two mechanisms have separate registries. `vx shim sync` maintains command
shims; `vx global shim-update` maintains installed package shims.

Package `shim-update` also removes names absent from the package registry in
`$VX_HOME/shims` and beside the running vx executable, without checking the
command-shim marker. Even a distinct alias such as `codex-vx` can be removed
there. On Windows it scans `.cmd` files, so an extensionless companion may
remain and make the result differ between shells. A dedicated `--dir` outside
both locations prevents this overlap; do not use package `shim-update` to repair
command shims.

The command shim stores the supplied specification literally. A package version
may be expressed as `npm:@openai/codex@<version>::codex`, but this is not proof
that an already installed package will switch versions: the merged package
executor looks up the registry by ecosystem and package name without comparing
the requested version. Inspect the installed package with the global-package
commands supported by that build before promising a pinned result.

## Shell resolution and PATH

| Caller | Generated file | Inspect resolution |
| --- | --- | --- |
| Windows PowerShell | `<name>.cmd`, resolved through `PATHEXT` | `Get-Command codex-vx -All`; `where.exe codex-vx` |
| Windows cmd.exe | `<name>.cmd`, resolved through `PATHEXT` | `where codex-vx` |
| Windows Git Bash / MSYS2 | Extensionless `<name>` shell script | `type -a codex-vx`; `command -v codex-vx` |
| Linux / macOS POSIX shells | Extensionless `<name>` shell script | `command -v codex-vx` |

Windows generation produces both files in one operation, not a `.ps1` file.
The batch wrapper forwards `%*` and uses `exit /b %ERRORLEVEL%`; the POSIX
wrapper forwards `"$@"` using `exec`, with forward slashes in the launcher path.
Do not rename one variant to stand in for the other. A native process launcher
that cannot execute batch or shell scripts should invoke the vx executable with
separate arguments, such as `npm:@openai/codex::codex`, rather than assuming a
command shim is a native executable.

PATH membership does not prove which command wins. Check aliases and functions
in PowerShell and Bash, earlier PATH entries, Windows `PATHEXT`, and cmd.exe's
current-directory lookup. In PowerShell use `where.exe`, since `where` is
normally an alias. After changing PATH or moving a wrapper, open a fresh shell;
in Bash, `hash -r` clears cached command locations. An editor or service may
inherit a different PATH and need a restart.

`vx shim path` reports only the **default** directories and their presence on
the current process's PATH. It does not report arbitrary `--dir` entries or
prove command precedence. Use `vx shim list --json` to inspect the stored
`dirs`, `files`, `runtime`, and `launcher` for custom entries.

vx prints PATH hints but does not edit shell profiles, persistent PATH, or the
Windows registry. If the user chooses to expose the reviewed directory for the
current shell, the syntax differs:

```powershell
# PowerShell; substitute the reviewed directory.
$env:PATH = "<absolute-owned-bin-directory>;$env:PATH"
```

```bat
:: cmd.exe; substitute the reviewed directory.
set "PATH=<absolute-owned-bin-directory>;%PATH%"
```

```sh
# Git Bash/POSIX: use the shell's path spelling, such as /c/... in Git Bash.
export PATH="<absolute-owned-bin-directory>:$PATH"
```

These are optional session changes, not installation steps to run automatically.
The Windows hint printed by vx uses PowerShell syntax even when the caller is
Git Bash or cmd.exe.

## Refresh after upgrading or moving vx

Invoke the intended new vx executable explicitly if several installations are
on PATH. In the same `VX_HOME` that owns the entries:

```sh
vx shim list --json
vx shim sync
vx shim list
```

Before `sync`, inspect **all** registered destinations and confirm they still
contain the intended managed wrappers. `sync` rewrites every registered shim
against the running vx executable and recreates missing variants; it has no
per-name selector, dry-run, or ownership check before writing. If a registered
path now holds another program's wrapper or a user edit, resolve that conflict
before syncing. It keeps the recorded directories rather than relocating them
beside the new vx executable. It does not upgrade Codex or other target packages.

The text status `ok` only means every recorded file exists. `incomplete` means
at least one is missing. Neither status validates marker content, launcher
availability, PATH order, or target execution; JSON lists registry entries
without computing a status field.

## Remove safely and diagnose drift

```sh
vx shim list --json
vx shim remove codex-vx
```

Removal requires an entry in `$VX_HOME/config/command-shims.json` and deletes
only recorded files whose contents contain the `vx-shim` marker. Missing,
unreadable, or unmarked files are skipped. The entry is still removed from the
registry after a successful pass, even when every file was skipped. Review the
warnings and remaining files. Do not use `remove --force` to bypass ownership:
it only changes the no-files-removed reporting and never authorizes deletion
of an unmarked file. Removing a command shim does not uninstall its package.

If `list` unexpectedly becomes empty, verify `VX_HOME` and the JSON file before
adding or syncing. The loader treats a missing or corrupt registry as empty;
a later `add` can replace that registry with the newly registered entry.
Preserve the original file for diagnosis. Do not delete or claim ownership of
an unregistered wrapper just because it has a familiar name.

For a local smoke test, use a private `VX_HOME`, an explicit private `--dir`,
and a unique alias. Check generated files, registry data, `list`, `sync`, and
`remove` there. Argument/exit-code tests can use an owned fixture launcher;
they do not need a real Codex session or an external account.

## Implementation references

- [RFC 0042](../rfcs/0042-platform-command-shims.md) and [CLI reference](../cli/shim.md)
- [Command arguments and handlers](https://github.com/loonghao/vx/tree/18194435d990267fd9be6e9c0977866758c9cd5a/crates/vx-cli/src/commands/shim)
- [Platform variants and file writes](https://github.com/loonghao/vx/blob/18194435d990267fd9be6e9c0977866758c9cd5a/crates/vx-runtime/src/shim.rs)
- [Registry model](https://github.com/loonghao/vx/blob/18194435d990267fd9be6e9c0977866758c9cd5a/crates/vx-runtime/src/shim_registry.rs)
- [Package request parser](https://github.com/loonghao/vx/blob/18194435d990267fd9be6e9c0977866758c9cd5a/crates/vx-shim/src/request.rs) and [executor](https://github.com/loonghao/vx/blob/18194435d990267fd9be6e9c0977866758c9cd5a/crates/vx-shim/src/executor.rs)
- [Global package shim maintenance](https://github.com/loonghao/vx/blob/18194435d990267fd9be6e9c0977866758c9cd5a/crates/vx-cli/src/commands/global/handler.rs) and [directory sync](https://github.com/loonghao/vx/blob/18194435d990267fd9be6e9c0977866758c9cd5a/crates/vx-paths/src/shims.rs)
