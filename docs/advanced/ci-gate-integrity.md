# CI Gate Integrity

How vx keeps a `[skip ci]` marker from silently switching off CI on `main`.

## The failure mode

GitHub suppresses **every** workflow run of a push event when the head commit
message contains one of these markers:

```text
[skip ci]   [ci skip]   [no ci]   [skip actions]   [actions skip]
```

A squash merge builds its commit message from two parts: the pull request title
becomes the subject, and GitHub appends a generated bullet list of **every
commit on the branch**. A marker on a single branch commit therefore survives
into the merge commit:

```text
fix(ci): stop skipping every test job in change detection

* fix(ci): stop skipping every test job in change detection
* chore: regenerate workspace-hack (cargo-hakari) [skip ci]   ← inherits the marker
* fix(ci): harden change detection base resolution
```

The merge commit then lands on `main` with a clean-looking subject and **zero
workflow runs**. Nothing turns red, because nothing ran. Whoever performed the
merge cannot see it either: the marker hides in the generated body, and the
Actions tab simply shows no run for that commit.

This happened on 2026-09-19: the squash merge of PR #1097 (`4562a26a`) reached
`main` with a marker in the generated bullet list and produced zero push runs.
The code was fine — it was verified green on a later commit — which is exactly
what makes this failure mode dangerous. It is invisible unless someone queries
runs by head SHA.

## Quoting a marker still triggers it

GitHub matches the marker as plain text over the whole commit message. Backticks,
quotation marks and indentation make no difference, so a commit message that
*discusses* the marker suppresses CI just as effectively as one that intends to:

```text
fix: explain why the housekeeping commit used a skip marker

The commit read: chore: regenerate workspace-hack (cargo-hakari) [skip ci]
```

That commit runs no workflow either. **Describe the marker in words in commit
messages** ("a skip marker", "the CI skip marker") and never paste the bracketed
form. The guard rejects any commit message containing it, including this one.

File contents are unaffected: only commit messages are scanned.

## The three layers

| Layer | Workflow | What it does |
| --- | --- | --- |
| Block | [CI Skip Marker Guard](https://github.com/loonghao/vx/actions/workflows/pr-ci-marker-guard.yml) | Fails the pull request when the title or any branch commit carries a marker. It runs on `pull_request_target`, which GitHub evaluates on the default branch, so it becomes active once this change reaches `main` |
| Prevent | `ci.yml` | No longer writes the marker in the commit CI itself pushes to pull request branches |
| Detect | [CI Gate Sentinel](https://github.com/loonghao/vx/actions/workflows/ci-gate-sentinel.yml) | Hourly scan of `main` for commits with no push-event run |

The first two layers stop the marker before a merge. The third catches what they
cannot see: pull requests opened before the guard existed, direct pushes to
`main`, and markers arriving by any other route.

### Layer 1: the pull request guard

`CI Skip Marker Guard` runs on `pull_request_target` and publishes a
`CI Skip Marker` status on the pull request head. It reads commit messages
through the API and never executes pull request code, so it stays safe for
forks. The check fails when a marker appears in:

- the pull request title (which becomes the squash subject), or
- the message of any commit on the branch.

Run it locally before merging:

```bash
just check-pr-ci-markers 1097      # or: bash scripts/check_pr_ci_markers.sh 1097
```

### Layer 2: do not write the marker

`ci.yml` regenerates `workspace-hack` and pushes the result back to the pull
request branch. That commit used to be marked `[skip ci]` to avoid a commit
loop. The marker was never needed: a push made with the repository
`GITHUB_TOKEN` does not start a new workflow run, which is GitHub's own
recursion guard. The marker only accomplished the bypass.

Any automation that writes a commit message in this repository must keep it
marker-free. The pull request guard fails the pull request if one reappears.

### Layer 3: the sentinel

`CI Gate Sentinel` runs hourly and inspects the last commits on `main`. For each
commit it counts push-event workflow runs and classifies the ones that have
none:

| Kind | Severity | Meaning |
| --- | --- | --- |
| `bypass` | error | Marker sits in the generated bullet list of a squash merge — the invisible case |
| `intentional-skip` | warning | Marker is in the author's own text, so the push was skipped on purpose |
| `missing-runs` | warning | No marker and no run; typically an intermediate commit of a multi-commit push |
| `marker-ineffective` | warning | Marker present but runs exist (for example a re-triggered run) |
| `bot-commit` | info | Authored by a bot; `GITHUB_TOKEN` pushes start no run, which is expected |
| `too-recent` | info | Younger than the grace period; runs may not exist yet |

Only `bypass` fails the run. A failing scheduled run notifies the workflow
author, and the workflow opens a `ci-gate` issue so the bypass stays visible
after the run log expires.

Run it locally:

```bash
just check-main-ci 25              # or: python3 scripts/check_main_ci_runs.py --branch main
```

## Merging a pull request

Squash with an explicit subject and body instead of accepting the generated
list. A body that does not enumerate branch commits cannot carry a marker:

```bash
gh pr merge 1097 --squash \
  --subject "fix(ci): harden change detection base resolution" \
  --body "Diff against the merge base instead of the base tip."
```

After the merge, confirm the gate actually ran. Zero push runs means it did
not:

```bash
gh api "repos/loonghao/vx/actions/runs?head_sha=<sha>&event=push" --jq .total_count
```

If the count is `0`, re-trigger the checks for that commit — an empty commit on
`main`, or a re-run of the CI workflow for the merge commit.

## Recommended follow-up

`main` is currently **not** a protected branch, so no check — including the pull
request guard — can block a merge on its own. Enabling branch protection with
`CI Skip Marker` and `CI Success` as required status checks turns layer 1 from a
visible warning into a hard block. That is a repository setting rather than a
code change, so it needs an owner decision.

## Related

- [Contributing](./contributing.md) — general contribution workflow
- [Release Process](./release-process.md) — how release commits reach `main`
