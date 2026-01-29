## Summary

This PR adds a new **Conda provider** to vx, enabling package, dependency, and environment management for Python and scientific computing (including CUDA, PyTorch, TensorFlow).

### What does this PR change?

**New Conda Provider** (`crates/vx-providers/conda/`)
- **micromamba**: Minimal standalone mamba - single binary, no installer needed (recommended)
- **conda**: Full Conda installation via Miniforge
- **mamba**: Fast package manager bundled with Miniforge

**Conda-tools Isolation Support**
- New `~/.vx/conda-tools/` directory for isolated conda environments
- Follows the same pattern as existing `pip-tools` and `npm-tools`
- PathManager and PathResolver support for conda-tools

### Why?

Conda/Mamba is essential for ML/AI development workflows that require:
- CUDA toolkit and GPU libraries
- PyTorch, TensorFlow with GPU support
- Scientific computing packages (NumPy, SciPy, etc.)
- Cross-platform binary package management

This fills a gap in vx's toolchain support for data science and machine learning developers.

## Type

- [x] feat
- [ ] fix
- [ ] docs
- [ ] refactor
- [ ] perf
- [ ] ci/chore

## Checklist

- [x] PR title follows Conventional Commits (e.g., feat: add X)
- [x] Tests added/updated when necessary
- [x] Docs updated (README/docs/CHANGELOG as needed)
- [ ] CI green

## Breaking changes

- [x] No breaking changes
- [ ] Breaking changes (describe):

## Additional context

### Directory Structure

```
~/.vx/
├── store/              # runtimes (node, go, micromamba...)
├── npm-tools/          # npm package tools
├── pip-tools/          # pip package tools
└── conda-tools/        # 🆕 conda package tools (isolated environments)
    └── <package>/<version>/env/
```

### Usage Examples

**Basic Usage**
```bash
# Install micromamba (recommended)
vx install micromamba

# Or install specific version
vx install micromamba@2.5.0-1

# List available versions
vx list micromamba
```

**Create & Use Conda Environments**
```bash
# Create environment with PyTorch + CUDA
vx micromamba create -n ml python=3.11 pytorch pytorch-cuda=12.1 -c pytorch -c nvidia

# Run command in environment
vx micromamba run -n ml python train.py

# Activate environment (interactive)
vx micromamba shell -n ml
```

**Project-level Configuration (vx.toml)**
```toml
[tools]
micromamba = "latest"

[env]
# Project-level isolation - conda envs stored in project directory
MAMBA_ROOT_PREFIX = "${VX_PROJECT_ROOT}/.mamba"

[scripts]
setup = "micromamba create -n dev python=3.11 pytorch -c pytorch -y"
train = "micromamba run -n dev python train.py"
```

**Why Micromamba?**

Micromamba is the recommended choice because it fits vx's installation model perfectly:

```
✅ micromamba: Download .tar.bz2 → Extract → Ready to use
❌ conda/mamba: Download .sh/.exe installer → Run installer → Then use
```

vx's design philosophy is **download → extract → use**, and micromamba is the only conda-compatible tool that supports this pattern as a standalone binary.

> **Note**: All three tools (micromamba, conda, mamba) are fully compatible - they use the same package repositories (conda-forge, anaconda) and can manage the same environments.

| Feature | micromamba | conda/mamba |
|---------|------------|-------------|
| Installation | Single binary ✅ | Requires installer |
| Size | ~10MB | ~400MB+ |
| Speed | Very fast | Fast (mamba) / Slow (conda) |
| Dependencies | None | Python runtime |
| vx compatible | Direct extract ✅ | Needs installer execution |
| Conda compatibility | Full | Native |

### Files Changed

| Path | Description |
|------|-------------|
| `crates/vx-providers/conda/` | New provider (6 files) |
| `crates/vx-paths/src/` | conda-tools path support |
| `crates/vx-runtime/src/` | conda-tools trait methods |
| `crates/vx-cli/` | Register conda provider |

### Platform Support

| Platform | micromamba | conda/mamba |
|----------|------------|-------------|
| Linux x86_64 | ✅ | ✅ |
| Linux aarch64 | ✅ | ✅ |
| macOS x86_64 | ✅ | ✅ |
| macOS arm64 | ✅ | ✅ |
| Windows x86_64 | ✅ | ✅ |
