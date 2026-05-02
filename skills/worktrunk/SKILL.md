---
name: worktrunk
description: "Git worktree manager for parallel AI agent workflows. Use when the user needs to manage multiple working directories for parallel development, create feature branches with isolated environments, or run multiple AI agents concurrently. Covers `vx worktrunk` and `vx wt` commands."
---

# Worktrunk — Git Worktree Manager for AI Agents

> **One-sentence summary**: `vx worktrunk` (alias `vx wt`) manages Git worktrees so you can run multiple AI agents in parallel without conflicts.

worktrunk is a Rust CLI tool designed for **parallel AI agent workflows**. It makes Git worktrees as easy to use as branches, with quality-of-life features like hooks, LLM commit messages, and per-worktree dev servers.

## Why AI Agents Need This

When running multiple AI agents (Claude Code, Codex, etc.) on the same repo, they conflict:
- Agent A modifies `src/main.rs`
- Agent B modifies `src/main.rs` → **conflict**

**Solution**: Give each agent its own worktree (isolated working directory).

```
Main repo:     ~/projects/my-app/          (main branch)
Agent 1:      ~/projects/my-app.wt/feat-auth/  (feat/auth branch)
Agent 2:      ~/projects/my-app.wt/feat-pay/   (feat/pay branch)
```

## Installation (via vx)

```bash
vx install worktrunk
vx worktrunk --version    # verify
```

## Core Commands

### `wt switch` — Navigate Worktrees

```bash
# Create new worktree + switch to it
vx wt switch -c feat/auth

# Switch to existing worktree
vx wt switch feat/auth

# Switch back to main repo
vx wt switch -

# Create AND launch an AI agent in one command
vx wt switch -c -x claude feat/auth
```

### `wt list` — Show All Worktrees

```bash
vx wt list            # compact list
vx wt list --full     # with CI status, AI-generated summaries, diff preview
```

### `wt merge` — Clean Merge Workflow

```bash
vx wt merge          # squash merge + delete worktree + delete branch
vx wt merge --rebase  # rebase merge
vx wt merge --keep    # merge but keep worktree
```

### `wt remove` — Clean Up

```bash
vx wt remove feat/auth          # remove worktree only
vx wt remove --with-branch feat/auth  # remove worktree + branch
```

## Advanced Features

### PR Checkout

```bash
vx wt switch pr:123    # checkout PR #123 into a worktree
```

### Hooks (Automate Setup)

Configure in `.worktrunk/hooks.toml`:

```toml
[post_create]
command = "npm install"     # auto-install deps on create

[pre_merge]
command = "vx run test"      # run tests before merge
```

### Share Build Caches Between Worktrees

```bash
# Copy node_modules/ to new worktree (skip cold install)
vx wt switch -c --copy-cache feat/auth
```

### Per-Worktree Dev Server (Unique Port)

worktrunk can assign each worktree a unique port (via `hash_port` template filter):

```bash
# In package.json:
"scripts": {
  "dev": "next dev -p {{ hash_port 3000 }}"
}
# Main repo  → port 3000
# feat/auth → port 3217 (deterministic hash)
```

## Typical AI Agent Workflow

```bash
# 1. Main repo: assign tasks to agents
cd ~/projects/my-app
echo "Add JWT auth" > .AITASK

# 2. Create isolated worktree for agent 1
vx wt switch -c -x claude feat/auth

# (inside worktree, agent runs autonomously)
# Agent modifies files, commits, etc.

# 3. Meanwhile, create another worktree for agent 2
vx wt switch -c -x claude feat/pay

# 4. When agents finish, merge both
vx wt merge feat/auth
vx wt merge feat/pay
```

## vx Integration

worktrunk is installed and managed by vx:

```bash
# Install
vx install worktrunk

# Run (vx auto-installs if missing)
vx worktrunk --version
vx wt switch -c feat/auth

# Update
vx install worktrunk@latest
```

## Command Reference

| Command | Description |
|----------|-------------|
| `vx wt switch [branch]` | Switch to worktree |
| `vx wt switch -c [branch]` | Create worktree + switch |
| `vx wt switch -c -x <cmd> [branch]` | Create + run command (e.g., launch AI agent) |
| `vx wt switch pr:<number>` | Checkout PR into worktree |
| `vx wt list` | List worktrees |
| `vx wt list --full` | Detailed list with CI/LLM summary |
| `vx wt merge` | Squash merge + cleanup |
| `vx wt remove [branch]` | Remove worktree |
| `vx wt remove --with-branch [branch]` | Remove worktree + branch |

## Tips for AI Agents

1. **Always use `vx wt switch -c`** to create isolated environments before making changes
2. **Use `-x claude`** to automatically launch the AI agent in the new worktree
3. **Use `vx wt merge`** instead of manual `git merge` — it handles worktree cleanup
4. **Configure hooks** in `.worktrunk/hooks.toml` to auto-install dependencies on create
5. **Use `--copy-cache`** to share `node_modules/` / `target/` between worktrees
