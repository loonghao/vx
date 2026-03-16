# install

Install one or more runtimes.

## Synopsis

```bash
vx install <runtime>[@version] [<runtime>[@version] ...] [--force]
```

## Description

`vx install` installs vx-managed runtimes explicitly.

- If no version is provided, vx resolves `latest`.
- You can install multiple runtimes in one command.
- Bundled runtimes are installed via their parent runtime automatically.

Examples:

- `cargo` and `rustc` are bundled with `rustup`.
- Installing `cargo` may redirect to installing `rustup`.

## Options

| Option | Description |
|---|---|
| `-f`, `--force` | Reinstall even when already installed |

## Usage Examples

```bash
# Install latest versions
vx install node uv go

# Install specific versions
vx install node@22 go@1.22

# Rust ecosystem (recommended)
vx install rustup
vx cargo --version
vx rustc --version

# Force reinstall
vx install node@22 --force
```

## Version Notes

- Runtime versions and toolchain versions can differ.
- For Rust, prefer configuring/installing `rustup`, then use `vx cargo` / `vx rustc`.

## Related

- [`overview`](./overview) - CLI overview
- [`list`](./list) - List installed/available runtimes
- [`global`](./global) - Global package management
