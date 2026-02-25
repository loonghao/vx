# 隐式包执行

执行全局安装包或按需运行包，无需显式安装，类似于 `npx` 和 `uvx`，但支持跨语言。

## 概述

隐式包执行功能允许你使用统一语法直接运行包。与 `npx`（仅 Node.js）或 `uvx`（仅 Python）不同，vx 支持多个生态系统，提供一致的界面。

**主要优势：**
- 🚀 **一键执行**：无需预先安装即可运行包
- 🌍 **跨语言**：适用于 npm、pip、cargo、go 等
- 📦 **自动安装**：首次运行时自动安装包
- 🔒 **隔离性**：每个包都安装在自己的隔离环境中
- 🎯 **版本控制**：指定确切版本以确保可重现性

## 语法

```
vx <生态系统>[@运行时版本]:<包名>[@版本][::可执行文件] [参数...]
```

### 语法组件

| 组件 | 描述 | 示例 |
|------|------|------|
| `生态系统` | 包生态系统（npm、pip、cargo、go 等） | `npm`、`pip` |
| `@运行时版本` | （可选）要使用的运行时版本 | `@20`、`@3.11` |
| `包名` | 包名称 | `typescript`、`ruff` |
| `@版本` | （可选）包版本 | `@5.3`、`@0.3` |
| `::可执行文件` | （可选）可执行文件名（如果与包名不同） | `::tsc`、`::rg` |

## 基本用法

### 运行已安装的工具

通过 `vx global install` 安装包后，可以直接运行：

```bash
# 通过可执行文件名运行已安装工具
vx tsc --version
vx black --check .
vx rg "pattern" ./src
```

### 显式包语法

当包名与可执行文件名不同时，使用完整语法：

```bash
# 包名 ≠ 可执行文件名
vx npm:typescript::tsc --version      # typescript 包，tsc 可执行文件
vx pip:httpie::http GET example.com   # httpie 包，http 命令
vx cargo:ripgrep::rg "pattern"        # ripgrep 包，rg 可执行文件
```

### 自动检测与安装

如果包尚未安装，vx 会自动下载并安装：

```bash
# 首次运行 - 自动安装 typescript
vx npm:typescript --version

# 首次运行 - 自动安装 ruff
vx pip:ruff check .

# 包会被缓存以供后续使用
```

## 支持的生态系统

| 生态系统 | 别名 | 运行时 | 描述 | 示例 |
|----------|------|--------|------|------|
| `npm` | `node`, `npx` | Node.js | npm 包 | `npm:typescript` |
| `bun` | `bunx` | Bun | Bun 包 | `bun:typescript` |
| `yarn` | - | Node.js | Yarn 包 | `yarn:typescript` |
| `pnpm` | - | Node.js | pnpm 包 | `pnpm:typescript` |
| `dlx` | - | Node.js | pnpm dlx 一次性运行（类似 npx） | `dlx:create-react-app` |
| `pip` | `python`, `pypi` | Python | pip 包 | `pip:black` |
| `uv` | - | Python（通过 uv） | uv 包 | `uv:ruff` |
| `uvx` | - | Python（通过 uvx） | uvx 一次性运行 | `uvx:ruff` |
| `pipx` | - | Python（通过 pipx） | pipx 一次性运行 | `pipx:cowsay` |
| `deno` | - | Deno | 通过 deno run 运行 npm/JSR 包 | `deno:cowsay` |
| `dotnet-tool` | `dotnet` | .NET | 通过 dotnet tool install 安装 .NET 工具 | `dotnet-tool:dotnet-script` |
| `jbang` | `java` | Java | 通过 jbang 运行 Java 工具 | `jbang:picocli` |
| `cargo` | `rust`, `crates` | Rust | Rust crate | `cargo:ripgrep` |
| `go` | `golang` | Go | Go 包 | `go:golangci-lint` |
| `gem` | `ruby`, `rubygems` | Ruby | Ruby gem | `gem:rails` |
| `choco` | `chocolatey` | Windows | Chocolatey 包 | `choco:git` |

## 常见用例

### TypeScript/Node.js

```bash
# 编译 TypeScript（如需要自动安装）
vx npm:typescript::tsc --version

# 运行 ESLint
vx npm:eslint .

# 使用指定 Node 版本创建 React 应用
vx npm@18:create-react-app my-app

# 运行作用域包（@biomejs/biome）
vx npm:@biomejs/biome::biome check .

# 运行指定版本的 TypeScript
vx npm:typescript@5.3::tsc --version
```

### Python

```bash
# 使用 black 格式化代码
vx pip:black .

# 使用 ruff 检查（指定版本）
vx pip:ruff@0.3 check .

# 运行 pytest
vx pip:pytest -v

# 使用指定 Python 版本
vx pip@3.11:black .

# 使用 uv（更快）
vx uv:ruff check .

# 使用 uvx（隔离、一次性）
vx uvx:ruff check .
vx uvx:black .

# 使用 pipx（隔离、一次性）
vx pipx:cowsay Hello World
vx pipx:httpie::http GET example.com

# HTTP 客户端
vx pip:httpie::http GET example.com
```

### Deno

```bash
# 通过 deno 运行 npm 包
vx deno:cowsay Hello

# 通过 deno 运行 JSR 包
vx deno:@std/cli --help

# 使用指定 deno 版本
vx deno@2:cowsay Hello
```

### .NET 工具

```bash
# 运行 dotnet-script
vx dotnet-tool:dotnet-script script.csx

# 运行 dotnet-format
vx dotnet-tool:dotnet-format --verify-no-changes

# 运行 dotnet-ef（Entity Framework）
vx dotnet-tool:dotnet-ef migrations list

# 使用别名
vx dotnet:dotnet-script script.csx
```

### Java（JBang）

```bash
# 运行 JBang 工具
vx jbang:picocli --help

# 使用 GAV 坐标
vx jbang:info.picocli:picocli-codegen --help

# 使用别名
vx java:picocli --help
```

### pnpm dlx

```bash
# 通过 pnpm dlx 运行（类似 npx）
vx dlx:create-react-app my-app
vx dlx:vite --version
vx dlx:@biomejs/biome::biome check .
```

### Rust

```bash
# 使用 ripgrep 搜索
vx cargo:ripgrep::rg "pattern" ./src

# 使用 fd 查找文件
vx cargo:fd-find::fd ".rs$" .

# 使用 bat 语法高亮
vx cargo:bat::bat src/main.rs
```

### Go

```bash
# 运行 linter
vx go:golangci-lint run

# 运行语言服务器
vx go:gopls version
```

## `::` 分隔符说明

许多包提供的可执行文件名与包名不同。`::` 分隔符让你可以指定确切的可执行文件：

| 包名 | 可执行文件 | 完整命令 | 简写（如已安装） |
|------|------------|----------|------------------|
| `typescript` | `tsc` | `vx npm:typescript::tsc` | `vx tsc` |
| `typescript` | `tsserver` | `vx npm:typescript::tsserver` | `vx tsserver` |
| `httpie` | `http` | `vx pip:httpie::http` | `vx http` |
| `ripgrep` | `rg` | `vx cargo:ripgrep::rg` | `vx rg` |
| `fd-find` | `fd` | `vx cargo:fd-find::fd` | `vx fd` |
| `bat` | `bat` | `vx cargo:bat::bat` | `vx bat` |

### 何时使用 `::`

**使用 `::` 的情况：**
- 包名与可执行文件名不同（如 `typescript` → `tsc`）
- 包有多个可执行文件（如 `typescript` 有 `tsc` 和 `tsserver`）
- 你想明确指定运行哪个可执行文件

**省略 `::` 的情况：**
- 包名等于可执行文件名（如 `eslint`、`ruff`）
- 安装后通过简写运行

## 版本规范

### 包版本

```bash
# 最新版本（默认）
vx npm:typescript --version

# 指定版本
vx npm:typescript@5.3 --version

# 版本范围
vx npm:typescript@^5.0 --version
```

### 运行时版本

```bash
# 使用指定 Node.js 版本
vx npm@20:typescript::tsc --version
vx npm@18:eslint .

# 使用指定 Python 版本
vx pip@3.11:black .
vx pip@3.12:ruff check .

# 使用最新运行时（默认）
vx npm:typescript --version
```

### 组合规范

```bash
# 完整规范：生态系统@运行时:包名@版本::可执行文件
vx npm@20:typescript@5.3::tsc --version
# │    │  │          │   │  │
# │    │  │          │   │  └── 可执行文件
# │    │  │          │   └───── 包版本
# │    │  │          └───────── 包名
# │    │  └──────────────────── 运行时版本
# │    └─────────────────────── 运行时
# └──────────────────────────── 生态系统
```

## 作用域 npm 包

对于带作用域的 npm 包（@组织/包）：

```bash
# Biome（JavaScript 工具链）
vx npm:@biomejs/biome::biome check .

# OpenAI Codex
vx npm:@openai/codex::codex

# TypeScript Go 实现
vx npm:@aspect-build/tsgo::tsgo check .
```

## 与现有工具对比

### vx vs npx

| 场景 | npx | vx |
|------|-----|-----|
| 基本执行 | `npx eslint` | `vx npm:eslint` 或 `vx eslint`（已安装） |
| 不同可执行文件 | `npx -p typescript tsc` | `vx npm:typescript::tsc` |
| 指定版本 | `npx eslint@8` | `vx npm:eslint@8` |
| 运行时版本 | ❌ 不支持 | `vx npm@20:eslint` |
| 其他生态系统 | ❌ 不支持 | ✅ pip、cargo、go 等 |

### vx vs uvx

| 场景 | uvx | vx |
|------|-----|-----|
| 基本执行 | `uvx ruff` | `vx pip:ruff` 或 `vx ruff`（已安装） |
| 不同可执行文件 | `uvx --from httpie http` | `vx pip:httpie::http` |
| 指定版本 | `uvx ruff@0.3` | `vx pip:ruff@0.3` |
| 运行时版本 | `uvx --python 3.11 ruff` | `vx pip@3.11:ruff` |
| 其他生态系统 | ❌ 不支持 | ✅ npm、cargo、go 等 |

## 项目级配置

对于项目，可以在 `vx.toml` 中声明常用包：

```toml
[tools.global]
typescript = "5.3"
eslint = "8"
black = "24.1"
ruff = "0.3"
```

然后直接使用：

```bash
vx sync    # 安装所有声明的全局工具

vx tsc --version    # 使用项目的 typescript 版本
vx eslint .
vx black .
```

## 环境变量

| 变量 | 描述 |
|------|------|
| `VX_AUTO_INSTALL` | 启用/禁用自动安装（默认：`true`） |
| `VX_GLOBAL_CACHE` | 覆盖全局包缓存目录 |

## 故障排除

### "找不到包"

```bash
# 确保使用正确的生态系统
vx npm:my-package      # 用于 npm 包
vx pip:my-package      # 用于 Python 包

# 检查包是否存在
vx global list
```

### "运行时未安装"

```bash
# 首先安装所需的运行时
vx install node        # 用于 npm 包
vx install python      # 用于 pip 包
vx install rust        # 用于 cargo 包
```

### 命令冲突

如果命令与运行时名称冲突：

```bash
# 使用显式语法
vx npm:node             # 运行 'node' 包，而非 node 运行时

# 或使用全局命令
vx global install npm:node
vx node                 # 现在运行的是该包
```

## 最佳实践

### 1. 固定版本以确保可重现性

```bash
# 好：指定版本
vx npm:typescript@5.3.3 --version

# 不太可预测：最新版本
vx npm:typescript --version
```

### 2. 在脚本中使用显式语法

```bash
# 在 CI/CD 或共享脚本中，保持明确
vx npm:typescript@5.3::tsc --project tsconfig.json
```

### 3. 对于频繁使用的工具，优先使用 `vx global install`

```bash
# 一次安装，多次使用
vx global install npm:typescript@5.3

# 然后使用简写
vx tsc --version
```

### 4. 使用 `vx dev` 进行项目隔离

```bash
# 进入项目环境
vx dev

# 所有工具都可用，无需前缀
tsc --version
black .
ruff check .
```

## 实现细节

### vx-shim Crate

`vx-shim` crate 实现了 RFC 0027 的解析和执行逻辑：

```rust
// 解析 RFC 0027 语法
let request = PackageRequest::parse("npm@20:typescript@5.3::tsc")?;
// request.ecosystem = "npm"
// request.package = "typescript"
// request.version = Some("5.3")
// request.executable = Some("tsc")
// request.runtime_spec = Some(RuntimeSpec { runtime: "node", version: "20" })
```

**架构：**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         vx-shim 架构                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  PackageRequest（包请求）                                        │   │
│  │  ├── parse(input: &str) -> Result<Self>                         │   │
│  │  ├── is_package_request(input: &str) -> bool                    │   │
│  │  └── executable_name() -> &str                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  ShimExecutor（Shim 执行器）                                     │   │
│  │  ├── execute_request(req, args) -> Result<ExitCode>             │   │
│  │  ├── find_package(ecosystem, package) -> Option<GlobalPackage>  │   │
│  │  └── resolve_executable(package, exe_name) -> PathBuf           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  执行流程                                                        │   │
│  │  1. 解析请求 (ecosystem:package@version::executable)             │   │
│  │  2. 检查包是否已安装在 PackageRegistry 中                        │   │
│  │  3. 如未安装：返回 PackageNotInstalled 错误                      │   │
│  │  4. 如已安装：解析可执行文件路径                                 │   │
│  │  5. 使用运行时环境执行                                           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 自动安装机制

当找不到包时，CLI 触发自动安装：

```rust
// 在 vx-cli/src/lib.rs 中
async fn execute_package_request(ctx, spec, args) {
    match executor.execute_request(&pkg_request, args).await {
        Ok(exit_code) => Ok(()),
        Err(ShimError::PackageNotInstalled { ecosystem, package }) => {
            // 自动安装包
            auto_install_package(ctx, &pkg_request).await?;
            // 重试执行
            executor.execute_request(&pkg_request, args).await
        }
    }
}
```

这提供了类似 `uvx`/`npx` 的无缝体验：
- 首次运行：自动安装并执行
- 后续运行：从缓存执行

### 支持的语法模式

| 模式 | 示例 | 描述 |
|------|------|------|
| 简单 | `npm:typescript` | 包名 = 可执行文件名 |
| 带版本 | `npm:typescript@5.3` | 指定包版本 |
| 不同可执行文件 | `npm:typescript::tsc` | 显式可执行文件名 |
| 完整语法 | `npm@20:typescript@5.3::tsc` | 运行时 + 包版本 + 可执行文件 |
| 作用域 npm | `npm:@biomejs/biome::biome` | 作用域包与可执行文件 |
| 运行时版本 | `pip@3.11:black` | 指定运行时版本 |

### 解析器实现

解析器处理带作用域的 npm 包等边界情况：

```rust
// 作用域包：@org/package@version
if part.starts_with('@') {
    // 处理 @types/node 或 @types/node@1.0
    if let Some(slash_pos) = part.find('/') {
        // 解析作用域和包名
        // 处理包名后的版本
    }
}
```

## Shell 执行语法

vx 支持在当前终端中启动带有特定运行时环境的 shell（不会打开新窗口）：

```
vx <运行时>[::shell名称]
```

### 示例

```bash
# 在当前终端打开 git-bash（Windows）
vx git::git-bash

# 打开带 node 环境的 PowerShell
vx node::powershell

# 打开带 go 环境的 cmd
vx go::cmd

# 打开带 python 环境的 bash
vx python::bash

# 打开带 uv 环境的默认 shell
vx uv::bash
```

### 支持的 Shell

| Shell | 平台 | 说明 |
|-------|------|------|
| `git-bash` | Windows | Git Bash（MINGW64），附加到当前终端 |
| `bash` | Unix/Windows | Bash shell |
| `zsh` | macOS/Linux | Z shell |
| `fish` | Unix | Fish shell |
| `powershell` | Windows/Unix | PowerShell |
| `cmd` | Windows | 命令提示符 |
| `sh` | Unix | POSIX shell |

### 行为说明

- **默认**：Shell 在当前终端中运行（不打开新窗口）
- **Windows git-bash**：直接使用 `bin/bash.exe`（与 VSCode 做法相同），而非 `git-bash.exe`（MinTTY 启动器）。使用 `--login -i` 参数以交互式登录 shell 运行。
- **所有 shell**：继承运行时的 PATH 和环境变量

## 相关命令

- [`vx global`](./global) - 管理全局包
- [`vx install`](./install) - 安装运行时版本
- [RFC 0027: 隐式包执行](../rfcs/0027-implicit-package-execution.md)
- [RFC 0025: 跨语言包隔离](../rfcs/0025-cross-language-package-isolation.md)
