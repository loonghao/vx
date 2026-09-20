# Lockfile drift on dependency bump pull requests

## Symptom

Two failure modes, both produced by the same instability:

1. **Unrelated edges move backwards** — several crates move their `windows-sys` edge from `0.61.2` to `0.52.0` alongside the intended bump.
2. **The bump is not delivered at all** — the branch advertises a bump, but the merge result contains the old version. In the extreme the two commits cancel each other and the pull request shows a net change of zero files.

A Renovate crate bump adds unrelated changes to `Cargo.lock`. Alongside the
intended bump, several crates move their `windows-sys` edge from `0.61.2` back
to `0.52.0`:

The crates affected by the first mode:

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

Neither mode yields a broken lockfile: every version involved is declared,
every reference resolves, and the builds and tests are green. The problem is
that the pull request does not do what it says — it changes something it does
not advertise, or it advertises a change it undid.

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

## A bump that never lands

The same instability can take the bump itself. `cargo hakari generate` runs in
`Code Quality (via vx)`, its result is committed back onto the pull request
branch, and the re-resolution it performs has been seen removing the very
package the branch was opened to bump:

```text
532c7487  fix(deps): update rust crate tower-http to 0.7
          Cargo.lock +28/-2   -> tower-http 0.6.11 and 0.7.1 both present
527ca3a6  chore: regenerate workspace-hack (cargo-hakari)
          Cargo.lock +10/-34  -> the 0.7.1 package entry is gone
```

The second commit reverted the first, so the branch head and `main` carried
the same `tower-http 0.6.11`, the pull request showed a net change of zero
files, and merging it would have delivered nothing. Note that the drift check
alone does not catch this: nothing moved *backwards*, the package simply
disappeared.

Two independent automations writing to one branch is what makes this hard to
see from the diff, because the diff is empty.

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

The title is compared against the merged lockfile as well, so a branch that
claims a bump and does not deliver it is reported:

```bash
python3 scripts/check_lockfile_drift.py \
  --base origin/main --head pr/1129 \
  --title "fix(deps): update rust crate tower-http to 0.7"
```

```text
No dependency moves backwards. ✅

1 claimed bump(s) were not delivered: ⚠️

  - tower-http 0.7 is not delivered (lockfile has: 0.6.11)
```

Only titles that claim a bump are checked, so a pull request that touches no
dependencies is never reported. A missed bump **warns** rather than fails:
blocking a merge over a title the parser misread is worse than merging a pull
request that did nothing. Pass `--strict` to make it fail instead, after
checking the report by hand.

Exit codes: `0` nothing moved backwards and every claimed bump was delivered,
`1` at least one dependency moved backwards (or, with `--strict`, a claimed
bump was not delivered), `2` the comparison could not be made.

The comparison uses `git merge-tree --write-tree`, so a branch that lags
behind the base is not blamed for the base's own progress. `Lockfile Drift`
runs it on every pull request that touches a manifest or the lockfile.

Because the resolution oscillates, regenerating the lockfile is worth
retrying: a fresh `cargo update` lands on a resolution without the unrelated
moves often enough to be worth one attempt before investigating further.
Re-run the check after each attempt; it is the only way to tell.

For a branch whose bump was swallowed, a rebase is often not enough — the two
commits are already on the branch and a rebase replays both. Recreating the
branch (`cargo update` again from the current base) is what produces a branch
whose head actually carries the bump.
