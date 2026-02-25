# Build Cache Tools

vx supports compilation cache tools to speed up builds across projects and sessions. These tools cache compilation results, avoiding redundant work.

## Overview

### Compiler Cache Tools

| Tool | Languages | Best For | Speed Improvement |
|------|-----------|----------|-------------------|
| **sccache** | Rust, C/C++, CUDA | Cross-language, CI/CD | 20-50% |
| **ccache** | C/C++ | Native C/C++ projects | 30-60% |
| **buildcache** | C/C++, CUDA | MSVC, Visual Studio | 30-50% |

### Node.js Build Cache Tools

| Tool | Type | Best For | Speed Improvement |
|------|------|----------|-------------------|
| **Nx** | Monorepo + Cache | Large monorepos | 50-90% |
| **Turborepo** | Monorepo + Cache | Medium monorepos | 50-90% |

## sccache

**sccache** is Mozilla's shared compilation cache, supporting multiple compilers and languages.

### Installation

```bash
# Install via vx
vx install sccache

# Or auto-install on first use
vx sccache --version
```

### Supported Compilers

| Language | Compiler | Support |
|----------|----------|---------|
| Rust | rustc | ✅ Full |
| C/C++ | GCC | ✅ Full |
| C/C++ | Clang | ✅ Full |
| C/C++ | MSVC (cl.exe) | ✅ Full |
| CUDA | nvcc | ✅ Full |

### Basic Usage

```bash
# Start the cache server
vx sccache --start-server

# View cache statistics
vx sccache --show-stats

# Reset statistics
vx sccache --zero-stats

# Stop the cache server
vx sccache --stop-server
```

### Configuration

#### Environment Variables

```bash
# Cache size limit (default: 10G)
export SCCACHE_CACHE_SIZE="20G"

# Cache directory
export SCCACHE_DIR="$HOME/.cache/sccache"

# Log level (error, warn, info, debug, trace)
export SCCACHE_LOG="info"

# Enable debug logging
export SCCACHE_DEBUG=1
```

#### Rust Integration (Automatic)

sccache is automatically enabled for Rust when configured in `.cargo/config.toml`:

```toml
[build]
rustc-wrapper = "sccache"
```

#### C/C++ Integration

```bash
# GCC/Clang
export CC="sccache gcc"
export CXX="sccache g++"

# Or use compiler-specific wrapper
export CC="sccache cc"
export CXX="sccache c++"
```

#### CMake Integration

```cmake
# CMakeLists.txt
set(CMAKE_C_COMPILER_LAUNCHER "sccache")
set(CMAKE_CXX_COMPILER_LAUNCHER "sccache")

# Or via command line
cmake -DCMAKE_C_COMPILER_LAUNCHER=sccache \
      -DCMAKE_CXX_COMPILER_LAUNCHER=sccache \
      -B build
```

### Remote Cache Backends

sccache supports remote cache backends for team sharing:

```bash
# Amazon S3
export SCCACHE_BUCKET="my-sccache-bucket"
export SCCACHE_REGION="us-east-1"
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."

# Google Cloud Storage
export SCCACHE_GCS_BUCKET="my-sccache-bucket"
export SCCACHE_GCS_OAUTH2_URL="..."

# Redis
export SCCACHE_REDIS="redis://localhost:6379"

# Azure Blob Storage
export SCCACHE_AZURE_CONNECTION_STRING="..."
```

### vx.toml Configuration

```toml
[tools]
sccache = "latest"

[env]
SCCACHE_CACHE_SIZE = "20G"
SCCACHE_DIR = "$HOME/.cache/sccache"
```

## ccache

**ccache** is the classic C/C++ compiler cache, widely used and highly optimized.

### Installation

```bash
# Install via vx
vx install ccache

# Check version
vx ccache --version
```

### Supported Compilers

| Compiler | Support |
|----------|---------|
| GCC | ✅ Full |
| Clang | ✅ Full |
| MSVC | ❌ Not supported |
| NVIDIA nvcc | ✅ Full |

### Basic Usage

```bash
# View statistics
vx ccache -s

# Clear cache
vx ccache -C

# Reset statistics
vx ccache -z

# Set max cache size
vx ccache -M 20G

# Show config
vx ccache -p
```

### Configuration

#### Environment Variables

```bash
# Max cache size
export CCACHE_MAXSIZE="20G"

# Cache directory
export CCACHE_DIR="$HOME/.cache/ccache"

# Compression
export CCACHE_COMPRESS="true"
export CCACHE_COMPRESSLEVEL="6"

# Hard link instead of copy (faster)
export CCACHE_HARDLINK="true"
```

#### Usage with GCC/Clang

```bash
# Method 1: Override compiler
export CC="ccache gcc"
export CXX="ccache g++"

# Method 2: Symlink (ccache in PATH before gcc)
export PATH="/usr/lib/ccache:$PATH"

# Method 3: CMake integration
cmake -DCMAKE_C_COMPILER_LAUNCHER=ccache \
      -DCMAKE_CXX_COMPILER_LAUNCHER=ccache \
      -B build
```

### vx.toml Configuration

```toml
[tools]
ccache = "latest"

[env]
CCACHE_MAXSIZE = "20G"
CCACHE_DIR = "$HOME/.cache/ccache"
CCACHE_COMPRESS = "true"
```

## buildcache

**buildcache** is a compiler cache with excellent MSVC support.

### Installation

```bash
# Install via vx
vx install buildcache

# Check version
vx buildcache --version
```

### Supported Compilers

| Compiler | Support |
|----------|---------|
| MSVC (cl.exe) | ✅ Full |
| GCC | ✅ Full |
| Clang | ✅ Full |
| CUDA (nvcc) | ✅ Full |

### Basic Usage

```bash
# View statistics
vx buildcache -s

# Clear cache
vx buildcache -C

# Set max cache size
vx buildcache -m 20G
```

### Configuration

#### Environment Variables

```bash
# Max cache size
export BUILDCACHE_MAX_CACHE_SIZE="20000000000"  # 20GB in bytes

# Cache directory
export BUILDCACHE_DIR="$HOME/.cache/buildcache"

# Enable debug logging
export BUILDCACHE_DEBUG="true"
```

#### MSVC Integration

```cmake
# CMakeLists.txt for MSVC
set(CMAKE_C_COMPILER_LAUNCHER "buildcache")
set(CMAKE_CXX_COMPILER_LAUNCHER "buildcache")

# Or use as compiler wrapper
set(CMAKE_C_COMPILER "buildcache cl")
set(CMAKE_CXX_COMPILER "buildcache cl")
```

### vx.toml Configuration

```toml
[tools]
buildcache = "latest"

[env]
BUILDCACHE_MAX_CACHE_SIZE = "20000000000"
BUILDCACHE_DIR = "$HOME/.cache/buildcache"
```

## Nx

**Nx** is a smart build system for monorepos with powerful caching capabilities. It caches build artifacts and test results, supporting both local and remote caching.

### Installation

```bash
# Install via vx (routes to npx nx)
vx nx --version

# Or auto-install on first use
vx nx build myapp
```

### Features

- **Local Caching**: Caches build outputs locally for fast rebuilds
- **Remote Caching**: Share cache across team members via Nx Cloud
- **Affected Project Detection**: Only build/test projects affected by changes
- **Task Orchestration**: Parallel task execution with dependency management
- **Code Generation**: Built-in generators for common patterns

### Basic Usage

```bash
# Build with cache
vx nx build myapp

# Test with cache
vx nx test myapp

# Build all affected projects
vx nx affected -t build

# Skip cache
vx nx build myapp --skip-nx-cache

# Clear cache
vx nx reset
```

### Configuration

#### nx.json

```json
{
  "tasksRunnerOptions": {
    "default": {
      "runner": "nx/tasks-runners/default",
      "options": {
        "cacheableOperations": ["build", "test", "lint", "e2e"],
        "parallel": 3
      }
    }
  }
}
```

#### Environment Variables

```bash
# Custom cache directory
export NX_CACHE_DIRECTORY=".nx-cache"

# Disable daemon
export NX_DAEMON=false

# Enable verbose logging
export NX_VERBOSE_LOGGING=true
```

### Remote Cache (Nx Cloud)

```bash
# Connect to Nx Cloud
vx nx connect

# View cache statistics
# (via Nx Cloud dashboard)
```

### vx.toml Configuration

```toml
[tools]
nx = "latest"

[env]
NX_CACHE_DIRECTORY = ".nx-cache"
```

### Use Cases

| Use Case | Recommended |
|----------|-------------|
| Angular monorepo | ✅ Nx (native support) |
| React monorepo | ✅ Nx (native support) |
| Node.js monorepo | ✅ Nx |
| Mixed language monorepo | ⚠️ Consider Turborepo |

## Turborepo

**Turborepo** is a high-performance build system for JavaScript/TypeScript monorepos. Written in Rust, it provides intelligent caching and parallel task execution.

### Installation

```bash
# Install via vx (routes to npx turbo)
vx turbo --version

# Or auto-install on first use
vx turbo build
```

### Features

- **Local Caching**: Content-aware hashing for precise cache invalidation
- **Remote Caching**: Built-in Vercel integration for team cache sharing
- **Parallel Execution**: Automatic task parallelization
- **Incremental Builds**: Only rebuild what changed
- **Prune Command**: Create minimal deployment-ready subset

### Basic Usage

```bash
# Build with cache
vx turbo build

# Run multiple tasks
vx turbo build test lint

# Force rebuild (skip cache)
vx turbo build --force

# Custom cache directory
vx turbo build --cache-dir=./cache

# Prune for deployment
vx turbo prune --scope=web --docker
```

### Configuration

#### turbo.json

```json
{
  "$schema": "https://turbo.build/schema.json",
  "pipeline": {
    "build": {
      "outputs": [".next/**", "!.next/cache/**"],
      "dependsOn": ["^build"]
    },
    "test": {
      "dependsOn": ["build"],
      "outputs": ["coverage/**"]
    },
    "lint": {
      "outputs": []
    }
  }
}
```

#### Environment Variables

```bash
# Custom cache directory
export TURBO_CACHE_DIR="./turbo-cache"

# Remote cache team
export TURBO_TEAM="my-team"

# Remote cache token
export TURBO_TOKEN="xxx"

# Enable remote cache only
export TURBO_REMOTE_ONLY=true
```

### Remote Cache (Vercel)

```bash
# Login to Vercel
vx npx turbo login

# Link to remote cache
vx npx turbo link

# View cache usage
# (via Vercel dashboard)
```

### vx.toml Configuration

```toml
[tools]
turbo = "latest"

[env]
TURBO_CACHE_DIR = "./turbo-cache"
TURBO_TEAM = "my-team"
```

### Use Cases

| Use Case | Recommended |
|----------|-------------|
| Next.js monorepo | ✅ Turborepo (native support) |
| React monorepo | ✅ Turborepo |
| Node.js monorepo | ✅ Turborepo |
| Simple monorepo | ✅ Turborepo |

## Comparison

### Compiler Cache Performance

| Scenario | sccache | ccache | buildcache |
|----------|---------|--------|------------|
| Rust projects | ⭐⭐⭐⭐⭐ | N/A | N/A |
| C/C++ GCC/Clang | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| C/C++ MSVC | ⭐⭐⭐⭐ | ❌ | ⭐⭐⭐⭐⭐ |
| CUDA | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Remote cache | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ❌ |

### Node.js Build Cache Performance

| Scenario | Nx | Turborepo |
|----------|-----|-----------|
| Large monorepo (50+ packages) | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Medium monorepo (10-50 packages) | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Small monorepo (<10 packages) | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Remote cache | ⭐⭐⭐⭐⭐ (Nx Cloud) | ⭐⭐⭐⭐⭐ (Vercel) |
| Affected project detection | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |

### Recommendations

#### Compiler Cache

| Use Case | Recommended Tool |
|----------|-----------------|
| Rust projects | **sccache** |
| C/C++ on Linux/macOS | **ccache** or sccache |
| C/C++ on Windows MSVC | **buildcache** or sccache |
| CI/CD with remote cache | **sccache** |
| Mixed Rust + C/C++ | **sccache** |

#### Node.js Build Cache

| Use Case | Recommended Tool |
|----------|-----------------|
| Angular monorepo | **Nx** |
| Large enterprise monorepo | **Nx** |
| Next.js monorepo | **Turborepo** |
| Simple monorepo | **Turborepo** |
| Already using Vercel | **Turborepo** |

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/build.yml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup sccache
        run: |
          curl -fsSL https://get.vx.dev | bash
          vx install sccache

      - name: Configure cache
        run: |
          echo "SCCACHE_CACHE_SIZE=20G" >> $GITHUB_ENV
          echo "SCCACHE_DIR=$HOME/.cache/sccache" >> $GITHUB_ENV

      - name: Cache sccache
        uses: actions/cache@v4
        with:
          path: ~/.cache/sccache
          key: sccache-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            sccache-${{ runner.os }}-

      - name: Build
        run: cargo build --release

      - name: Show cache stats
        run: vx sccache --show-stats
```

### GitLab CI

```yaml
# .gitlab-ci.yml
variables:
  SCCACHE_CACHE_SIZE: "20G"
  SCCACHE_DIR: "$CI_PROJECT_DIR/.cache/sccache"

cache:
  paths:
    - .cache/sccache/

build:
  script:
    - curl -fsSL https://get.vx.dev | bash
    - vx install sccache
    - cargo build --release
    - vx sccache --show-stats
```

## Troubleshooting

### Cache Not Working

```bash
# Check if wrapper is active
which sccache
sccache --show-stats

# Check environment
echo $SCCACHE_CACHE_SIZE
echo $SCCACHE_DIR

# Restart server
sccache --stop-server
sccache --start-server
```

### Cache Directory Full

```bash
# Clear cache
sccache --stop-server
rm -rf ~/.cache/sccache
sccache --start-server

# Or increase limit
export SCCACHE_CACHE_SIZE="50G"
```

### Permission Errors

```bash
# Check directory permissions
ls -la ~/.cache/sccache

# Fix permissions
chmod -R u+rw ~/.cache/sccache
```

## See Also

- [Build Tools](/tools/build-tools) - Build systems and compilers
- [Best Practices](/guide/best-practices) - Performance tips
- [Environment Variables](/config/env-vars) - Configuration options
