# Tool Pin Baseline

> One checkable answer to "how do I write `[tools]` in `vx.toml`?".
> Adopt this page if you maintain more than one repository with vx — the point is not
> that the rules are clever, but that every repository answers the question the same way.

## Why a baseline

Two failure modes drive every rule below.

1. **Two pins that can silently disagree.** Declaring `rust = "1.95"` in `vx.toml` while
   the repository also ships `rust-toolchain.toml` gives you two sources of truth. Only
   one of them wins and nothing tells you which.
2. **Pins that float.** `python = "latest"` records no version at all — the toolchain that
   ran yesterday's CI is not written down anywhere.

## Version syntax

| Syntax              | Resolves to                                            |
| ------------------- | ------------------------------------------------------ |
| `major.minor`       | newest patch in that minor line (`3.12` → `3.12.x`)     |
| `major.minor.patch` | that exact version                                     |
| `latest`            | newest stable release                                  |

See [Version Management](./version-management.md) for the full syntax.

## The three tool classes

| Class          | Definition                                     | Allowed value                                                | Example                 |
| -------------- | ---------------------------------------------- | ------------------------------------------------------------ | ----------------------- |
| **runtime**    | a language runtime your code runs on           | `major.minor`; exact patch only with a reason comment          | `python = "3.12"`       |
| **build tool** | produces no shipped artifact                   | `latest`                                                      | `uv = "latest"`         |
| **artifact**   | its version changes a published artifact       | exact, or `major.minor` with a reason comment                 | `maturin = "1.9.6"`     |

Typical members: **runtime** — `python`, `node`, `rust`. **build tool** — `uv`, `just`,
`cmake`, `prek`, `actionlint`, `sccache`. **artifact** — `maturin`, `cargo-llvm-cov`,
`cargo-nextest`, `msvc`.

## Rules

- **R1 — one source of truth per tool.** When the ecosystem already owns the pin
  (`rust-toolchain.toml`), `vx.toml` must not repeat it.
- **R2 — no floating runtimes.** `latest`, `stable`, `nightly` and `*` are banned in the
  runtime class.
- **R3 — exact pins carry a reason.** A `major.minor.patch` or artifact pin gets a
  trailing comment naming the regression or constraint.
- **R4 — no default-restating config.** If a setting already equals vx's default, delete it.
- **R5 — exceptions are inline.** Any deviation from this page gets a one-line comment in
  `vx.toml`, not a wiki page.

## Rust

Rust is the only runtime with a competing native pin file, so it gets explicit rules. vx
decides ownership in this order:

1. `rust-toolchain.toml` / `rust-toolchain`, walking up from the working directory →
   **rustup owns it**. vx does not install, does not prepend its store to `PATH`, and never
   changes the rustup default.
2. `RUSTUP_TOOLCHAIN` in the environment → **rustup owns it**.
3. `rust = "rustup-managed"` in `vx.toml` → **rustup owns it**. This is the explicit opt-out
   for repositories that deliberately keep Rust out of vx.
4. Otherwise → **vx owns it** and installs into its own store. Even then the toolchain is
   injected into the subprocess environment with `RUSTUP_TOOLCHAIN`; the user's global
   rustup default is never rewritten.

| Your repository has                             | Write in `vx.toml`                                    |
| ----------------------------------------------- | ----------------------------------------------------- |
| `rust-toolchain.toml`                           | nothing — that file is the pin                        |
| rustup available, no toolchain file             | `rust = "rustup-managed"`                             |
| no rustup at all (bare container, slim CI image) | `rust = "1.95"` — a `major.minor` pin, never a channel |

The ownership order and the `rustup-managed` sentinel ship with the rust-toolchain
ownership change. On older vx, omit the `rust` row and rely on `rust-toolchain.toml` alone.

## vx.lock — commit it

- `vx.lock` **overrides** `vx.toml`: locked versions win during resolution, so the lock is
  what actually makes CI reproducible.
- **Commit `vx.lock`.** Regenerate it with `vx lock` whenever `[tools]` changes — a stale
  lock keeps pinning the old version and makes your `vx.toml` edit look like it did nothing.
- Guard it in CI:

  ```bash
  test -f vx.lock || { echo "vx.lock missing; run 'vx lock'"; exit 1; }
  vx lock --check
  ```

  `vx lock --check` fails when a tool is missing from the lock, when the locked
  `resolved_from` no longer matches `vx.toml`, or when the lock still carries a tool that
  `vx.toml` dropped. It exits **0** when the lock file is absent, which is why the
  existence check has to come first.
- One committed lock serves every platform. Locked version strings are platform
  independent, `platform_urls` carries per-target download URLs, and the `platform`
  metadata field only records where the lock was generated.

## `[settings]` — delete the defaults

| Key                 | vx behaviour                          | Verdict                                        |
| ------------------- | ------------------------------------- | ---------------------------------------------- |
| `auto_install`      | defaults to `true`                    | delete the row unless you set it to `false`    |
| `cache_duration`    | parsed, but not consumed by resolution | delete — inert                                 |
| `parallel_install`  | parsed, but not consumed by resolution | delete — inert                                 |

`cache_duration` and `parallel_install` are still parsed, migrated and written out by
`vx init`, but nothing in the resolver reads them, so they only add diff noise. Keep
`[settings]` for real overrides — `auto_install = false` on an air-gapped runner, for
example — with a comment saying why.

## Exceptions

Any row that leaves the baseline carries a trailing comment:

```toml
[tools]
python = "3.14.4"   # rez 3.x needs the 3.14 importlib.metadata fix; see issue #1234
rust   = "1.90.0"   # MSRV of the bundled CEF wrapper; bump with the wrapper
```

## Machine-checkable gate

```bash
# R1: no duplicate rust pin next to a toolchain file
if test -f rust-toolchain.toml && grep -qE '^[[:space:]]*rust[[:space:]]*=' vx.toml; then
  echo "rust is pinned twice (rust-toolchain.toml and vx.toml)"; exit 1
fi
# R2: no floating runtimes
if grep -qE '^[[:space:]]*(python|node|rust)[[:space:]]*=[[:space:]]*"(latest|stable|nightly|\*)"' vx.toml; then
  echo "runtime pinned to a floating channel"; exit 1
fi
# R4: no default-restating settings
if grep -qE '^[[:space:]]*(auto_install[[:space:]]*=[[:space:]]*true|cache_duration|parallel_install)' vx.toml; then
  echo "[settings] restates vx defaults"; exit 1
fi
# vx.lock present and consistent with vx.toml
test -f vx.lock || { echo "vx.lock missing; run 'vx lock'"; exit 1; }
vx lock --check
```

R3 and R5 are review-time rules: a `major.minor.patch` or artifact pin without a trailing
comment fails review.

## Worked example — adopting this across 12 repositories

Snapshot taken 2026-09-28 from the `dcc-mcp` organisation and the `loonghao` personal
repos. It is a snapshot, not part of the contract — the rules above are the contract.

| Repository                  | rust                        | python           | node          | uv            | `[settings]` | `vx.lock` |
| --------------------------- | --------------------------- | ---------------- | ------------- | ------------- | ------------ | --------- |
| `dcc-mcp/dcc-mcp-core`      | `1.95` + toolchain file     | `3.12`           | `22`          | `latest`      | yes          | yes       |
| `dcc-mcp/dcc-mcp-photoshop` | —                           | `3.12`           | —             | `latest`      | yes          | no        |
| `dcc-mcp/dcc-mcp-unreal`    | —                           | `3.12`           | —             | —             | no           | no        |
| `dcc-mcp/dcc-cua`           | —                           | —                | `22`          | —             | no           | no        |
| `loonghao/fpt-cli`          | `stable`                    | `3.12`           | `22`          | `latest`      | yes          | yes       |
| `loonghao/shotgrid-mcp-*`   | —                           | `latest`         | —             | `0.7.12`      | no           | no        |
| `loonghao/auroraview`       | `1.90.0`                    | `3.11`           | —             | `latest`      | yes          | yes       |
| `loonghao/vx`               | omitted                     | `3.11`           | `latest`      | `latest`      | yes          | yes       |
| `loonghao/msvc-kit`         | `1.93.1`                    | —                | `22`          | —             | yes          | yes       |
| `loonghao/rez-next`         | omitted                     | `3.14.4`         | —             | `latest`      | yes          | no        |
| `loonghao/transx`           | —                           | `3.12`           | —             | `latest`      | yes          | yes       |
| `loonghao/rez-lsp-server`   | —                           | —                | `22`          | —             | no           | no        |

What the baseline changes in that snapshot:

- **rust** — `dcc-mcp-core` drops `rust = "1.95"` (a toolchain file already pins it);
  `fpt-cli` replaces `stable`; `auroraview` and `msvc-kit` move their pins into a
  `rust-toolchain.toml` and drop the `vx.toml` row.
- **python** — `shotgrid-mcp-server` moves off `latest`; `3.14.4` in `rez-next` stays and
  gains the reason comment R3 requires; `3.11` in `auroraview` and `vx` either moves to
  `3.12` or gains a comment.
- **node** — `vx` moves from `latest` to `22`, matching every other repository.
- **uv** — `0.7.12` in `shotgrid-mcp-server` becomes `latest` unless a recorded regression
  says otherwise.
- **`[settings]`** — removed from the eight repositories that only restate defaults.
- **`vx.lock`** — generated and committed in the six repositories that lack one.

## Related

- [Configuration](./configuration.md) — every `vx.toml` section
- [Version Management](./version-management.md) — the full version syntax
- [Best Practices](./best-practices.md) — pinning guidance in context
