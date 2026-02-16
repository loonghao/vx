# Build Tools

vx supports various build systems, compilers, and task runners.

## .NET SDK & MSBuild

### .NET SDK

The .NET SDK includes the dotnet CLI, MSBuild, NuGet, and compilers for C#, F#, and VB.NET.

```bash
# Install .NET SDK
vx install `dotnet@latest
vx install dotnet 8.0        # LTS version

# Common commands
vx dotnet --version
vx dotnet new console -n MyApp
vx dotnet build
vx dotnet run
vx dotnet test
vx dotnet publish -c Release
```

### MSBuild (Bundled with .NET SDK)

MSBuild is the Microsoft Build Engine, bundled with .NET SDK. vx automatically uses `dotnet msbuild` when you run `vx msbuild`.

```bash
# MSBuild commands (uses dotnet msbuild)
vx msbuild MyProject.csproj
vx msbuild MySolution.sln /p:Configuration=Release
vx msbuild /t:Build /p:Platform=x64
vx msbuild /t:Clean
vx msbuild /t:Restore

# Equivalent to:
vx dotnet msbuild MyProject.csproj
```

**RFC 0028: Bundled Runtime**

MSBuild is a "bundled runtime" - it's not independently installable but comes with .NET SDK. When you run `vx msbuild`, vx:
1. Finds the .NET SDK installation (vx-managed or system)
2. Executes via `dotnet msbuild`
3. On Windows, can also use Visual Studio's standalone MSBuild if .NET SDK is not available

**Example .NET Build Workflow:**

```bash
# Create new project
vx dotnet new webapi -n MyApi

# Build with MSBuild
cd MyApi
vx msbuild MyApi.csproj /p:Configuration=Release

# Or use dotnet build (simpler)
vx dotnet build -c Release
```

## C/C++ Compilers

### MSVC Build Tools (Windows)

Microsoft Visual C++ compiler and build tools for Windows development.

```bash
# Install MSVC Build Tools
vx install `msvc@latest
vx install msvc 14.40       # Specific version

# Using MSVC tools via namespace (recommended)
vx msvc cl main.cpp -o main.exe
vx msvc link main.obj
vx msvc nmake
vx msvc lib /OUT:mylib.lib *.obj

# Direct aliases (for common tools)
vx cl main.cpp              # Same as: vx msvc cl
vx nmake                    # Same as: vx msvc nmake

# Version-specific usage
vx msvc@14.40 cl main.cpp   # Use MSVC 14.40
vx msvc@14.29 cl legacy.cpp # Use MSVC 14.29 (VS2019)
```

**Available MSVC Tools:**

| Tool | Command | Description |
|------|---------|-------------|
| cl | `vx msvc cl` | C/C++ compiler |
| link | `vx msvc link` | Linker |
| lib | `vx msvc lib` | Library manager |
| nmake | `vx msvc nmake` / `vx nmake` | Make utility |
| ml64 | `vx msvc ml64` | MASM x64 assembler |
| dumpbin | `vx msvc dumpbin` | Binary file dumper |
| editbin | `vx msvc editbin` | Binary file editor |

**Example CMake + MSVC Workflow:**

```bash
# Configure with MSVC
vx cmake -B build -G "NMake Makefiles"

# Build
vx nmake -C build
```

**vx.toml Configuration:**

```toml
[tools]
msvc = "14.40"

# Or with detailed configuration
[tools.msvc]
version = "14.40"
sdk_version = "10.0.22621"
```

**Using MSVC with other tools (companion tool injection):**

When `vx.toml` includes MSVC, vx automatically injects MSVC discovery environment variables
(`VCINSTALLDIR`, `VCToolsInstallDir`, `GYP_MSVS_VERSION`, etc.) into **all** subprocess
environments — not just MSVC tools. This allows any tool that needs a C/C++ compiler to
discover the vx-managed MSVC installation without a full Visual Studio installation.

Supported scenarios include:
- **node-gyp** / **Electron**: `vx npx node-gyp rebuild`
- **CMake**: `vx cmake -B build` (auto-detects MSVC via `VCINSTALLDIR`)
- **Cargo** (cc crate): `vx cargo build` for crates with C dependencies
- **.NET Native AOT**: `vx dotnet publish -c Release`
- **Meson**: `vx meson setup build`

```toml
# vx.toml — MSVC env vars are injected for ALL tools listed here
[tools]
node = "22"
cmake = "3.28"
rust = "1.82"

[tools.msvc]
version = "14.42"
os = ["windows"]
```

```bash
# node-gyp will automatically find MSVC via VCINSTALLDIR
vx npx node-gyp rebuild

# CMake also discovers the compiler
vx cmake -B build -G "Ninja"

# Cargo cc crate finds MSVC for C dependencies
vx cargo build

# Verify environment variables are set from any tool
vx node -e "console.log('VCINSTALLDIR:', process.env.VCINSTALLDIR)"
# Output: VCINSTALLDIR: C:\Users\you\.vx\store\msvc\14.42\VC\
```

**How companion tool injection works:**

vx uses a "companion tools" mechanism: when executing any tool (e.g., `vx node`, `vx cmake`),
vx also calls `prepare_environment()` for all other tools defined in `vx.toml`.
This injects discovery/marker environment variables without polluting the full
compilation environment (LIB/INCLUDE/PATH).

Environment variables injected by MSVC companion:

| Variable | Example | Purpose |
|----------|---------|---------|
| `VCINSTALLDIR` | `C:\...\VC\` | VS install path (used by node-gyp, CMake, etc.) |
| `VCToolsInstallDir` | `C:\...\VC\Tools\MSVC\14.42.34433\` | Exact toolchain path |
| `VSCMD_VER` | `17.0` | VS Command Prompt version |
| `GYP_MSVS_VERSION` | `2022` | node-gyp VS version hint |
| `VX_MSVC_ROOT` | `C:\...\store\msvc\14.42` | vx MSVC root path |
| `VX_MSVC_FULL_VERSION` | `14.42.34433` | Full MSVC version |

## vcpkg - C++ Package Manager

vcpkg is a C++ library manager that simplifies the installation of C++ libraries and their dependencies. It is particularly useful for native Node.js modules that require C++ dependencies.

### Installation

```bash
# Install vcpkg
vx install vcpkg

# This downloads vcpkg-tool binary and shallow-clones the vcpkg registry from GitHub
# Requires git to be available on PATH
```

### vx-Managed Cache Directories

vcpkg uses vx-managed cache directories to store downloads and binary caches:

| Directory | Purpose | Location |
|-----------|---------|----------|
| Downloads | Source archives and assets | `~/.vx/cache/vcpkg/downloads/` |
| Archives | Binary cache for compiled packages | `~/.vx/cache/vcpkg/archives/` |

This ensures:
- **Consistent storage**: All vcpkg artifacts are in the vx cache directory
- **Easy cleanup**: Remove `~/.vx/cache/vcpkg/` to clear all vcpkg caches
- **Shared across versions**: Multiple vcpkg versions share the same cache

vcpkg itself is installed at `~/.vx/store/vcpkg/<version>/` (e.g., `~/.vx/store/vcpkg/2025.12.16/`). The installation includes a shallow clone of the vcpkg registry (triplets, scripts, ports, versions) with documentation excluded to save disk space.

### Uninstalling

```bash
# Uninstall vcpkg
vx uninstall vcpkg

# This removes the installation directory (including the registry clone).
# Shared caches at ~/.vx/cache/vcpkg/ are preserved.
# To clean caches manually:
# rm -rf ~/.vx/cache/vcpkg/
```

### Installing C++ Packages

```bash
# Install a C++ library
vx vcpkg install openssl

# Install for a specific triplet
vx vcpkg install openssl:x64-windows
vx vcpkg install openssl:x64-windows-static

# Search for packages
vx vcpkg search sqlite
```

### Common Packages for Native Node.js Modules

| Package | Description |
|---------|-------------|
| `winpty` | Terminal emulation library (required by node-pty) |
| `openssl` | OpenSSL library |
| `sqlite3` | SQLite database |
| `libpng` | PNG library |
| `zstd` | Zstandard compression |

### Integration with MSVC

When both vcpkg and MSVC are installed, vx automatically integrates vcpkg paths into the MSVC environment. This allows native Node.js modules to find C++ libraries without additional configuration.

```bash
# Install vcpkg and MSVC
vx install vcpkg
vx install msvc

# Install winpty (for node-pty)
vx vcpkg install winpty

# Build node-pty in your Electron project
vx npm install node-pty
```

### Environment Variables

vcpkg sets the following environment variables:

| Variable | Description |
|----------|-------------|
| `VCPKG_ROOT` | Path to vcpkg installation |
| `CMAKE_TOOLCHAIN_FILE` | Path to vcpkg.cmake for CMake integration |
| `VCPKG_DEFAULT_TRIPLET` | Default triplet (e.g., x64-windows) |
| `VCPKG_DOWNLOADS` | vx-managed downloads cache directory |
| `VCPKG_DEFAULT_BINARY_CACHE` | vx-managed binary cache directory |
| `INCLUDE` | Prepended with vcpkg installed headers path |
| `LIB` | Prepended with vcpkg installed library path |

### Using with CMake

```bash
# CMake automatically detects vcpkg via CMAKE_TOOLCHAIN_FILE
vx cmake -B build -S .
vx cmake --build build
```

### vx.toml Configuration

```toml
[tools]
vcpkg = "latest"
msvc = "14.42"
cmake = "3.28"

# Project-specific C++ dependencies
[dependencies.cpp]
vcpkg_packages = ["winpty", "openssl"]
```

## Task Runners

### Just

A handy way to save and run project-specific commands.

```bash
vx install `just@latest

vx just --version
vx just --list
vx just build
vx just test
vx just deploy
```

**Example Justfile:**

```makefile
# Build the project
build:
    cargo build --release

# Run tests
test:
    cargo test

# Format code
fmt:
    cargo fmt
```

### Task (go-task)

Task runner / simpler Make alternative written in Go.

```bash
vx install `task@latest

vx task --version
vx task --list
vx task build
vx task test
```

**Example Taskfile.yml:**

```yaml
version: '3'

tasks:
  build:
    cmds:
      - go build -o app .

  test:
    cmds:
      - go test ./...
```

## Build Systems

### CMake

Cross-platform build system generator.

```bash
vx install `cmake@latest

vx cmake --version
vx cmake -B build -S .
vx cmake --build build
vx cmake --build build --config Release
vx cmake --install build
```

**Common CMake Workflow:**

```bash
# Configure
vx cmake -B build -DCMAKE_BUILD_TYPE=Release

# Build
vx cmake --build build --parallel

# Install
vx cmake --install build --prefix /usr/local
```

### Ninja

Small build system with a focus on speed.

```bash
vx install `ninja@latest

vx ninja --version
vx ninja -C build
vx ninja -C build clean
vx ninja -C build -j 8
```

**Using with CMake:**

```bash
vx cmake -B build -G Ninja
vx ninja -C build
```

### protoc

Protocol Buffers compiler.

```bash
vx install `protoc@latest

vx protoc --version
vx protoc --cpp_out=. message.proto
vx protoc --python_out=. message.proto
vx protoc --go_out=. message.proto
vx protoc --rust_out=. message.proto
```

## Frontend Build Tools

### Vite

Next generation frontend tooling.

```bash
vx install `vite@latest

vx vite --version
vx vite                    # Start dev server
vx vite build             # Build for production
vx vite preview           # Preview production build
```

## Project Configuration Example

```toml
[tools]
just = "latest"
task = "latest"
cmake = "3.28"
ninja = "latest"
protoc = "latest"
vite = "latest"

[scripts]
build = "just build"
cmake-build = "cmake -B build && cmake --build build"
proto-gen = "protoc --go_out=. *.proto"
dev = "vite"
```
