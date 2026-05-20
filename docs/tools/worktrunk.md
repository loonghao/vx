# worktrunk (wt) - Git Worktree Manager

worktrunk (`wt`) is a Git worktree manager designed for parallel AI agent workflows. It simplifies creating, switching between, and merging worktrees.

## Quick Start

```bash
# Install worktrunk
vx install worktrunk@latest

# Create and switch to a new worktree
vx wt switch feat/add-new-provider

# List all worktrees
vx wt list

# Merge current worktree
vx wt merge

# Remove a worktree
vx wt remove
```

## Installation

worktrunk is available via `vx`:

```bash
vx install worktrunk@latest
```

vx downloads worktrunk from GitHub Releases (built with Rust, single binary).

## Commands

| Command | Description |
|---------|-------------|
| `switch` | Switch to a worktree; create if needed |
| `list` | List worktrees and their status |
| `remove` | Remove worktree; delete branch if merged |
| `merge` | Merge current branch into the target branch |
| `step` | Run individual operations |
| `hook` | Run configured hooks |
| `config` | Manage user & project configs |

## Usage Examples

### Create and Switch to Worktree

```bash
# Create worktree with new branch
vx wt switch feat/add-new-provider

# Create worktree with existing branch
vx wt switch fix/bug-123
```

### List Worktrees

```bash
vx wt list
# Output:
#   main          /path/to/repo                   2b0059cb [main]
# * feat/add      /path/to/repo-feat-add          a1b2c3d4 [feat/add]
#   fix/bug      /path/to/repo-fix-bug           e5f6g7h8 [fix/bug]
```

### Merge Worktree

```bash
# While in a worktree, merge into target branch (default: main)
vx wt merge

# This runs: git checkout main && git merge <current-branch>
```

### Remove Worktree

```bash
# Remove worktree and delete branch (if merged)
vx wt remove feat/add-new-provider
```

## Integration with vx

worktrunk is designed for parallel AI agent workflows. Use it with vx to:

1. **Create isolated workspaces** for each agent
2. **Switch between tasks** without stashing
3. **Merge completed work** back to main branch

```bash
# Agent 1: Add new provider
vx wt switch agent1/add-provider
vx add provider

# Agent 2: Fix bug
vx wt switch agent2/fix-bug
vx fix bug

# Merge when done
vx wt merge  # From agent1/worktree
```

## Multi-Agent Workflow

```bash
# Create worktrees for multiple agents
vx wt switch agent1/task-a
vx wt switch agent2/task-b
vx wt switch agent3/task-c

# List all active worktrees
vx wt list

# Each agent works in their worktree independently
# When done, each agent merges their work
vx wt merge  # Run from each worktree
```

## Configuration

worktrunk can be configured via `wt.toml` (in project root):

```toml
[worktrunk]
default_target = "main"
auto_cleanup = true
```

## Platform Support

| Platform | Supported |
|----------|-----------|
| Windows  | ✓ |
| macOS    | ✓ |
| Linux    | ✓ |

## Source

- **Repository**: [loonghao/worktrunk](https://github.com/loonghao/worktrunk)
- **License**: MIT
- **Language**: Rust

## Related

- [`AGENTS.md`](../AGENTS) - Multi-Agent Development section
- [Git Worktree Documentation](https://git-scm.com/docs/git-worktree)
