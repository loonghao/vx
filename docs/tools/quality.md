# Code Quality Tools

vx supports a wide range of tools for maintaining code quality, linting, formatting, and security auditing.

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
    rev: v5.0.0
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
    rev: v0.8.0
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

## ruff

An extremely fast Python linter and code formatter, written in Rust.

```bash
vx install uv          # ruff is bundled with uv

vx ruff check .                     # Lint Python files
vx ruff check --fix .              # Auto-fix issues
vx ruff format .                   # Format Python files
vx ruff check --select F401 .     # Check unused imports
```

**Key Features:**

- 10-100x faster than existing tools (flake8, isort, pyupgrade, etc.)
- Built-in formatter (replaces Black)
- Comprehensive rule set (700+ rules)
- Zero configuration needed for basic use

**Project Configuration:**

```toml
[tools]
uv = "latest"

[scripts]
lint = "ruff check ."
format = "ruff format ."
lint-fix = "ruff check --fix . && ruff format --check ."
```

**Configuration (pyproject.toml):**

```toml
[tool.ruff]
line-length = 100
target-version = "py38"

[tool.ruff.lint]
select = ["E", "F", "I", "N", "W"]
ignore = ["E501"]
```

## ripgrep (rg)

A fast search tool that recursively searches directories for a regex pattern.

```bash
vx install ripgrep

vx rg "function" .               # Search for "function"
vx rg -i "TODO" .               # Case-insensitive search
vx rg --type py "import" .       # Search only Python files
vx rg -A 3 -B 3 "error" log.txt  # Show context lines
```

**Key Features:**

- Much faster than grep
- Respects .gitignore by default
- Automatic regex optimization
- Supports many file types

## fd

A simple, fast, and user-friendly alternative to `find`.

```bash
vx install fd

vx fd "*.py" .                  # Find all Python files
vx fd --type f "test" .          # Find files (not directories)
vx fd --extension rs "impl" .    # Find Rust files containing "impl"
vx fd --hidden --no-ignore "conf"  # Include hidden files
```

**Key Features:**

- Intuitive syntax
- Much faster than `find`
- Colorized output
- Respects .gitignore by default

## bat

A cat clone with syntax highlighting and Git integration.

```bash
vx install bat

vx bat file.py                    # View file with syntax highlighting
vx bat --style changes file.py    # Show Git changes in the file
vx bat --language py --plain     # Force language for syntax
vx cat file.py | vx bat          # Pipe into bat (automatic paging)
```

**Key Features:**

- Syntax highlighting for 150+ languages
- Git integration (shows modifications)
- Automatic paging
- File concatenation with style

**Configuration:**

```bash
# Set as default pager
export PAGER="bat --paging=always"

# Configure in ~/.config/bat/config
```

## biome

A fast formatter and linter for JavaScript, TypeScript, JSX, and JSON.

```bash
vx install biome

vx biome check .                   # Lint and format check
vx biome check --apply .            # Auto-fix issues
vx biome format --write .           # Format files
```

**Key Features:**

- Extremely fast (Rust-based)
- Replaces Prettier + ESLint for basic use cases
- Supports JavaScript, TypeScript, JSX, TSX, JSON, JSONC
- Integrated formatter and linter

**Project Configuration (biome.json):**

```json
{
  "formatter": {
    "enabled": true,
    "indentStyle": "space",
    "indentSize": 2
  },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true
    }
  }
}
```

## oxlint

A confident, Rust-based JavaScript linter.

```bash
vx install oxlint

vx oxlint .                        # Lint JavaScript/TypeScript files
vx oxlint --deny no-console .      # Enable specific rules
vx oxlint --config .oxlintrc.json .  # Use config file
```

**Key Features:**

- No dependencies needed
- Extremely fast (Rust-based)
- 100+ built-in rules
- Replaces ESLint for many use cases

## golangci-lint

Fast Go linters runner.

```bash
vx install golangci-lint

vx golangci-lint run               # Run all linters
vx golangci-lint run --fix         # Auto-fix issues
vx golangci-lint run --new-from-rev=main  # Lint only new changes
```

**Key Features:**

- Runs 10-50 linters in parallel
- Caches results
- YAML configuration
- Integrates with major IDEs

**Configuration (.golangci.yml):**

```yaml
linters:
  enable:
    - gofmt
    - golint
    - govet
    - staticcheck
    - errcheck
```

## cargo-audit

Audit Cargo dependencies for security vulnerabilities.

```bash
vx install cargo-audit

vx cargo-audit                  # Check for vulnerabilities
vx cargo-audit --fix            # Auto-fix (update dependencies)
vx cargo-audit db --fetch       # Fetch vulnerability database
```

**Key Features:**

- Uses RustSec Advisory Database
- Checks both direct and transitive dependencies
- Can be integrated into CI/CD
- Supports ignoring specific advisories

**Project Configuration:**

```toml
[tools]
cargo-audit = "latest"

[scripts]
security-audit = "cargo-audit"
```

## cargo-deny

Cargo dependency policy checker.

```bash
vx install cargo-deny

vx cargo-deny check              # Run all checks
vx cargo-deny check licenses     # Check license compliance
vx cargo-deny check advisories   # Check security advisories
vx cargo-deny check bans        # Check for banned dependencies
```

**Key Features:**

- License compliance checking
- Security advisory detection
- Dependency duplicate detection
- Configurable deny/bans list

**Configuration (deny.toml):**

```toml
[graph-roots]
all-features = true

[advisories]
vulnerability = "deny"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
```

## cargo-nextest

Next-generation test runner for Rust.

```bash
vx install cargo-nextest

vx cargo-nextest run             # Run all tests
vx cargo-nextest run --release  # Run tests in release mode
vx cargo-nextest list            # List all tests
```

**Key Features:**

- Faster than `cargo test` (parallel execution)
- Better output (structured, colorized)
- Test partitioning for CI parallelism
- Handles test failures gracefully

**Project Configuration:**

```toml
[tools]
cargo-nextest = "latest"

[scripts]
test = "cargo-nextest run"
test-release = "cargo-nextest run --release"
```

## hadolint

A smarter Dockerfile linter.

```bash
vx install hadolint

vx hadolint Dockerfile           # Lint a Dockerfile
vx hadolint --ignore DL3008 Dockerfile  # Ignore specific rules
```

**Key Features:**

- Validates Dockerfile syntax
- Checks for best practices
- Integrates with major CI/CD systems
- Supports inline suppression

## actionlint

Static checker for GitHub Actions workflow files.

```bash
vx install actionlint

vx actionlint                       # Lint all workflow files
vx actionlint .github/workflows/     # Lint specific directory
vx actionlint --color               # Enable colored output
```

**Key Features:**

- Checks for syntax errors and semantic problems
- Validates action references
- Checks shell script safety
- Fast and comprehensive

**Project Configuration:**

```toml
[tools]
actionlint = "latest"

[scripts]
lint-actions = "actionlint"
```

## Integration with CI/CD

### GitHub Actions

```yaml
name: Lint

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup vx
        uses: loonghao/vx@main

      - name: Install tools
        run: |
          vx install pre-commit
          vx install ruff
          vx install ripgrep

      - name: Run pre-commit
        run: vx pre-commit run --all-files

      - name: Run ruff
        run: vx ruff check .

      - name: Run actionlint
        run: |
          vx install actionlint
          vx actionlint
```

### Pre-commit Configuration

Example `.pre-commit-config.yaml` with multiple tools:

```yaml
repos:
  # Python
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.8.0
    hooks:
      - id: ruff
      - id: ruff-format

  # Rust
  - repo: local
    hooks:
      - id: cargo-audit
        name: cargo-audit
        entry: vx cargo-audit
        language: system
        pass_filenames: false

      - id: cargo-deny
        name: cargo-deny
        entry: vx cargo-deny check
        language: system
        pass_filenames: false

  # GitHub Actions
  - repo: https://github.com/rhysd/actionlint
    rev: v1.7.1
    hooks:
      - id: actionlint

  # Docker
  - repo: local
    hooks:
      - id: hadolint
        name: hadolint
        entry: vx hadolint
        language: system
        files: Dockerfile
```

## Best Practices

1. **Pin Versions**: Pin tool versions in `vx.toml` for reproducibility
2. **CI Integration**: Run linters in CI to catch issues early
3. **Pre-commit Hooks**: Install pre-commit hooks for local development
4. **Combine Tools**: Use multiple tools together (e.g., ruff + mypy for Python)
5. **Configuration Files**: Commit configuration files to version control

```toml
[tools]
pre-commit = "3.6"
ruff = "latest"         # Managed by uv
ripgrep = "14.1"
fd = "10.2"
bat = "0.25"
biome = "1.8"
golangci-lint = "1.61"
cargo-audit = "0.21"
cargo-deny = "0.16"
cargo-nextest = "0.9"
hadolint = "2.12"
actionlint = "1.7"
```
