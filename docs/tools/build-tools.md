# Build Tools

vx supports various build systems and task runners.

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
