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

**Using MSVC with Node.js native modules (node-gyp):**

When `vx.toml` specifies both Node.js and MSVC, vx automatically injects MSVC discovery
environment variables (`VCINSTALLDIR`, `VCToolsInstallDir`, `GYP_MSVS_VERSION`, etc.) into
all subprocess environments. This allows tools like node-gyp to find the vx-managed MSVC
compiler without a full Visual Studio installation.

```toml
# vx.toml
[tools]
node = "22"

[tools.msvc]
version = "14.42"
os = ["windows"]
```

```bash
# node-gyp will automatically find MSVC via VCINSTALLDIR
vx npx node-gyp rebuild

# Electron native modules also work
vx npx electron-builder install-app-deps

# Verify environment variables are set
vx node -e "console.log('VCINSTALLDIR:', process.env.VCINSTALLDIR)"
# Output: VCINSTALLDIR: C:\Users\you\.vx\store\msvc\14.42\VC\
```

**How companion tool injection works:**

vx uses a "companion tools" mechanism: when executing any tool (e.g., `vx node`),
vx also calls `prepare_environment()` for all other tools defined in `vx.toml`.
This injects discovery/marker environment variables without polluting the full
compilation environment (LIB/INCLUDE/PATH), which would break node-gyp's own
Visual Studio discovery logic.

Environment variables injected by MSVC companion:

| Variable | Example | Purpose |
|----------|---------|---------|
| `VCINSTALLDIR` | `C:\...\VC\` | VS install path (node-gyp discovery) |
| `VCToolsInstallDir` | `C:\...\VC\Tools\MSVC\14.42.34433\` | Exact toolchain path |
| `VSCMD_VER` | `17.0` | VS Command Prompt version |
| `GYP_MSVS_VERSION` | `2022` | node-gyp VS version hint |
| `VX_MSVC_ROOT` | `C:\...\store\msvc\14.42` | vx MSVC root path |
| `VX_MSVC_FULL_VERSION` | `14.42.34433` | Full MSVC version |

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
