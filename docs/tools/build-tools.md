# Build Tools

vx supports various build systems, compilers, and task runners.

## C/C++ Compilers

### MSVC Build Tools (Windows)

Microsoft Visual C++ compiler and build tools for Windows development.

```bash
# Install MSVC Build Tools
vx install msvc latest
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

## Task Runners

### Just

A handy way to save and run project-specific commands.

```bash
vx install just latest

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
vx install task latest

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
vx install cmake latest

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
vx install ninja latest

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
vx install protoc latest

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
vx install vite latest

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
