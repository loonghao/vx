# Provider 版本解析和环境变量构建指南

本文档说明如何在 Provider 中实现版本解析和 REZ-like 环境变量构建。

## 概述

新的 Provider 级别版本解析系统提供了：

1. **统一版本解析**：`vx python@3.11` -> `3.11.11`（自动选择最新的补丁版本）
2. **缓存机制**：解析结果会被缓存，提高后续查询性能
3. **REZ-like 环境变量**：自动设置 `VX_<PROVIDER>_ROOT`、`VX_<PROVIDER>_VERSION` 等
4. **路径组装**：自动计算安装目录、bin 目录、可执行文件路径
5. **环境隔离**：避免 vx 管理的工具与系统安装的工具冲突

## 环境隔离设计原则

### 核心原则

1. **不干扰用户的包管理**
   - ❌ 避免设置 `PYTHONPATH`、`NODE_PATH` 等会覆盖默认模块查找路径的变量
   - ✅ 让语言的原生包管理机制正常工作（pip、npm、cargo、go 等）

2. **只设置必要的环境变量**
   - ✅ 仅设置语言运行时必需的环境变量（如 `GOROOT`、`RUSTUP_HOME`）
   - ❌ 不设置用户可能想自定义的变量（如 `CARGO_HOME`、`GOPATH`）

3. **使用隔离模式**
   - ✅ 通过 `[env.advanced]` 的 `isolate = true` 启用环境隔离
   - ✅ 通过 `inherit_system_vars` 指定需要继承的系统变量

4. **支持用户的自定义环境**
   - ✅ 用户可以在 `~/.vx/config.toml` 中覆盖或添加环境变量
   - ✅ 用户可以在项目中使用 `.vx.toml` 定义项目特定的配置

### 各语言的最佳实践

#### 默认继承的系统变量

vx 自动为所有 Provider 继承一组常用的系统环境变量，无需在 `inherit_system_vars` 中显式指定：

```rust
// DEFAULT_INHERIT_SYSTEM_VARS（不包含 PATH，PATH 有专门处理）
const DEFAULT: &[&str] = &[
    // 用户和会话
    "HOME", "USER", "USERNAME", "USERPROFILE", "LOGNAME",
    // Shell 和终端
    "SHELL", "TERM", "COLORTERM",
    // 本地化
    "LANG", "LANGUAGE", "LC_*",
    // 时区
    "TZ",
    // 临时目录
    "TMPDIR", "TEMP", "TMP",
    // 显示（GUI 应用）
    "DISPLAY", "WAYLAND_DISPLAY",
    // XDG 目录（Linux）
    "XDG_*",
];

// SYSTEM_PATH_PREFIXES - 隔离模式下保留的系统 PATH 目录
// 这些目录包含基础系统工具（sh, bash, cat 等），子进程可能需要
// 用户目录（如 ~/.local/bin）被排除以保持隔离
const SYSTEM_PATHS: &[&str] = &[
    // Unix
    "/bin", "/usr/bin", "/usr/local/bin",
    "/sbin", "/usr/sbin", "/usr/local/sbin",
    "/opt/homebrew/bin",  // macOS Homebrew (Apple Silicon)
    // Windows
    "C:\\Windows\\System32", "C:\\Windows\\SysWOW64",
    "C:\\Windows\\System32\\WindowsPowerShell",
];
```

Provider 只需添加**额外**需要的变量，如：
- **Git**: `SSH_AUTH_SOCK`, `GPG_TTY`（SSH agent 和 GPG 签名）
- **CMake**: `CC`, `CXX`, `CFLAGS`, `CXXFLAGS`, `LDFLAGS`（编译器配置）
- **Docker**: `DOCKER_HOST`, `DOCKER_CONFIG`（Docker daemon 配置）

#### Python

```toml
[env]
# 注意：避免设置 PYTHONPATH 以避免干扰模块搜索路径
# Python 会自动使用 site-packages
# 用户应该使用虚拟环境（uv venv, venv, poetry 等）进行包管理
vars = { PYTHONDONTWRITEBYTECODE = "1" }

[env.advanced]
# PATH 配置（按优先级排序）
path_prepend = ["{install_dir}/bin", "{install_dir}/Scripts"]
# 隔离模式：不从系统环境继承 PYTHON_* 变量
isolate = true
# 默认系统变量（HOME, USER, SHELL, TERM 等）已自动继承
# 无需显式指定 inherit_system_vars
```

**原因**：
- Python 会自动查找 `site-packages`，不需要 `PYTHONPATH`
- 用户应该使用虚拟环境（`uv venv`、`poetry`、`pipenv` 等）
- `PYTHONPATH` 会干扰项目依赖查找，导致错误的包被导入

#### Node.js

```toml
[env]
# 注意：不设置 NODE_PATH 以避免干扰 node_modules 解析
# Node.js 默认从当前目录向上查找 node_modules
vars = { }

[env.advanced]
path_prepend = ["{install_dir}/bin"]
# 隔离模式：不从系统环境继承 NODE_* 变量
# 默认系统变量（HOME, USER, SHELL, TERM 等）已自动继承
```

**原因**：
- Node.js 会从当前目录向上查找 `node_modules`，这是标准的模块解析机制
- `NODE_PATH` 是过时的特性，会覆盖标准的模块查找
- 用户应该使用 `pnpm`、`yarn`、`bun` 等现代包管理器

#### Go

```toml
[env]
# Go 需要明确设置 GOROOT 和 GOBIN
vars = { GOROOT = "{install_dir}", GOBIN = "{install_dir}/bin" }

[env.advanced]
path_prepend = ["{install_dir}/bin"]
# 隔离模式：不从系统环境继承 GO_* 变量
# 默认系统变量已自动继承，无需额外配置
```

**原因**：
- Go 需要知道标准库（`GOROOT`）和工具（`GOBIN`）的位置
- `GOPATH` 在 Go 1.11+ 之后不再必需（Go modules 取代了它）
- 用户可以在自己的 `$HOME/go` 或任何目录下创建 workspace

#### Rust

```toml
[env]
# Rust 工具链的环境变量
vars = { RUSTUP_HOME = "$HOME/.rustup" }

[env.advanced]
path_prepend = ["$HOME/.cargo/bin", "{install_dir}/bin"]
# 隔离模式：不从系统环境继承 RUST_* 变量
# 默认系统变量已自动继承，无需额外配置
```

**原因**：
- Rustup 需要知道自己的安装位置（`RUSTUP_HOME`）
- Cargo 会使用默认的 `$HOME/.cargo`，用户可以通过 `~/.cargo/config` 自定义
- 强制设置 `CARGO_HOME` 会干扰用户的 workspace 配置

### REZ 的环境隔离参考

REZ 包管理系统的隔离原则：

1. **每个包都有独立的安装目录**
   - `/packages/python/3.11.11`
   - `/packages/node/20.0.0`

2. **激活包时设置环境变量**
   - `REZ_PYTHON_ROOT=/packages/python/3.11.11`
   - `PATH=/packages/python/3.11.11/bin:$PATH`

3. **包之间相互隔离**
   - 不同版本的环境变量不会相互干扰
   - 用户可以同时激活多个包（如 python@3.11 和 node@20）

vx 采用了类似的设计，通过 `vx env` 命令创建隔离的环境。

## 核心组件

### 1. IntegratedVersionResolver

```rust
use vx_runtime::IntegratedVersionResolver;

// 创建解析器
let resolver = IntegratedVersionResolver::new()?;

// 解析版本并获取路径
let resolved = resolver.resolve_and_get_paths(
    "python",
    "3.11",           // 版本请求（可以是部分版本如 "3.11"）
    &Ecosystem::Python,  // 生态系统
)?;

println!("Resolved version: {}", resolved.version);
println!("Install dir: {}", resolved.install_dir.display());
println!("Bin dir: {}", resolved.bin_dir.display());
println!("Executable: {}", resolved.executable_path.display());
```

### 2. ProviderEnvBuilder

```rust
use vx_runtime::{ProviderEnvBuilder, Ecosystem};
use std::collections::HashMap;

// 创建环境构建器
let builder = ProviderEnvBuilder::new()?;

// 构建环境（支持 manifest 环境变量）
let mut manifest_vars = HashMap::new();
manifest_vars.insert("PYTHONHOME".to_string(), "{install_dir}".to_string());

let env = builder.build_for_version(
    "python",        // provider 名称
    "python",        // runtime 名称
    "3.11",          // 版本请求
    &Ecosystem::Python,
    Some(&manifest_vars),
)?;

// 获取环境变量
let env_vars = builder.build_env_vars_from_env(&env);

// REZ-like 变量：
// VX_PYTHON_ROOT=/path/to/python/3.11.11
// VX_PYTHON_VERSION=3.11.11
// VX_PYTHON_ORIGINAL_REQUEST=3.11
// PYTHONHOME=/path/to/python/3.11.11
```

### 3. 构建 PATH

```rust
use vx_runtime::ProviderEnvBuilder;

let builder = ProviderEnvBuilder::new()?;
let env = builder.build_for_version("python", "python", "3.11", &Ecosystem::Python, None)?;

// 获取需要预置到 PATH 的条目
let path_entries = builder.build_path_from_env(&env);

// 使用方式：
// PATH="{path_entry1}:{path_entry2}:...:$PATH"
```

## 在 Provider Manifest 中配置环境变量

### 环境变量配置

在 `provider.toml` 中使用 `[env]` 部分配置环境变量：

```toml
[env]
# 注意：避免设置 PYTHONPATH/NODE_PATH 等变量
# 这会干扰包管理机制（node_modules, venv, 等）
vars = { PYTHONDONTWRITEBYTECODE = "1" }

[env.advanced]
# PATH 配置（按优先级排序）
path_prepend = ["{install_dir}/bin", "{install_dir}/Scripts"]
# 隔离模式：不从系统环境继承特定变量
isolate = true
# 默认系统变量（HOME, USER, SHELL, TERM, LANG, LC_*, TZ, TMPDIR, TEMP, TMP 等）
# 已自动继承，无需显式指定
# 只需添加 provider 特定的额外变量：
inherit_system_vars = ["SSH_AUTH_SOCK", "GPG_TTY"]  # 例如 Git 需要的
```

### 支持的占位符

| 占位符 | 说明 | 示例值 |
|--------|------|--------|
| `{install_dir}` | 安装根目录 | `/home/user/.vx/store/python/3.11.11` |
| `{version}` | 完整版本号 | `3.11.11` |
| `{runtime}` | Runtime 名称 | `python` |
| `{provider}` | Provider 名称 | `python` |

### 完整示例

#### Python Provider

```toml
[provider]
name = "python"
description = "Python programming language"

[[runtimes]]
name = "python"
executable = "python"

[env]
# 注意：不设置 PYTHONPATH 以避免干扰模块搜索路径
# Python 会自动使用 site-packages
# 用户应该使用虚拟环境（uv venv, venv, poetry 等）进行包管理
vars = { PYTHONDONTWRITEBYTECODE = "1" }

[env.advanced]
# PATH 配置（按优先级排序）
path_prepend = ["{install_dir}/bin", "{install_dir}/Scripts"]
# 隔离模式：不从系统环境继承 PYTHON_* 变量
isolate = true
# 默认系统变量（HOME, USER, SHELL, TERM 等）已自动继承
# 无需显式指定 inherit_system_vars
```

**设计原则**：
- ✅ **不设置 PYTHONPATH**：让 Python 正常查找 `site-packages` 和项目依赖
- ✅ **使用虚拟环境**：用户应该通过 `uv venv`、`venv`、`poetry` 等创建隔离的 Python 环境
- ✅ **环境隔离**：通过 `isolate = true` 确保不继承系统的 PYTHON_* 变量
- ✅ **默认继承**：HOME, USER, SHELL, TERM, LANG 等系统变量自动继承

#### Node.js Provider

```toml
[provider]
name = "node"
description = "Node.js JavaScript runtime"

[[runtimes]]
name = "node"
executable = "node"

[[runtimes]]
name = "npm"
executable = "npm"

[env]
# 注意：不设置 NODE_PATH 以避免干扰 node_modules 解析
# Node.js 默认从当前目录向上查找 node_modules
# 用户可以通过 npm config 设置全局安装位置
vars = { }

[env.advanced]
path_prepend = ["{install_dir}/bin"]
# 隔离模式：不从系统环境继承 NODE_* 变量
# 默认系统变量（HOME, USER, SHELL, TERM 等）已自动继承
```

**设计原则**：
- ✅ **不设置 NODE_PATH**：让 Node.js 正常查找 `node_modules`（项目或全局）
- ✅ **使用 pnpm/yarn/bun**：这些工具有更好的 workspace 和 monorepo 支持
- ✅ **环境隔离**：不继承系统的 NODE_* 变量，避免冲突
- ✅ **默认继承**：HOME, USER, SHELL, TERM 等系统变量自动继承

#### Go Provider

```toml
[provider]
name = "go"
description = "Go programming language"

[[runtimes]]
name = "go"
executable = "go"

[env]
# Go 需要明确设置 GOROOT 和 GOBIN
vars = { GOROOT = "{install_dir}", GOBIN = "{install_dir}/bin" }

[env.advanced]
path_prepend = ["{install_dir}/bin"]
# 隔离模式：不从系统环境继承 GO_* 变量
# 默认系统变量（HOME, USER, SHELL, TERM 等）已自动继承
```

**设计原则**：
- ✅ **设置 GOROOT 和 GOBIN**：Go 需要明确知道标准库和工具的位置
- ✅ **不设置 GOPATH**：让 Go 使用默认的 `$HOME/go`，用户可以在自己的目录下安装包
- ✅ **支持 Go modules**：Go 1.11+ 使用 Go modules，不需要 GOPATH
- ✅ **环境隔离**：不继承系统的 GO_* 变量
- ✅ **默认继承**：HOME, USER, SHELL, TERM 等系统变量自动继承

#### Rust Provider

```toml
[provider]
name = "rust"
description = "A language empowering everyone to build reliable and efficient software"

[[runtimes]]
name = "rustup"
description = "The Rust toolchain installer"
executable = "rustup"

[env]
# Rust 工具链的环境变量
# 注意：不强制设置 CARGO_HOME，让用户控制 cargo 的安装位置
vars = { RUSTUP_HOME = "$HOME/.rustup" }

[env.advanced]
# PATH 配置（优先级排序）
# cargo/bin 应该在 PATH 前面，这样用户安装的工具优先
path_prepend = ["$HOME/.cargo/bin", "{install_dir}/bin"]
# 隔离模式：不从系统环境继承 RUST_* 变量
# 默认系统变量（HOME, USER, SHELL, TERM 等）已自动继承
```

**设计原则**：
- ✅ **设置 RUSTUP_HOME**：Rustup 需要知道安装位置
- ✅ **不设置 CARGO_HOME**：让 Cargo 使用默认的 `$HOME/.cargo`，用户可以通过 `~/.cargo/config` 自定义
- ✅ **支持 workspace**：用户可以在自己的目录下创建 Rust workspace
- ✅ **环境隔离**：不继承系统的 RUST_* 变量
- ✅ **默认继承**：HOME, USER, SHELL, TERM 等系统变量自动继承

## 在 env handler 中使用

### 替代当前的 parse_runtime_version

**旧代码** (`crates/vx-cli/src/commands/env/helpers.rs`):
```rust
pub fn parse_runtime_version(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.splitn(2, '@').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid format...");
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
```

**新代码** (使用 IntegratedVersionResolver):
```rust
use vx_runtime::{IntegratedVersionResolver, Ecosystem};

pub fn resolve_and_get_env(
    runtime_name: &str,
    version_request: &str,
    ecosystem: &Ecosystem,
) -> Result<HashMap<String, String>> {
    let resolver = IntegratedVersionResolver::new()?;
    let builder = ProviderEnvBuilder::new()?;

    let env = builder.build_for_version(
        runtime_name,
        runtime_name,  // 假设 provider 和 runtime 名称相同
        version_request,
        ecosystem,
        None,
    )?;

    Ok(builder.build_env_vars_from_env(&env))
}

pub fn resolve_and_get_path(runtime_name: &str, version_request: &str) -> Result<PathBuf> {
    let resolver = IntegratedVersionResolver::new()?;
    let resolved = resolver.resolve_and_get_paths(
        runtime_name,
        version_request,
        &Ecosystem::Node, // 或根据 runtime 查找 ecosystem
    )?;
    Ok(resolved.bin_dir)
}
```

### 更新 build_tools_from_env_dir

**旧代码** (`crates/vx-cli/src/commands/env/helpers.rs`):
```rust
pub fn build_tools_from_env_dir(
    env_dir: &Path,
    _path_manager: &PathManager,
) -> Result<HashMap<String, String>> {
    // ... 从 symlinks 提取版本 ...
}
```

**新代码** (直接解析版本字符串):
```rust
pub fn build_tools_from_version_strings(
    version_strings: &HashMap<String, String>,  // runtime_name -> version_request
    ecosystem_map: &HashMap<String, Ecosystem>,
) -> Result<HashMap<String, String>> {
    let mut tools = HashMap::new();
    let builder = ProviderEnvBuilder::new()?;

    for (runtime_name, version_request) in version_strings {
        let ecosystem = ecosystem_map.get(runtime_name)
            .unwrap_or(&Ecosystem::Generic);

        let env = builder.build_for_version(
            runtime_name,
            runtime_name,
            version_request,
            ecosystem,
            None,
        )?;

        // 将解析后的版本存入 tools map
        tools.insert(runtime_name.clone(), env.version_info.version);
    }

    Ok(tools)
}
```

## 环境变量说明

### `VX_<PROVIDER>_ROOT`

指向该 provider 的安装根目录。
```bash
VX_PYTHON_ROOT=/home/user/.vx/store/python/3.11.11
VX_NODE_ROOT=/home/user/.vx/store/node/20.0.0
```

### `VX_<PROVIDER>_VERSION`

完整版本号。
```bash
VX_PYTHON_VERSION=3.11.11
VX_NODE_VERSION=20.0.0
```

### `VX_<PROVIDER>_ORIGINAL_REQUEST`

用户原始的版本请求字符串。
```bash
VX_PYTHON_ORIGINAL_REQUEST=3.11
VX_NODE_ORIGINAL_REQUEST=20
```

### Provider-Specific Variables
从 provider manifest 中的 `[env]` 配置解析的环境变量。

**示例：Python Provider**
```toml
[env]
vars = { PYTHONDONTWRITEBYTECODE = "1" }

[env.advanced]
path_prepend = ["{install_dir}/bin"]
```
会扩展为：
```bash
VX_PYTHON_ROOT=/home/user/.vx/store/python/3.11.11
VX_PYTHON_VERSION=3.11.11
VX_PYTHON_ORIGINAL_REQUEST=3.11
PYTHONDONTWRITEBYTECODE=1
PATH=/home/user/.vx/store/python/3.11.11/bin:$PATH
```

**注意**：
- 我们不设置 `PYTHONPATH`，避免干扰模块查找
- 用户应该使用虚拟环境进行包管理
- `VX_PYTHON_*` 变量由 vx 自动生成，用于环境隔离

## 示例使用场景

### 场景 1: vx env add python@3.11

```rust
// 在 add_runtime 函数中
async fn add_runtime(runtime_version: &str, env_name: Option<&str>, global: bool) -> Result<()> {
    let (runtime, version) = parse_runtime_version(runtime_version)?;

    // 使用新的解析器
    let resolver = IntegratedVersionResolver::new()?;
    let resolved = resolver.resolve_and_get_paths(&runtime, &version, &get_ecosystem(&runtime))?;

    // 创建链接到 store 中的已解析版本
    let env_runtime_path = env_dir.join(&runtime);
    link::create_link(&resolved.install_dir, &env_runtime_path, LinkStrategy::SymLink)?;
}
```

### 场景 2: vx env shell (激活环境)

```rust
async fn env_shell(name: Option<&str>, global: bool, ...) -> Result<()> {
    let (env_dir, env_name) = resolve_env_for_shell(name, global, &path_manager)?;
    let tools = build_tools_from_env_dir(&env_dir, &path_manager)?;

    // 构建环境变量
    let builder = ProviderEnvBuilder::new()?;
    let mut all_env_vars = HashMap::new();

    for (runtime_name, version_request) in &tools {
        let ecosystem = get_ecosystem(runtime_name);
        let env = builder.build_for_version(runtime_name, runtime_name, version_request, &ecosystem, None)?;
        let env_vars = builder.build_env_vars_from_env(&env);
        all_env_vars.extend(env_vars);
    }

    // 创建 SessionContext
    let session = SessionContext::new(&env_name).env_vars(&all_env_vars);

    // ... 继续启动 shell
}
```

## 迁移检查清单

- [ ] 更新 `parse_runtime_version` 使用 `IntegratedVersionResolver`
- [ ] 更新 `build_tools_from_env_dir` 使用版本解析器
- [ ] 更新 `env_shell` 构建 REZ-like 环境变量
- [ ] 在 `SessionContext` 中使用新的环境变量
- [ ] 更新所有 provider.toml 添加 `[env]` 配置
- [ ] 更新测试用例
- [ ] 更新文档

## 优势

1. **统一版本解析**：所有版本解析逻辑集中在 Provider 层，避免重复
2. **性能优化**：缓存机制减少重复计算和网络请求
3. **REZ 兼容**：环境变量命名遵循 REZ 规范，便于迁移
4. **类型安全**：使用 Rust 类型系统确保路径和版本的正确性
5. **可测试性**：核心逻辑与文件系统和网络解耦，易于单元测试
6. **可扩展性**：Provider 可以自定义环境变量和 PATH 配置

## vx dev 集成

### 概述

`Runtime::prepare_environment` 方法现在已集成到 `vx dev` 和 `vx env --export` 命令中。这确保了运行时特定的环境变量（如 MSVC 的 INCLUDE 和 LIB）在进入开发环境时被自动配置。

### 工作原理

1. 运行 `vx dev` 时，命令会调用 `build_dev_environment()`
2. 对于 `vx.toml` 中的每个工具，`build_dev_environment()` 会：
   - 从 provider registry 获取 runtime
   - 调用 `runtime.prepare_environment(version, context).await`
   - 将返回的环境变量合并到 dev 环境
   - 使用正确的 bin 目录创建 ToolSpec
3. 合并的环境变量（PATH + 运行时特定变量）用于生成 dev shell

### 示例：dev 环境中的 MSVC

在 `vx.toml` 中配置 MSVC：

```toml
[tools]
msvc = "14.40"
cmake = "latest"
```

运行 `vx dev` 时：

1. MSVC 的 `prepare_environment()` 返回：
   - `INCLUDE`：MSVC 头文件路径
   - `LIB`：MSVC 库文件路径
   - `PATH`：MSVC 二进制文件路径

2. 这些变量被合并到 dev 环境中
3. CMake 现在可以自动找到 MSVC 的头文件和库，无需手动配置

### 运行时特定环境

以下运行时通过 `prepare_environment()` 提供特殊环境变量：

| 运行时 | 环境变量 | 用途 |
|---------|----------------------|---------|
| **MSVC** | `INCLUDE`、`LIB`、`PATH` | 使用 cl.exe 的 C/C++ 编译 |
| **Python** | `PYTHONDONTWRITEBYTECODE`、`PYTHONPATH`（如果配置） | Python 开发 |
| **Node.js** | （无，由 PATH 处理） | JavaScript/TypeScript 开发 |
| **Go** | `GOROOT`、`GOBIN`（如果配置） | Go 开发 |
| **Rust** | `RUSTUP_HOME`（如果配置） | Rust 开发 |

### 实现细节

#### handler.rs

在 `build_dev_environment` 函数中：
```rust
// 调用 prepare_environment 并合并环境变量
if let Ok(runtime_env) = runtime.prepare_environment(version, &context).await {
    for (key, value) in runtime_env {
        env_vars.insert(key, value);
    }
}
```

#### export.rs

在 `generate_env_export` 函数中：
```rust
// 调用 prepare_environment 并合并环境变量
if let Ok(runtime_env) = runtime.prepare_environment(version, &context).await {
    for (key, value) in runtime_env {
        env_vars.insert(key, value);
    }
}
```

### 注意事项

1. **异步函数**：`prepare_environment` 是异步的，因此 `build_dev_environment` 和 `generate_env_export` 必须也是异步的
2. **环境优先级**：运行时环境变量优先级高于 `vx.toml` 中的 `env` 配置
3. **性能考虑**：`prepare_environment` 的调用是串行的，但开销很小（通常只返回少量环境变量）

## 相关文档

- [Provider 开发指南](../guide/manifest-driven-providers.md)
- [REZ 包管理系统](https://github.com/nerdvegas/rez)
- [环境变量最佳实践](../guide/best-practices.md)
