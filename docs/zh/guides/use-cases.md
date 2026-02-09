# 实际使用案例

本指南展示了 vx 在各种开发场景中的实际应用。

## Windows C++ 开发与 MSVC

vx 提供便携式 MSVC Build Tools 支持，无需安装 Visual Studio 即可进行 C++ 编译。

### 基本 MSVC 使用

```bash
# 安装 MSVC Build Tools
vx install msvc

# 编译简单的 C++ 程序
vx cl main.cpp /Fe:main.exe

# 使用 nmake 进行构建自动化
vx nmake /f Makefile
```

### CMake 与 MSVC

vx 自动设置 MSVC 编译所需的环境变量（INCLUDE、LIB、PATH）：

```bash
# 一次安装多个工具
vx install msvc cmake ninja

# 使用 CMake 配置 MSVC
vx cmake -B build -G "Ninja" -DCMAKE_C_COMPILER=cl -DCMAKE_CXX_COMPILER=cl

# 构建项目
vx cmake --build build
```

### 环境变量

通过 vx 使用 MSVC 时，以下环境变量会自动配置：

| 变量 | 说明 |
|------|------|
| `INCLUDE` | MSVC 和 Windows SDK 头文件路径 |
| `LIB` | MSVC 和 Windows SDK 库文件路径 |
| `PATH` | MSVC 编译器二进制文件路径 |

这意味着您可以像使用 Visual Studio 开发者命令提示符中的 `cl.exe` 一样使用 `vx cl`。

## DCC（数字内容创作）开发

vx 对于 DCC 工具开发特别有用，您经常需要为 Maya、Houdini 和 Unreal Engine 等应用程序编译插件。

### Maya 插件开发

Maya 插件需要使用特定版本的 MSVC 编译。vx 使这变得简单：

```bash
# 安装所需的 MSVC 版本
vx install msvc@14.29  # VS 2019 用于 Maya 2024

# 设置项目
vx cmake -B build -G "Ninja" \
  -DCMAKE_C_COMPILER=cl \
  -DCMAKE_CXX_COMPILER=cl \
  -DMAYA_ROOT="C:/Program Files/Autodesk/Maya2024"

# 构建插件
vx cmake --build build
```

### Houdini 插件开发

Houdini HDK 开发同样受益于 vx 的便携式 MSVC：

```bash
# 安装 MSVC 和 CMake
vx install msvc cmake

# 配置 Houdini 插件构建
vx cmake -B build -G "Ninja" \
  -DCMAKE_C_COMPILER=cl \
  -DCMAKE_CXX_COMPILER=cl \
  -DHOUDINI_ROOT="C:/Program Files/Side Effects Software/Houdini 20.0"

# 构建
vx cmake --build build
```

### Unreal Engine 插件开发

对于 Unreal Engine C++ 开发：

```bash
# 安装 MSVC Build Tools
vx install msvc

# 使用 vx 管理的 MSVC 运行 Unreal Build Tool
# UBT 会自动检测编译器
```

## 使用 UV 进行 Python 开发

vx 与 uv 无缝集成，用于 Python 开发：

### 项目设置

```bash
# 安装 uv
vx install uv

# 创建新的 Python 项目
vx uv init my-project
cd my-project

# 添加依赖
vx uv add requests numpy pandas

# 运行脚本
vx uv run python main.py
```

### 虚拟环境管理

```bash
# 创建虚拟环境
vx uv venv

# 从 pyproject.toml 同步依赖
vx uv sync

# 运行测试
vx uv run pytest
```

## 使用 Just 进行任务自动化

vx 与 just 配合使用效果很好：

```bash
# 安装 just
vx install just

# 运行任务
vx just build

# 列出可用任务
vx just --list
```

### Justfile 示例

```makefile
# 构建项目
build:
    vx cmake --build build

# 运行测试
test:
    vx uv run pytest

# 清理构建产物
clean:
    rm -rf build/

# 完整 CI 流程
ci: build test
```

## Node.js 开发

### 项目设置

```bash
# 安装 Node.js 和 pnpm
vx install node pnpm

# 创建新项目
vx pnpm init

# 添加依赖
vx pnpm add express

# 运行开发服务器
vx pnpm run dev
```

### 使用 npx

```bash
# 创建 React 应用
vx npx create-react-app my-app

# 运行一次性命令
vx npx prettier --write .
```

## Go 开发

```bash
# 安装 Go
vx install go

# 初始化模块
vx go mod init myproject

# 构建项目
vx go build ./...

# 运行测试
vx go test ./...
```

## 多语言项目

vx 在使用多种语言的项目中表现出色：

```bash
# 安装所有需要的工具
vx install node uv go rust

# 或使用 vx.toml 进行声明式设置
cat > vx.toml << 'EOF'
[tools]
node = "22"
uv = "latest"
go = "1.22"
rust = "stable"
EOF

# 设置所有工具
vx setup
```

## CI/CD 集成

### GitHub Actions

```yaml
name: Build
on: [push, pull_request]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v6
      
      - name: Setup vx
        uses: loonghao/vx@main
        
      - name: Install tools
        run: vx install msvc cmake ninja
        
      - name: Build
        run: |
          vx cmake -B build -G Ninja
          vx cmake --build build
```

### GitLab CI

```yaml
build:
  image: ubuntu:latest
  script:
    - curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | sh
    - vx install node uv
    - vx pnpm install
    - vx pnpm run build
```

## 技巧和最佳实践

1. **使用 vx.toml 确保可重现性**：在 `vx.toml` 中定义工具版本，确保团队成员之间环境一致。

2. **利用自动安装**：当您尝试使用缺失的工具时，vx 会自动安装它们。

3. **与任务运行器结合**：将 vx 与 just 或 make 结合使用，实现强大的构建自动化。

4. **环境隔离**：每个 vx 管理的工具都是隔离的，防止版本冲突。

5. **跨平台脚本**：使用 vx 命令编写可在 Windows、macOS 和 Linux 上运行的脚本。
