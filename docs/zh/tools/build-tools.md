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

## C/C++ 编译器

### MSVC 构建工具（Windows）

用于 Windows 开发的 Microsoft Visual C++ 编译器和构建工具。

```bash
# 安装 MSVC 构建工具
vx install `msvc@latest
vx install msvc 14.40       # 指定版本

# 通过命名空间使用 MSVC 工具（推荐）
vx msvc cl main.cpp -o main.exe
vx msvc link main.obj
vx msvc nmake
vx msvc lib /OUT:mylib.lib *.obj

# 直接别名（用于常用工具）
vx cl main.cpp              # 等同于：vx msvc cl
vx nmake                    # 等同于：vx msvc nmake

# 指定版本使用
vx msvc@14.40 cl main.cpp   # 使用 MSVC 14.40
vx msvc@14.29 cl legacy.cpp # 使用 MSVC 14.29 (VS2019)
```

**可用的 MSVC 工具：**

| 工具 | 命令 | 描述 |
|------|------|------|
| cl | `vx msvc cl` | C/C++ 编译器 |
| link | `vx msvc link` | 链接器 |
| lib | `vx msvc lib` | 库管理器 |
| nmake | `vx msvc nmake` / `vx nmake` | Make 工具 |
| ml64 | `vx msvc ml64` | MASM x64 汇编器 |
| dumpbin | `vx msvc dumpbin` | 二进制文件转储工具 |
| editbin | `vx msvc editbin` | 二进制文件编辑工具 |

**CMake + MSVC 工作流示例：**

```bash
# 使用 MSVC 配置
vx cmake -B build -G "NMake Makefiles"

# 构建
vx nmake -C build
```

**vx.toml 配置：**

```toml
[tools]
msvc = "14.40"

# 或使用详细配置
[tools.msvc]
version = "14.40"
sdk_version = "10.0.22621"
```

**配合其他工具使用 MSVC（伴随工具注入）：**

当 `vx.toml` 包含 MSVC 时，vx 会自动将 MSVC 发现环境变量
（`VCINSTALLDIR`、`VCToolsInstallDir`、`GYP_MSVS_VERSION` 等）注入到**所有**子进程环境中——
不仅限于 MSVC 工具本身。这使得任何需要 C/C++ 编译器的工具都能发现 vx 管理的 MSVC 安装，
无需完整安装 Visual Studio。

支持的场景包括：
- **node-gyp** / **Electron**：`vx npx node-gyp rebuild`
- **CMake**：`vx cmake -B build`（通过 `VCINSTALLDIR` 自动检测 MSVC）
- **Cargo**（cc crate）：`vx cargo build` 编译包含 C 依赖的 crate
- **.NET Native AOT**：`vx dotnet publish -c Release`
- **Meson**：`vx meson setup build`

```toml
# vx.toml — MSVC 环境变量会注入到这里列出的所有工具
[tools]
node = "22"
cmake = "3.28"
rust = "1.82"

[tools.msvc]
version = "14.42"
os = ["windows"]
```

```bash
# node-gyp 会通过 VCINSTALLDIR 自动找到 MSVC
vx npx node-gyp rebuild

# CMake 也能发现编译器
vx cmake -B build -G "Ninja"

# Cargo cc crate 找到 MSVC 编译 C 依赖
vx cargo build

# 从任何工具都能验证环境变量已设置
vx node -e "console.log('VCINSTALLDIR:', process.env.VCINSTALLDIR)"
# 输出：VCINSTALLDIR: C:\Users\you\.vx\store\msvc\14.42\VC\
```

**伴随工具注入机制：**

vx 使用"伴随工具"机制：当执行任何工具（如 `vx node`、`vx cmake`）时，
vx 还会为 `vx.toml` 中定义的所有其他工具调用 `prepare_environment()`。
这会注入发现/标记环境变量，而不会污染完整的编译环境（LIB/INCLUDE/PATH）。

MSVC 伴随工具注入的环境变量：

| 变量 | 示例 | 用途 |
|------|------|------|
| `VCINSTALLDIR` | `C:\...\VC\` | VS 安装路径（node-gyp、CMake 等使用） |
| `VCToolsInstallDir` | `C:\...\VC\Tools\MSVC\14.42.34433\` | 精确工具链路径 |
| `VSCMD_VER` | `17.0` | VS 命令提示符版本 |
| `GYP_MSVS_VERSION` | `2022` | node-gyp VS 版本提示 |
| `VX_MSVC_ROOT` | `C:\...\store\msvc\14.42` | vx MSVC 根路径 |
| `VX_MSVC_FULL_VERSION` | `14.42.34433` | 完整 MSVC 版本号 |

## vcpkg - C++ 包管理器

vcpkg 是一个 C++ 库管理器，简化了 C++ 库及其依赖项的安装。对于需要 C++ 依赖的原生 Node.js 模块特别有用。

### 安装

```bash
# 安装 vcpkg
vx install vcpkg

# 这会从 GitHub 克隆 vcpkg 并进行引导
```

### 安装 C++ 包

```bash
# 安装 C++ 库
vx vcpkg install openssl

# 为特定 triplet 安装
vx vcpkg install openssl:x64-windows
vx vcpkg install openssl:x64-windows-static

# 搜索包
vx vcpkg search sqlite
```

### 原生 Node.js 模块常用包

| 包名 | 描述 |
|------|------|
| `winpty` | 终端仿真库（node-pty 必需） |
| `openssl` | OpenSSL 库 |
| `sqlite3` | SQLite 数据库 |
| `libpng` | PNG 库 |
| `zstd` | Zstandard 压缩库 |

### 与 MSVC 集成

当同时安装 vcpkg 和 MSVC 时，vx 会自动将 vcpkg 路径集成到 MSVC 环境中。这使得原生 Node.js 模块无需额外配置即可找到 C++ 库。

```bash
# 安装 vcpkg 和 MSVC
vx install vcpkg
vx install msvc

# 安装 winpty（用于 node-pty）
vx vcpkg install winpty

# 在 Electron 项目中构建 node-pty
vx npm install node-pty
```

### 环境变量

vcpkg 设置以下环境变量：

| 变量 | 描述 |
|------|------|
| `VCPKG_ROOT` | vcpkg 安装路径 |
| `CMAKE_TOOLCHAIN_FILE` | CMake 集成用的 vcpkg.cmake 路径 |
| `VCPKG_DEFAULT_TRIPLET` | 默认 triplet（如 x64-windows） |

### 与 CMake 配合使用

```bash
# CMake 通过 CMAKE_TOOLCHAIN_FILE 自动检测 vcpkg
vx cmake -B build -S .
vx cmake --build build
```

### vx.toml 配置

```toml
[tools]
vcpkg = "latest"
msvc = "14.42"
cmake = "3.28"

# 项目特定的 C++ 依赖
[dependencies.cpp]
vcpkg_packages = ["winpty", "openssl"]
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
