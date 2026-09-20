# Lockfile drift on dependency bump pull requests

## Symptom

A Renovate crate bump adds unrelated changes to `Cargo.lock`. Alongside the
intended bump, several crates move their `windows-sys` edge from `0.61.2` back
to `0.52.0`:

| crate |
| --- |
| `colored` |
| `errno` |
| `is-terminal` |
| `rustix` |
| `rustls-platform-verifier` |
| `tempfile` |
| `winapi-util` |

The same shape appears for `getrandom`, where `tempfile` moves from `0.4.3`
back to `0.3.4`.

The result is never a broken lockfile: every version involved is declared,
every reference resolves, and the builds and tests are green. The problem is
that the pull request changes something it does not advertise, in the
backwards direction.

## Cause

`workspace-hack` (cargo-hakari) pins **both** `windows-sys 0.52` and
`windows-sys 0.61`, because different third-party crates genuinely require
each of them — `fd-lock`, `rustyline`, `self-replace` and `filetime_creation`
require `^0.52`, while others use features only `0.61` provides. Both entries
must therefore exist in the lockfile, and they cannot be merged into one.

Crates whose own requirement accepts either candidate are then *free edges*:
the resolver may point them at either version. Which one it picks is not
stable. Re-running an identical command on an unchanged tree flips them:

```console
$ cargo update -p tokio --precise 1.53.1   # pass 1
$ cargo tree -p colored -e normal --depth 1
└── windows-sys v0.52.0
$ cargo update -p tokio --precise 1.53.1   # pass 2, same command
$ cargo tree -p colored -e normal --depth 1
└── windows-sys v0.61.2
```

Two consequences follow, and both were verified:

- **Rebasing does not fix it.** The drift is present in branches that are
  already rebased onto the current `main`, so it is not caused by the branch
  lagging behind.
- **No lockfile state prevents it.** Since the resolution oscillates rather
  than converging, there is no committed state that a future bump is
  guaranteed to preserve.

The same holds for the *targeted* form of the update. A bare `cargo update`
is not required to trigger it; `cargo update -p <crate> --precise <version>`
reproduces it, and so does a no-op update that names the version already in
the lockfile.

## What to do

There is no configuration that makes the resolver stable, so the check is on
the outcome. `scripts/check_lockfile_drift.py` compares the `Cargo.lock` of
the real merge result against the base branch and reports any edge that moved
backwards:

```bash
# Current branch against the default base (origin/main).
python3 scripts/check_lockfile_drift.py

# Any two refs, for example a pull request before approving it.
python3 scripts/check_lockfile_drift.py --base origin/main --head pr/1121

# Machine readable, for a review note.
python3 scripts/check_lockfile_drift.py --base origin/main --head pr/1121 --format json
```

Exit codes: `0` nothing moved backwards, `1` at least one dependency moved
backwards, `2` the comparison could not be made.

The comparison uses `git merge-tree --write-tree`, so a branch that lags
behind the base is not blamed for the base's own progress. `Lockfile Drift`
runs it on every pull request that touches a manifest or the lockfile.

Because the resolution oscillates, regenerating the lockfile is worth
retrying: a fresh `cargo update` lands on a resolution without the unrelated
moves often enough to be worth one attempt before investigating further.
Re-run the check after each attempt; it is the only way to tell.
