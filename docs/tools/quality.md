# Code Quality Tools

vx supports tools for maintaining code quality and consistency.

## pre-commit

A framework for managing and maintaining multi-language pre-commit hooks.

```bash
vx install pre-commit latest

vx pre-commit --version
vx pre-commit install            # Install hooks
vx pre-commit run --all-files    # Run on all files
vx pre-commit autoupdate         # Update hook versions
vx pre-commit uninstall          # Remove hooks
```

**Key Features:**

- Multi-language support
- Automatic hook management
- CI/CD integration
- Extensive hook ecosystem

**Example .pre-commit-config.yaml:**

```yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-json

  - repo: https://github.com/psf/black
    rev: 24.1.0
    hooks:
      - id: black

  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.14
    hooks:
      - id: ruff
        args: [--fix]
```

**Project Configuration:**

```toml
[tools]
pre-commit = "latest"

[scripts]
lint = "pre-commit run --all-files"
lint-install = "pre-commit install"
lint-update = "pre-commit autoupdate"
```

## Integration with CI/CD

```yaml
# GitHub Actions example
- name: Setup vx
  uses: loonghao/vx@v0.5

- name: Install pre-commit
  run: vx install pre-commit latest

- name: Run pre-commit
  run: vx pre-commit run --all-files
```

## Best Practices

1. **Commit Hook Installation**: Always run `pre-commit install` after cloning
2. **CI Integration**: Run pre-commit in CI to catch issues
3. **Version Pinning**: Pin hook versions for reproducibility
4. **Staged Files Only**: Default behavior runs only on staged files

```toml
[tools]
pre-commit = "3.6"  # Pin version

[scripts]
# Quick lint (staged files only)
lint = "pre-commit run"

# Full lint (all files)
lint-all = "pre-commit run --all-files"

# Setup for new developers
setup = "pre-commit install && pre-commit install --hook-type commit-msg"
```
