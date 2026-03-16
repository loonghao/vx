# provider.star — 语言与标准库参考

本文档是 provider.star DSL 的**权威参考**，用于定义 vx Provider。涵盖 Starlark 语言子集、执行模型、上下文对象、所有 stdlib 模块、Provider 模板和最佳实践。

> **相关文档**
>
> - [声明式 Provider](./manifest-driven-providers.md) — 入门教程
> - [Starlark Provider — 高级指南](./starlark-providers.md) — 多运行时、钩子、系统集成

---

## 目录

- [1. 执行模型](#1-执行模型)
- [2. 文件结构](#2-文件结构)
- [3. 顶层变量](#3-顶层变量)
- [4. Provider 函数](#4-provider-函数)
- [5. 上下文对象（`ctx`）](#5-上下文对象ctx)
- [6. 标准库模块](#6-标准库模块)
  - [6.1 provider.star — 统一入口](#61-providerstar--统一入口)
  - [6.2 runtime.star — 运行时定义](#62-runtimestar--运行时定义)
  - [6.3 env.star — 环境变量](#63-envstar--环境变量)
  - [6.4 platform.star — 平台检测](#64-platformstar--平台检测)
  - [6.5 http.star — HTTP 描述符](#65-httpstar--http-描述符)
  - [6.6 github.star — GitHub 辅助函数](#66-githubstar--github-辅助函数)
  - [6.7 install.star — 安装描述符](#67-installstar--安装描述符)
  - [6.8 layout.star — 布局、钩子与路径工厂](#68-layoutstar--布局钩子与路径工厂)
  - [6.9 permissions.star — 权限声明](#69-permissionsstar--权限声明)
  - [6.10 system_install.star — 包管理器策略](#610-system_installstar--包管理器策略)
  - [6.11 script_install.star — 脚本安装](#611-script_installstar--脚本安装)
  - [6.12 semver.star — 版本比较](#612-semverstar--版本比较)
  - [6.13 test.star — 测试 DSL](#613-teststar--测试-dsl)
  - [6.14 provider_templates.star — 高级模板](#614-provider_templatesstar--高级模板)
- [7. 安装布局类型](#7-安装布局类型)
- [8. 版本获取策略](#8-版本获取策略)
- [9. 钩子](#9-钩子)
- [10. Starlark 语言子集](#10-starlark-语言子集)
- [11. 编码规范](#11-编码规范)
- [12. 清单：创建新 Provider](#12-清单创建新-provider)

---

## 1. 执行模型

vx 使用**两阶段执行模型**（借鉴 Buck2）：

```
┌──────────────────────────────────┐     ┌──────────────────────────────────┐
│  第一阶段 — 分析（Starlark）      │     │  第二阶段 — 执行（Rust）          │
│                                  │     │                                  │
│  provider.star 运行并生成        │────▶│  Rust 解释描述符并执行真实 I/O：  │
│  描述符 dict（纯计算，无 I/O，   │     │  HTTP、文件系统、进程            │
│  无网络访问）                    │     │                                  │
└──────────────────────────────────┘     └──────────────────────────────────┘
```

关键原则：

| 原则 | 说明 |
|------|------|
| **无副作用** | Starlark 函数返回描述符 dict，永远不直接调用网络或文件系统 |
| **确定性** | 给定相同的 `ctx`，函数始终返回相同的结果 |
| **JSON 往返** | 所有值在 Starlark 和 Rust 之间通过 JSON 序列化传递 |
| **`None` = 不支持** | `download_url()` 返回 `None` 表示"在此平台上不可用" |

---

## 2. 文件结构

每个 Provider 位于单独的目录中：

```
crates/vx-providers/<name>/
├── provider.star     # 所有逻辑（必需）
├── provider.toml     # 可选的元数据补充
└── README.md         # 可选文档
```

用户自定义 Provider 遵循相同结构，位于 `~/.vx/providers/<name>/`。

---

## 3. 顶层变量

声明为模块级赋值，**不是**函数。

| 变量 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `name` | `string` | **是** | Provider 名称（必须匹配目录名） |
| `description` | `string` | **是** | 可读描述 |
| `runtimes` | `list[dict]` | **是** | 运行时定义（见 [§6.2](#62-runtimestar--运行时定义)） |
| `permissions` | `dict` | 否 | 权限声明（见 [§6.9](#69-permissionsstar--权限声明)） |
| `homepage` | `string` | 否 | 项目主页 URL |
| `repository` | `string` | 否 | 源码仓库 URL |
| `license` | `string` | 否 | SPDX 许可标识（如 `"MIT"`、`"Apache-2.0"`） |
| `ecosystem` | `string` | 否 | 分类：`nodejs`、`python`、`rust`、`go`、`devtools`、`system`、`custom` 等 |
| `package_alias` | `dict` | 否 | 路由到生态包运行器（如 `{"ecosystem": "uvx", "package": "ruff"}`） |
| `package_prefixes` | `list[string]` | 否 | 包执行前缀（如 `["bun", "bunx"]`） |
| `vx_version` | `string` | 否 | 最低 vx 版本要求（如 `">=0.7.0"`） |

```python
# 示例
name        = "ripgrep"
description = "ripgrep — 递归搜索目录中的正则模式"
homepage    = "https://github.com/BurntSushi/ripgrep"
repository  = "https://github.com/BurntSushi/ripgrep"
license     = "MIT OR Unlicense"
ecosystem   = "devtools"
```

---

## 4. Provider 函数

这些是模块级函数，由 Rust 运行时调用。

| 函数 | 签名 | 必需 | 返回 |
|------|------|------|------|
| `fetch_versions` | `(ctx) → descriptor` | **是** | 版本列表或获取描述符 |
| `download_url` | `(ctx, version) → string \| None` | **是** | 下载 URL，不支持时返回 `None` |
| `install_layout` | `(ctx, version) → dict \| None` | **是** | 安装布局描述符 |
| `environment` | `(ctx, version) → list[EnvOp]` | **是** | 环境变量操作列表 |
| `store_root` | `(ctx) → string` | 否 | Store 根目录路径 |
| `get_execute_path` | `(ctx, version) → string` | 否 | 可执行文件完整路径 |
| `post_install` | `(ctx, version) → dict \| None` | 否 | 安装后操作 |
| `post_extract` | `(ctx, version, install_dir) → list` | 否 | 解压后钩子操作 |
| `pre_run` | `(ctx, args, executable) → list` | 否 | 运行前钩子操作 |
| `deps` | `(ctx, version) → list[DepDef]` | 否 | 运行时依赖声明 |
| `system_install` | `(ctx) → dict` | 否 | 系统包管理器策略 |
| `script_install` | `(ctx) → dict` | 否 | 脚本安装命令 |
| `uninstall` | `(ctx, version) → bool` | 否 | 自定义卸载逻辑 |

### 最小 Provider 骨架

```python
load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "github_rust_provider")

name        = "mytool"
description = "我的工具"
ecosystem   = "devtools"

runtimes    = [runtime_def("mytool")]
permissions = github_permissions()

_p = github_rust_provider("owner", "repo",
    asset = "mytool-{vversion}-{triple}.{ext}")

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

---

## 5. 上下文对象（`ctx`）

`ctx` 对象由 Rust 运行时注入，使用 Starlark `struct` 语法（点号访问）。

| 字段 | 类型 | 说明 |
|------|------|------|
| `ctx.name` | `string` | Provider 名称 |
| `ctx.description` | `string` | Provider 描述 |
| `ctx.version` | `string` | 正在处理的版本 |
| `ctx.runtime_name` | `string` | 运行时名称（多运行时 Provider 使用） |
| `ctx.version_date` | `string` | 版本的构建标签或日期 |
| `ctx.vx_home` | `string` | vx 主目录（`~/.vx`） |
| `ctx.install_dir` | `string` | 版本专属安装目录 |
| `ctx.platform.os` | `string` | `"windows"` \| `"macos"` \| `"linux"` |
| `ctx.platform.arch` | `string` | `"x64"` \| `"arm64"` \| `"x86"` |
| `ctx.platform.target` | `string` | Rust 目标三元组（如 `"x86_64-pc-windows-msvc"`） |
| `ctx.env` | `dict` | 当前环境变量 |
| `ctx.paths.install_dir` | `string` | 同 `ctx.install_dir` |
| `ctx.paths.vx_home` | `string` | 同 `ctx.vx_home` |
| `ctx.paths.store_dir` | `string` | 全局 Store 目录 |
| `ctx.paths.cache_dir` | `string` | 缓存目录 |
| `ctx.paths.download_cache` | `string` | 下载缓存目录 |

### 使用

```python
def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    # ...
```

---

## 6. 标准库模块

所有模块通过以下方式加载：

```python
load("@vx//stdlib:<module>.star", "function1", "function2")
```

### 6.1 `provider.star` — 统一入口

`provider.star` 模块是一个**重导出外观**，从各子模块聚合所有公共 API。你可以从这里导入一切：

```python
load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "dep_def",
     "github_permissions", "system_permissions",
     "env_set", "env_prepend", "env_append", "env_unset",
     "platform_map", "platform_select", "rust_triple",
     "archive_layout", "binary_layout", "bin_subdir_layout",
     "bin_subdir_env", "bin_subdir_execute_path", "path_fns",
     "post_extract_flatten", "post_extract_shim",
     "post_extract_permissions", "post_extract_combine",
     "pre_run_ensure_deps",
     "fetch_versions_from_api", "fetch_versions_with_tag_prefix",
     "winget_install", "brew_install", "apt_install",
     "cross_platform_install",
     "github_rust_provider", "github_go_provider",
     "github_binary_provider", "system_provider")
```

或从特定子模块精确导入以减少命名空间污染。

---

### 6.2 `runtime.star` — 运行时定义

#### `runtime_def(name, **kwargs) → dict`

定义独立运行时。

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `name` | `string` | — | 运行时名称（必需） |
| `executable` | `string` | `name` | 可执行文件名 |
| `description` | `string` | `""` | 可读描述 |
| `aliases` | `list[string]` | `[]` | 替代名称 |
| `priority` | `int` | `100` | 解析优先级（越高越优先） |
| `version_cmd` | `string` | `None` | 自定义版本命令 |
| `version_pattern` | `string` | `None` | 匹配版本输出的正则 |
| `test_commands` | `list[dict]` | `[]` | 验证命令 |
| `auto_installable` | `bool` | `True` | 是否可自动安装 |
| `platform_constraint` | `dict` | `None` | 平台限制 |
| `system_paths` | `list[string]` | `[]` | 已知系统安装路径 |
| `bundled_with` | `string` | `None` | （请使用 `bundled_runtime_def` 代替） |

```python
runtimes = [
    runtime_def("node",
        aliases         = ["nodejs"],
        version_pattern = "v\\d+\\.\\d+\\.\\d+",
        test_commands   = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "v\\d+\\.\\d+"},
        ],
    ),
]
```

#### `bundled_runtime_def(name, bundled_with, **kwargs) → dict`

定义与其他运行时捆绑发布的运行时（如 `npm` 随 `node` 发布）。

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `name` | `string` | — | 运行时名称 |
| `bundled_with` | `string` | — | 父运行时名称 |
| `executable` | `string` | `name` | 可执行文件名 |
| `description` | `string` | `""` | 描述 |
| `aliases` | `list[string]` | `[]` | 替代名称 |
| `command_prefix` | `list[string]` | `None` | 调用时前置的参数（如 `["x"]` 使 `bunx foo` → `bun x foo`） |
| `test_commands` | `list[dict]` | `[]` | 验证命令 |
| `version_pattern` | `string` | `None` | 版本输出正则 |
| `auto_installable` | `bool` | `True` | 自动安装能力 |
| `platform_constraint` | `dict` | `None` | 平台限制 |

```python
runtimes = [
    runtime_def("node"),
    bundled_runtime_def("npm", "node",
        description = "Node 包管理器"),
    bundled_runtime_def("npx", "node",
        description = "Node 包执行器"),
]
```

#### `dep_def(runtime, **kwargs) → dict`

声明运行时依赖。

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `runtime` | `string` | — | 所需运行时名称 |
| `version` | `string` | `"*"` | 版本约束（如 `">=18"`） |
| `optional` | `bool` | `False` | 是否可选 |
| `reason` | `string` | `None` | 可读的原因说明 |

```python
def deps(_ctx, _version):
    return [
        dep_def("git", optional=True,
                reason="Git 用于获取模块"),
    ]
```

---

### 6.3 `env.star` — 环境变量

| 函数 | 签名 | 说明 |
|------|------|------|
| `env_set(key, value)` | `→ dict` | 设置环境变量 |
| `env_prepend(key, value, sep=None)` | `→ dict` | 前置到 PATH 类变量 |
| `env_append(key, value, sep=None)` | `→ dict` | 追加到 PATH 类变量 |
| `env_unset(key)` | `→ dict` | 移除环境变量 |

**返回格式：**

```python
env_set("GOROOT", "/path")
# → {"op": "set", "key": "GOROOT", "value": "/path"}

env_prepend("PATH", "/usr/local/go/bin")
# → {"op": "prepend", "key": "PATH", "value": "/usr/local/go/bin"}
```

**在 `environment()` 中使用：**

```python
def environment(ctx, _version):
    return [
        env_set("GOROOT", ctx.install_dir),
        env_prepend("PATH", ctx.install_dir + "/bin"),
        env_set("GO111MODULE", "on"),
    ]
```

---

### 6.4 `platform.star` — 平台检测

#### 布尔检查

| 函数 | 说明 |
|------|------|
| `is_windows(ctx)` | Windows 上返回 `True` |
| `is_macos(ctx)` | macOS 上返回 `True` |
| `is_linux(ctx)` | Linux 上返回 `True` |
| `is_x64(ctx)` | x64/amd64 上返回 `True` |
| `is_arm64(ctx)` | arm64/aarch64 上返回 `True` |

#### 三元组与架构

| 函数 | 签名 | 说明 |
|------|------|------|
| `platform_triple(ctx)` | `→ string` | 返回 `ctx.platform.target` |
| `rust_triple(ctx, linux_libc="musl")` | `→ string \| None` | 完整 Rust 目标三元组 |
| `go_os_arch(ctx)` | `→ (string, string)` | Go 风格 `(os, arch)` 元组 |
| `arch_to_gnu(arch)` | `→ string` | `"x64"` → `"x86_64"`，`"arm64"` → `"aarch64"` |
| `arch_to_go(arch)` | `→ string` | `"x64"` → `"amd64"`，`"arm64"` → `"arm64"` |
| `os_to_go(os)` | `→ string` | `"macos"` → `"darwin"` |

#### 扩展名辅助

| 函数 | 签名 | 说明 |
|------|------|------|
| `platform_ext(ctx)` | `→ string` | Windows 上 `".zip"`，其他 `".tar.gz"` |
| `archive_ext(ctx)` | `→ string` | Windows 上 `"zip"`，其他 `"tar.gz"`（无点号） |
| `exe_ext(ctx)` | `→ string` | Windows 上 `".exe"`，其他 `""` |
| `exe_suffix(ctx)` | `→ string` | 同 `exe_ext()` |

#### 平台分发

| 函数 | 签名 | 说明 |
|------|------|------|
| `platform_map(ctx, mapping, fallback=None)` | `→ any` | 按 `"{os}/{arch}"` 键查找映射 dict |
| `platform_select(ctx, windows, macos, linux, fallback=None)` | `→ any` | 按 OS 选择值 |

```python
# platform_map — 按 OS/arch 组合分发
_PLATFORMS = {
    "windows/x64":  "x86_64-pc-windows-msvc",
    "macos/arm64":  "aarch64-apple-darwin",
    "linux/x64":    "x86_64-unknown-linux-musl",
}
triple = platform_map(ctx, _PLATFORMS)  # 不支持时返回 None

# platform_select — 仅按 OS 分发
bin_dir = platform_select(ctx,
    windows = ctx.install_dir,
    macos   = ctx.install_dir + "/bin",
    linux   = ctx.install_dir + "/bin",
)
```

#### 资源模板展开

| 函数 | 签名 | 说明 |
|------|------|------|
| `expand_asset(template, ctx, version, ...)` | `→ string` | 替换 `{version}`、`{vversion}`、`{triple}`、`{os}`、`{arch}`、`{ext}`、`{exe}` |

```python
url = expand_asset(
    "mytool-{vversion}-{triple}.{ext}",
    ctx, "1.0.0",
)
# → "mytool-v1.0.0-x86_64-unknown-linux-musl.tar.gz"
```

#### 常量

| 常量 | 说明 |
|------|------|
| `RUST_TRIPLES_MUSL` | `"{os}/{arch}"` → musl 链接的 Rust 三元组映射 |
| `RUST_TRIPLES_GNU` | `"{os}/{arch}"` → GNU 链接的 Rust 三元组映射 |

---

### 6.5 `http.star` — HTTP 描述符

> **重要：** 这些函数返回**描述符 dict**，不是实际的 HTTP 响应。Rust 运行时负责解释和执行。

| 函数 | 签名 | 说明 |
|------|------|------|
| `github_releases(ctx, owner, repo, include_prereleases=False)` | `→ descriptor` | GitHub releases 描述符 |
| `github_latest_release(ctx, owner, repo)` | `→ descriptor` | 最新 release 描述符 |
| `github_download_url(owner, repo, tag, asset_name)` | `→ string` | 构建 GitHub asset 下载 URL |
| `parse_github_tag(tag)` | `→ string` | 剥离 tag 的 `v`/`release-`/`version-` 前缀 |
| `fetch_json(ctx, url)` | `→ descriptor` | 通用 JSON 获取描述符 |
| `fetch_json_versions(ctx, url, transform, headers={})` | `→ descriptor` | 带变换策略的版本获取 |
| `releases_to_versions(releases, tag_key="tag_name")` | `→ list \| descriptor` | 将 releases 数组转换为版本信息 |

#### `fetch_json_versions` 支持的变换策略

| 策略 | API 来源 |
|------|----------|
| `"nodejs_org"` | `https://nodejs.org/dist/index.json` |
| `"go_versions"` | `https://go.dev/dl/?mode=json&include=all` |
| `"adoptium"` | Eclipse Adoptium API |
| `"pypi"` | PyPI JSON API |
| `"npm_registry"` | npm 注册表 |
| `"hashicorp_releases"` | HashiCorp releases API |
| `"github_tags"` | GitHub tags API |

```python
# Node.js — 官方 API
fetch_versions = fetch_versions_from_api(
    "https://nodejs.org/dist/index.json",
    "nodejs_org",
)

# Go — 官方 API
fetch_versions = fetch_versions_from_api(
    "https://go.dev/dl/?mode=json&include=all",
    "go_versions",
)
```

---

### 6.6 `github.star` — GitHub 辅助函数

| 函数 | 签名 | 说明 |
|------|------|------|
| `github_asset_url(owner, repo, tag, asset_name)` | `→ string` | 构建 asset 下载 URL |
| `make_fetch_versions(owner, repo, include_prereleases=False)` | `→ function` | 返回绑定的 `fetch_versions(ctx)` |
| `make_download_url(owner, repo, asset_template)` | `→ function` | 返回绑定的 `download_url(ctx, version)` |
| `make_github_provider(owner, repo, asset_template=None, include_prereleases=False)` | `→ dict` | 完整 Provider 命名空间 |

```python
# 简单模式 — 将 fetch_versions 绑定到仓库
fetch_versions = make_fetch_versions("BurntSushi", "ripgrep")

# Asset URL 构建
url = github_asset_url("BurntSushi", "ripgrep", "14.1.1",
                       "ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz")
# → "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-..."
```

**`make_download_url` 模板占位符：**

| 占位符 | 展开 |
|--------|------|
| `{version}` | 版本字符串（如 `1.0.0`） |
| `{vversion}` | 带 `v` 前缀的版本（如 `v1.0.0`） |
| `{triple}` | Rust 目标三元组 |
| `{os}` | Go 风格 OS（`linux`、`darwin`、`windows`） |
| `{arch}` | Go 风格架构（`amd64`、`arm64`） |
| `{ext}` | 归档扩展名（Windows 上 `zip`，其他 `tar.gz`） |
| `{exe}` | 可执行文件后缀（Windows 上 `.exe`，其他为空） |

---

### 6.7 `install.star` — 安装描述符

| 函数 | 签名 | 说明 |
|------|------|------|
| `archive_install(url, strip_prefix, executable_paths)` | `→ descriptor` | 归档（tar.gz/zip）安装 |
| `binary_install(url, executable_name, permissions="755")` | `→ descriptor` | 单二进制下载 |
| `msi_install(url, executable_paths, strip_prefix, extra_args)` | `→ descriptor` | MSI 安装（Windows） |
| `platform_install(ctx, windows_url, macos_url, linux_url, ...)` | `→ descriptor` | 按平台选择 URL |
| `system_find(executable, system_paths, hint)` | `→ descriptor` | 查找系统已安装的工具 |
| `create_shim(name, target_executable, args, shim_dir)` | `→ descriptor` | 创建 shim 脚本 |
| `set_permissions(path, mode="755")` | `→ descriptor` | 设置文件权限 |
| `ensure_dependencies(package_manager, check_file, lock_file, install_dir)` | `→ descriptor` | 确保包依赖 |
| `run_command(executable, args, working_dir, env, on_failure="warn")` | `→ descriptor` | 运行任意命令 |
| `flatten_dir(pattern, keep_subdirs)` | `→ descriptor` | 展平目录结构 |

---

### 6.8 `layout.star` — 布局、钩子与路径工厂

#### 布局构建器

这些返回**函数**（而非 dict），可直接赋值给 `install_layout`。

| 函数 | 签名 | 说明 |
|------|------|------|
| `archive_layout(executable, strip_prefix=None)` | `→ fn(ctx, version) → dict` | 归档安装布局 |
| `binary_layout(executable)` | `→ fn(ctx, version) → dict` | 单二进制布局 |
| `bin_subdir_layout(executables, strip_prefix=None)` | `→ fn(ctx, version) → dict` | `bin/` 子目录布局 |

```python
# 归档 — 扁平结构
install_layout = archive_layout("mytool")

# 归档 — 带版本目录剥离
install_layout = archive_layout("mytool",
    strip_prefix="mytool-{vversion}-{triple}")

# 二进制 — 直接下载
install_layout = binary_layout("kubectl")

# bin/ 子目录（Node.js、Go、Java 模式）
install_layout = bin_subdir_layout(
    ["node", "npm", "npx"],
    strip_prefix="node-v{version}-{os}-{arch}")
```

#### 解压后钩子构建器

| 函数 | 签名 | 说明 |
|------|------|------|
| `post_extract_flatten(pattern=None)` | `→ fn(ctx, ver, dir) → list` | 展平顶层版本目录 |
| `post_extract_shim(shim_name, target_executable, args=None)` | `→ fn(ctx, ver, dir) → list` | 创建 shim 脚本 |
| `post_extract_permissions(paths, mode="755", unix_only=True)` | `→ fn(ctx, ver, dir) → list` | 设置可执行权限 |
| `post_extract_combine(hooks)` | `→ fn(ctx, ver, dir) → list` | 组合多个钩子 |

```python
# 在 Unix 上设置权限
post_extract = post_extract_permissions(["bin/node", "bin/npm", "bin/npx"])

# 创建 shim：`bunx foo` → `bun x foo`
post_extract = post_extract_shim("bunx", "bun", args=["x"])

# 组合多个钩子
post_extract = post_extract_combine([
    post_extract_flatten(pattern="jdk-*"),
    post_extract_permissions(["bin/java"]),
])
```

#### 运行前钩子构建器

| 函数 | 签名 | 说明 |
|------|------|------|
| `pre_run_ensure_deps(package_manager, trigger_args, check_file, lock_file=None, install_dir=None)` | `→ fn(ctx, args, exe) → list` | 运行前自动安装项目依赖 |

```python
# 执行 `npm run` 前确保 node_modules 存在
pre_run = pre_run_ensure_deps("npm",
    trigger_args = ["run", "run-script"],
    check_file   = "package.json",
    install_dir  = "node_modules",
)

# 执行 `uv run` 前确保 .venv 存在
pre_run = pre_run_ensure_deps("uv",
    trigger_args = ["run"],
    check_file   = "pyproject.toml",
    install_dir  = ".venv",
)
```

#### 路径与环境工厂

| 函数 | 签名 | 说明 |
|------|------|------|
| `path_fns(store_name, executable=None)` | `→ dict` | 返回 `{"store_root": fn, "get_execute_path": fn}` |
| `path_env_fns(extra_env=None)` | `→ dict` | 返回 `{"environment": fn, "post_install": fn}` |
| `bin_subdir_env(extra_env=None)` | `→ fn(ctx, version) → list` | 自动检测 `bin/` 与根目录的 PATH |
| `bin_subdir_execute_path(executable)` | `→ fn(ctx, version) → string` | `bin/` 子目录中的可执行文件路径 |

```python
# 快速设置 store_root + get_execute_path
paths            = path_fns("node")
store_root       = paths["store_root"]
get_execute_path = bin_subdir_execute_path("node")
environment      = bin_subdir_env()
```

#### 版本获取辅助

| 函数 | 签名 | 说明 |
|------|------|------|
| `fetch_versions_from_api(url, transform)` | `→ fn(ctx) → descriptor` | 非 GitHub 版本 API |
| `fetch_versions_with_tag_prefix(owner, repo, tag_prefix, prereleases=False)` | `→ fn(ctx) → descriptor` | 非标准 GitHub tag 前缀 |

```python
# Bun 使用 "bun-v" tag 前缀
fetch_versions = fetch_versions_with_tag_prefix(
    "oven-sh", "bun", tag_prefix="bun-v")

# Node.js 官方 API
fetch_versions = fetch_versions_from_api(
    "https://nodejs.org/dist/index.json", "nodejs_org")
```

---

### 6.9 `permissions.star` — 权限声明

| 函数 | 签名 | 说明 |
|------|------|------|
| `github_permissions(extra_hosts=None, exec_cmds=None)` | `→ dict` | 声明 GitHub API + 下载权限 |
| `system_permissions(exec_cmds=None, extra_hosts=None)` | `→ dict` | 无网络下载，仅系统包管理器 |

```python
# 标准 GitHub 工具
permissions = github_permissions()

# GitHub + 额外 API 主机
permissions = github_permissions(extra_hosts=["nodejs.org", "go.dev"])

# 仅系统安装（无二进制下载）
permissions = system_permissions()
```

---

### 6.10 `system_install.star` — 包管理器策略

#### 单策略构建器

| 函数 | 签名 | 说明 |
|------|------|------|
| `winget_install(package, priority=90, install_args=None)` | `→ dict` | winget（Windows） |
| `choco_install(package, priority=80, install_args=None)` | `→ dict` | Chocolatey（Windows） |
| `scoop_install(package, priority=70)` | `→ dict` | Scoop（Windows） |
| `brew_install(package, priority=90)` | `→ dict` | Homebrew（macOS/Linux） |
| `apt_install(package, priority=80)` | `→ dict` | APT（Debian/Ubuntu） |
| `dnf_install(package, priority=75)` | `→ dict` | DNF（Fedora/RHEL） |
| `pacman_install(package, priority=70)` | `→ dict` | pacman（Arch Linux） |
| `snap_install(package, priority=60, classic=False)` | `→ dict` | Snap（Linux） |

#### 多策略构建器

| 函数 | 签名 | 说明 |
|------|------|------|
| `pkg_strategy(manager, package, priority, install_args, platforms)` | `→ dict` | 通用策略 |
| `system_install_strategies(strategies)` | `→ dict` | 包装策略列表 |
| `cross_platform_install(windows, macos, linux, ...)` | `→ fn(ctx) → dict` | 按 OS 分发安装 |
| `windows_install(winget, choco, scoop, ...)` | `→ fn(ctx) → dict` | Windows 专用 |
| `multi_platform_install(windows_strategies, macos_strategies, linux_strategies)` | `→ fn(ctx) → dict` | 完全控制 |

```python
# 简单跨平台
system_install = cross_platform_install(
    windows = winget_install("7zip.7zip"),
    macos   = brew_install("sevenzip"),
    linux   = apt_install("p7zip-full"),
)

# 每个平台多个策略
system_install = multi_platform_install(
    windows_strategies = [
        winget_install("7zip.7zip", priority=90),
        choco_install("7zip", priority=80),
    ],
    macos_strategies = [brew_install("sevenzip")],
    linux_strategies = [
        apt_install("p7zip-full"),
        brew_install("sevenzip", priority=70),
    ],
)
```

---

### 6.11 `script_install.star` — 脚本安装

| 函数 | 签名 | 说明 |
|------|------|------|
| `curl_bash_install(url, post_install_cmds=None)` | `→ fn(ctx) → dict` | `curl \| bash`（Unix） |
| `curl_sh_install(url, post_install_cmds=None)` | `→ fn(ctx) → dict` | `curl \| sh`（POSIX） |
| `irm_iex_install(url, env_vars=None, pre_commands=None, post_install_cmds=None)` | `→ fn(ctx) → dict` | PowerShell `iex(irm ...)`（Windows） |
| `irm_install(url, env_vars=None, post_install_cmds=None)` | `→ fn(ctx) → dict` | 现代 PowerShell `irm`（Windows） |
| `platform_script_install(unix=None, windows=None)` | `→ fn(ctx) → dict` | 按 OS 分发脚本安装 |

```python
# Rustup 风格安装
script_install = platform_script_install(
    unix    = curl_sh_install("https://sh.rustup.rs"),
    windows = irm_iex_install("https://win.rustup.rs"),
)
```

---

### 6.12 `semver.star` — 版本比较

| 函数 | 签名 | 说明 |
|------|------|------|
| `semver_strip_v(version)` | `→ string` | 剥离 `v` 前缀 |
| `semver_parse(version)` | `→ [major, minor, patch]` | 解析为整数列表 |
| `semver_compare(a, b)` | `→ -1 \| 0 \| 1` | 比较两个版本 |
| `semver_gt(a, b)` | `→ bool` | 大于 |
| `semver_lt(a, b)` | `→ bool` | 小于 |
| `semver_gte(a, b)` | `→ bool` | 大于等于 |
| `semver_lte(a, b)` | `→ bool` | 小于等于 |
| `semver_eq(a, b)` | `→ bool` | 等于 |
| `semver_sort(versions, reverse=False)` | `→ list` | 版本排序 |

```python
load("@vx//stdlib:semver.star", "semver_gt", "semver_sort")

if semver_gt(version, "2.0.0"):
    # 使用新 API 格式
    pass

sorted_versions = semver_sort(["1.2.3", "1.0.0", "2.0.0"])
# → ["1.0.0", "1.2.3", "2.0.0"]
```

---

### 6.13 `test.star` — 测试 DSL

| 函数 | 签名 | 说明 |
|------|------|------|
| `cmd(command, name=None, expect_success=True, expected_output=None, timeout_ms=None)` | `→ dict` | 运行命令并检查结果 |
| `check_path(path, name=None)` | `→ dict` | 断言路径存在 |
| `check_not_path(path, name=None)` | `→ dict` | 断言路径不存在 |
| `check_env(var_name, name=None, expected_output=None)` | `→ dict` | 断言环境变量已设置 |
| `check_not_env(var_name, name=None)` | `→ dict` | 断言环境变量未设置 |
| `check_file(path, name=None, expected_output=None)` | `→ dict` | 断言文件存在且内容匹配 |

在运行时定义的 `test_commands` 中使用：

```python
runtimes = [
    runtime_def("node",
        test_commands=[
            cmd("{executable} --version",
                name="version_check",
                expected_output="v\\d+\\.\\d+"),
            check_path("{install_dir}/bin/node",
                name="binary_exists"),
        ],
    ),
]
```

---

### 6.14 `provider_templates.star` — 高级模板

模板返回一个预配置了所有标准 Provider 函数的 **dict**。解包到模块级变量中使用。

#### `github_rust_provider(owner, repo, **kwargs) → dict`

适用于在 GitHub releases 中使用 Rust 目标三元组命名的工具。

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `owner` | `string` | — | GitHub 所有者 |
| `repo` | `string` | — | GitHub 仓库 |
| `asset` | `string` | — | 资源名称模板 |
| `executable` | `string` | `repo` | 可执行文件名 |
| `store` | `string` | `repo` | Store 目录名 |
| `tag_prefix` | `string` | `"v"` | 要剥离的 tag 前缀 |
| `linux_libc` | `string` | `"musl"` | `"musl"` 或 `"gnu"` |
| `prereleases` | `bool` | `False` | 是否包含预发布 |
| `strip_prefix` | `string` | `None` | 要剥离的归档目录前缀 |
| `path_env` | `string` | `None` | 自定义 PATH 环境 |
| `extra_env` | `list` | `None` | 额外的环境变量操作 |

**资源模板占位符：** `{version}`、`{vversion}`、`{triple}`、`{ext}`、`{exe}`

```python
_p = github_rust_provider("BurntSushi", "ripgrep",
    asset        = "ripgrep-{version}-{triple}.{ext}",
    executable   = "rg",
    tag_prefix   = "",
    strip_prefix = "ripgrep-{version}-{triple}",
)
# 资源示例：ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz
```

#### `github_go_provider(owner, repo, **kwargs) → dict`

适用于使用 Go 风格 `{os}_{arch}` 命名的工具（goreleaser 模式）。

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `owner` | `string` | — | GitHub 所有者 |
| `repo` | `string` | — | GitHub 仓库 |
| `asset` | `string` | — | 资源名称模板 |
| `executable` | `string` | `repo` | 可执行文件名 |
| `store` | `string` | `repo` | Store 目录名 |
| `tag_prefix` | `string` | `"v"` | Tag 前缀 |
| `prereleases` | `bool` | `False` | 是否包含预发布 |
| `strip_prefix` | `string` | `None` | 要剥离的目录前缀 |
| `path_env` | `string` | `None` | 自定义 PATH 环境 |
| `extra_env` | `list` | `None` | 额外的环境变量操作 |

**资源模板占位符：** `{version}`、`{vversion}`、`{os}`、`{arch}`、`{ext}`、`{exe}`

```python
_p = github_go_provider("cli", "cli",
    asset        = "gh_{version}_{os}_{arch}.{ext}",
    executable   = "gh",
    strip_prefix = "gh_{version}_{os}_{arch}",
)
# 资源示例：gh_2.67.0_linux_amd64.tar.gz
```

#### `github_binary_provider(owner, repo, **kwargs) → dict`

适用于分发单个可执行文件的工具（无归档包）。

```python
_p = github_binary_provider("kubernetes", "kubectl",
    asset = "kubectl{exe}",
)
```

#### `system_provider(store_name, **kwargs) → dict`

适用于仅通过系统包管理器安装的工具。

```python
_p = system_provider("7zip", executable="7z")
```

#### 模板返回的 dict 键

所有模板返回包含以下键的 dict：

| 键 | 类型 | 说明 |
|----|------|------|
| `"fetch_versions"` | `function` | 版本获取器 |
| `"download_url"` | `function` | URL 构建器 |
| `"install_layout"` | `function` | 布局描述符 |
| `"store_root"` | `function` | Store 根路径 |
| `"get_execute_path"` | `function` | 可执行文件路径 |
| `"post_install"` | `function` | 安装后钩子 |
| `"environment"` | `function` | 环境设置 |
| `"deps"` | `function` | 依赖 |

**解包模式：**

```python
_p = github_rust_provider(...)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

---

## 7. 安装布局类型

`install_layout()` 函数返回描述符 dict。`__type`（或 `type`）字段决定策略：

| 类型 | 必需字段 | 可选字段 | 用途 |
|------|---------|---------|------|
| `"archive"` | `type` | `strip_prefix`、`executable_paths` | tar.gz、zip 归档 |
| `"binary"` | `type` | `executable_name`、`permissions` | 直接可执行文件下载 |
| `"msi"` | `type`、`url` | `executable_paths`、`strip_prefix`、`extra_args` | Windows MSI 安装 |
| `"system_find"` | `type`、`executable` | `system_paths`、`hint` | 系统已安装工具查找 |

### 归档布局

```python
def install_layout(ctx, version):
    return {
        "type":             "archive",
        "strip_prefix":     "mytool-v{}".format(version),
        "executable_paths": ["bin/mytool", "mytool"],
    }
```

### 二进制布局

```python
def install_layout(ctx, version):
    exe = "mytool.exe" if ctx.platform.os == "windows" else "mytool"
    return {
        "type":            "binary",
        "executable_name": exe,
        "permissions":     "755",
    }
```

### MSI 布局（Windows）

```python
def install_layout(ctx, version):
    return {
        "type":             "msi",
        "url":              download_url(ctx, version),
        "executable_paths": ["bin/tool.exe", "tool.exe"],
        "extra_args":       ["/quiet", "/norestart"],
    }
```

### 系统查找布局

```python
def install_layout(ctx, version):
    return {
        "type":         "system_find",
        "executable":   "cmake",
        "system_paths": ["/usr/local/bin/cmake", "C:\\Program Files\\CMake\\bin\\cmake.exe"],
        "hint":         "通过 'brew install cmake' 或 'winget install Kitware.CMake' 安装",
    }
```

---

## 8. 版本获取策略

| 策略 | 函数 | 适用场景 |
|------|------|---------|
| **GitHub releases（模板）** | `make_fetch_versions(owner, repo)` | 大多数 GitHub 托管工具 |
| **GitHub releases（原始）** | `github_releases(ctx, owner, repo)` | 需要自定义过滤 |
| **非标准 tag 前缀** | `fetch_versions_with_tag_prefix(owner, repo, "bun-v")` | 如 `bun-v1.2.3` 格式的 tag |
| **官方 API** | `fetch_versions_from_api(url, transform)` | Node.js、Go、Java 等 |
| **自定义** | 手写 `fetch_versions(ctx)` | 不常见的版本源 |

### 选择指南

```
                  ┌─ GitHub releases？ ──┐
                  │                      │
              ┌─ 是 ─┐            ┌── 否 ──┐
              │       │           │         │
        标准 tag？   非标准         有官方 API？
        (v1.2.3)     (bun-v1.2.3)       │
              │            │         ┌─ 是 ──┐
     make_fetch_versions   fetch_versions_   fetch_versions_
                           with_tag_prefix   from_api
                                            │
                                        ┌─ 否 ──┐
                                        │        │
                                   自定义 fetch_versions()
```

---

## 9. 钩子

### 解压后钩子

在归档解压后执行。用途：
- 展平嵌套目录
- 创建 shim 脚本
- 设置 Unix 文件权限

```python
# 展平 JDK 目录（jdk-21.0.1+12/ → 内容移动到根目录）
post_extract = post_extract_flatten(pattern="jdk-*")

# 创建 shim：`bunx` → `bun x`
post_extract = post_extract_shim("bunx", "bun", args=["x"])

# 设置多个文件的权限
post_extract = post_extract_permissions(["bin/node", "bin/npm", "bin/npx"])

# 组合
post_extract = post_extract_combine([
    post_extract_flatten(pattern="jdk-*"),
    post_extract_permissions(["bin/java", "bin/javac"]),
])
```

### 运行前钩子

在运行时命令执行前运行。用于自动安装项目依赖：

```python
# 执行 `npm run ...` 前确保 node_modules 存在
pre_run = pre_run_ensure_deps("npm",
    trigger_args = ["run", "run-script"],
    check_file   = "package.json",
    install_dir  = "node_modules",
)
```

---

## 10. Starlark 语言子集

provider.star 使用 [Starlark](https://github.com/bazelbuild/starlark)，一种有意设计限制的类 Python 语言：

### 支持的特性

| 特性 | 示例 |
|------|------|
| 变量 | `x = 42` |
| 字符串 | `"hello"`、`'hello'`、`"""多行"""` |
| 字符串格式化 | `"v{}".format(version)` |
| 列表 | `[1, 2, 3]`、列表推导式 |
| 字典 | `{"key": "value"}`、字典推导式 |
| 函数 | `def my_func(arg1, arg2="default"):` |
| 条件 | `if/elif/else` |
| 循环 | `for x in collection:` |
| 布尔逻辑 | `and`、`or`、`not` |
| None | `None` |
| 字符串方法 | `.format()`、`.get()`、`.startswith()` 等 |
| `load()` | `load("@vx//stdlib:module.star", "symbol")` |
| `fail()` | `fail("error message")` — 终止并报错 |

### 不支持的特性

| 特性 | 原因 |
|------|------|
| `import` | 使用 `load()` 代替 |
| `class` | Starlark 不支持 |
| `try/except` | 无异常处理 |
| `with` | 无上下文管理器 |
| `lambda` | 不支持 |
| `*args, **kwargs` | 不支持 |
| 冻结后修改 | 模块加载后顶层值不可变 |
| 副作用 | 无 I/O、网络或文件系统访问 |

### 与 Python 的关键区别

1. **冻结值不可修改** — 模块加载完成后，顶层数据结构不可变
2. **无 `set` 类型** — 使用 `dict` 或列表去重
3. **整数除法** — `//` 是整数除法，`/` 不可用
4. **字符串拼接** — `"a" + "b"` 可用，但推荐使用 `str.format()`
5. **无全局状态** — 函数不能修改模块级变量

---

## 11. 编码规范

### 命名

| 类别 | 约定 | 示例 |
|------|------|------|
| 模块变量 | `snake_case` | `name`、`fetch_versions` |
| 函数 | `snake_case` | `download_url()`、`install_layout()` |
| 私有函数 | `_` 前缀 | `_my_platform()`、`_triple()` |
| 常量 | `UPPER_SNAKE_CASE` 或 `_` 前缀 | `_PLATFORMS`、`RUST_TRIPLES_MUSL` |
| 模板变量 | `_p` | `_p = github_rust_provider(...)` |

### 文件组织

```python
# 1. load() 语句
load("@vx//stdlib:provider.star", ...)

# 2. 元数据变量
name        = "..."
description = "..."

# 3. 运行时定义
runtimes = [...]

# 4. 权限
permissions = ...

# 5. 私有辅助函数
def _my_platform(ctx): ...
_PLATFORMS = {...}

# 6. Provider 函数（或模板解包）
fetch_versions   = ...
download_url     = ...
install_layout   = ...
store_root       = ...
get_execute_path = ...
environment      = ...
```

### 平台处理

```python
# ✅ 正确 — 不支持的平台返回 None
def download_url(ctx, version):
    triple = platform_map(ctx, _PLATFORMS)
    if not triple:
        return None
    # ...

# ❌ 错误 — 不支持的平台使用 fail()
def download_url(ctx, version):
    triple = platform_map(ctx, _PLATFORMS)
    if not triple:
        fail("不支持的平台")  # 不要这样做！
```

### 字符串格式化

```python
# ✅ 正确 — 使用 .format()
url = "https://example.com/v{}/tool-{}.tar.gz".format(version, triple)

# ❌ 错误 — 使用 f-string（Starlark 不支持）
url = f"https://example.com/v{version}/tool-{triple}.tar.gz"

# ❌ 错误 — 使用 % 格式化（在 Starlark 中不可靠）
url = "https://example.com/v%s/tool-%s.tar.gz" % (version, triple)
```

### 未使用的参数

```python
# ✅ 正确 — 以下划线前缀
def deps(_ctx, _version):
    return []

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

---

## 12. 清单：创建新 Provider

创建新 Provider 时使用此清单：

- [ ] 创建 `crates/vx-providers/<name>/provider.star`
- [ ] 设置元数据：`name`、`description`、`ecosystem`、`license`
- [ ] 使用 `runtime_def()` 定义 `runtimes`（捆绑工具用 `bundled_runtime_def()`）
- [ ] 用 `github_permissions()` 或 `system_permissions()` 声明 `permissions`
- [ ] 选择策略：
  - [ ] **模板** — `github_rust_provider()`、`github_go_provider()`、`github_binary_provider()`
  - [ ] **自定义函数** — 手写 `fetch_versions`、`download_url`、`install_layout`
- [ ] 定义 `environment()`（至少将安装目录前置到 PATH）
- [ ] 按需添加钩子：
  - [ ] `post_extract` — 权限、shim、目录展平
  - [ ] `pre_run` — 依赖自动安装
- [ ] 如果工具依赖其他运行时，声明 `deps()`
- [ ] 添加 `system_install` 作为系统包管理器回退
- [ ] 在运行时定义中添加 `test_commands`
- [ ] 测试：`vx <runtime> --version`
- [ ] 在所有支持的平台上测试（Windows、macOS、Linux）

---

## 另请参阅

- [声明式 Provider](./manifest-driven-providers.md) — 入门指南
- [Starlark Provider — 高级指南](./starlark-providers.md) — 多运行时 Provider、自定义版本源
- [vx.toml 参考](../config/vx-toml.md) — 项目配置
- [vx.toml 语法指南](./vx-toml-syntax.md) — 模式与配方
