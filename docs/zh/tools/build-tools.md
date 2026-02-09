# 构建工具

vx 支持各种构建系统和任务运行器。

## .NET SDK 和 MSBuild

### .NET SDK

.NET SDK 包含 dotnet CLI、MSBuild、NuGet 以及 C#、F# 和 VB.NET 编译器。

```bash
# 安装 .NET SDK
vx install `dotnet@latest
vx install dotnet 8.0        # LTS 版本

# 常用命令
vx dotnet --version
vx dotnet new console -n MyApp
vx dotnet build
vx dotnet run
vx dotnet test
vx dotnet publish -c Release
```

### MSBuild（与 .NET SDK 捆绑）

MSBuild 是 Microsoft 构建引擎，与 .NET SDK 捆绑在一起。当你运行 `vx msbuild` 时，vx 会自动使用 `dotnet msbuild`。

```bash
# MSBuild 命令（使用 dotnet msbuild）
vx msbuild MyProject.csproj
vx msbuild MySolution.sln /p:Configuration=Release
vx msbuild /t:Build /p:Platform=x64
vx msbuild /t:Clean
vx msbuild /t:Restore

# 等同于：
vx dotnet msbuild MyProject.csproj
```

**RFC 0028：捆绑运行时**

MSBuild 是"捆绑运行时"——它不能独立安装，而是随 .NET SDK 一起提供。当你运行 `vx msbuild` 时，vx 会：
1. 查找 .NET SDK 安装（vx 管理的或系统的）
2. 通过 `dotnet msbuild` 执行
3. 在 Windows 上，如果没有 .NET SDK，也可以使用 Visual Studio 的独立 MSBuild

**示例 .NET 构建工作流：**

```bash
# 创建新项目
vx dotnet new webapi -n MyApi

# 使用 MSBuild 构建
cd MyApi
vx msbuild MyApi.csproj /p:Configuration=Release

# 或使用 dotnet build（更简单）
vx dotnet build -c Release
```

## 任务运行器

### Just

保存和运行项目特定命令的便捷工具。

```bash
vx install `just@latest

vx just --version
vx just --list
vx just build
vx just test
vx just deploy
```

**Justfile 示例：**

```makefile
# 构建项目
build:
    cargo build --release

# 运行测试
test:
    cargo test

# 格式化代码
fmt:
    cargo fmt
```

### Task (go-task)

用 Go 编写的任务运行器 / Make 替代品。

```bash
vx install `task@latest

vx task --version
vx task --list
vx task build
vx task test
```

**Taskfile.yml 示例：**

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

## 构建系统

### CMake

跨平台构建系统生成器。

```bash
vx install `cmake@latest

vx cmake --version
vx cmake -B build -S .
vx cmake --build build
vx cmake --build build --config Release
vx cmake --install build
```

**常见 CMake 工作流：**

```bash
# 配置
vx cmake -B build -DCMAKE_BUILD_TYPE=Release

# 构建
vx cmake --build build --parallel

# 安装
vx cmake --install build --prefix /usr/local
```

### Ninja

专注于速度的小型构建系统。

```bash
vx install `ninja@latest

vx ninja --version
vx ninja -C build
vx ninja -C build clean
vx ninja -C build -j 8
```

**与 CMake 配合使用：**

```bash
vx cmake -B build -G Ninja
vx ninja -C build
```

### protoc

Protocol Buffers 编译器。

```bash
vx install `protoc@latest

vx protoc --version
vx protoc --cpp_out=. message.proto
vx protoc --python_out=. message.proto
vx protoc --go_out=. message.proto
vx protoc --rust_out=. message.proto
```

## 前端构建工具

### Vite

下一代前端工具。

```bash
vx install `vite@latest

vx vite --version
vx vite                    # 启动开发服务器
vx vite build             # 生产构建
vx vite preview           # 预览生产构建
```

## 项目配置示例

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
