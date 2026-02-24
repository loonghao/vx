# RFC 0038: provider.star — 简洁优先的统一 Provider 格式

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2026-02-22
> **目标版本**: v0.16.0
> **依赖 RFC**: [RFC 0036](./0036-starlark-provider-support.md), [RFC 0037](./0037-provider-star-unified-facade.md)

## 摘要

将 `provider.star` 确立为描述 Provider 的**唯一格式**，完全替代 `provider.toml`。

本 RFC 在 RFC 0037 的基础上，重新审视 `provider.star` 的 API 设计，核心目标是：

- **简洁优先**：去掉冗余语法，只保留核心必需字段
- **缺省友好**：绝大多数字段有合理默认值，可以省略
- **动态优先**：所有静态字段都可以被同名函数动态覆盖
- **一致性**：统一函数签名，消除当前 API 中的不一致

参考 [rez](https://github.com/AcademySoftwareFoundation/rez) 的 `package.py` 设计理念，但针对 vx 的场景做了大幅简化。

---

## 动机：当前 API 的问题

### 问题 1：元数据用函数而非变量（冗余）

当前写法：

```python
def name():
    return "node"

def description():
    return "Node.js - JavaScript runtime built on Chrome's V8 engine"

def ecosystem():
    return "nodejs"
```

这是纯粹的冗余。静态字符串没有理由包在函数里。rez 的 `package.py` 直接用顶层变量：

```python
name        = "node"
description = "Node.js - JavaScript runtime built on Chrome's V8 engine"
ecosystem   = "nodejs"
```

### 问题 2：`make_github_provider` 的赋值模式不直观

当前写法：

```python
_p = make_github_provider("astral-sh", "uv", "uv-{triple}.{ext}")
fetch_versions = _p["fetch_versions"]
download_url   = _p["download_url"]
```

这是绕过 Starlark 不支持多返回值的 hack。应该改为更直观的方式：

```python
load("@vx//stdlib:github.star", "github_provider")

_gh = github_provider("astral-sh", "uv", asset = "uv-{triple}.{ext}")
```

然后 `fetch_versions` 和 `download_url` 自动从 `_gh` 中解析，或者直接支持 `provider` 顶层变量。

### 问题 3：`post_install` 和 `post_extract` 重复

当前同时存在 `post_install` 和 `post_extract`，语义重叠，开发者不知道用哪个。

### 问题 4：函数签名不统一

```python
def store_root(ctx):          # 没有 version
def get_execute_path(ctx, version):  # 有 version
def post_install(ctx, version, install_dir):  # 有 install_dir
def post_extract(ctx, version, install_dir):  # 同上，重复
```

### 问题 5：没有明确的必需/可选分层

开发者不知道哪些函数必须实现，哪些可以省略。

---

## 设计方案：简洁优先的 provider.star v3

### 核心原则

```
1. 顶层变量 = 静态元数据（零运行时开销）
2. 同名函数 = 动态覆盖（按需执行）
3. 缺省推断 = 能省则省
4. 统一签名 = 所有函数签名一致
```

### 必需 vs 可选一览

```
必需（2个）：
  name            顶层变量或函数
  fetch_versions  函数

强烈推荐（3个）：
  runtimes        顶层变量（缺省时从 name 推断单 runtime）
  download_url    函数（缺省时无法自动安装）
  install_layout  函数（缺省时使用 archive 默认布局）

可选（其余全部）：
  description, ecosystem, homepage, repository, license,
  platforms, authors, permissions,
  environment, post_install, pre_run, deps, constraints,
  system_install, detection, mirrors, health_check
```

---

## 完整 API 规范

### 层级 0：最小可用 Provider（仅系统工具）

```python
# ~/.vx/providers/my-tool/provider.star
# 最简形式：只声明名字，依赖系统已安装的工具

name = "my-tool"

def fetch_versions(ctx):
    return [{"version": "system"}]
```

### 层级 1：标准 GitHub Provider（最常见）

```python
# provider.star - 标准 GitHub Release Provider
# 覆盖 90% 的使用场景

load("@vx//stdlib:github.star", "github_releases", "github_asset")

# ── 元数据（顶层变量，全部可选除 name）────────────────────────
name        = "mytool"
description = "My awesome tool"
homepage    = "https://example.com"
repository  = "https://github.com/example/mytool"
license     = "MIT"
ecosystem   = "custom"

# ── Runtime 定义（可选，缺省从 name 推断单 runtime）────────────
runtimes = [
    {"name": "mytool", "executable": "mytool"},
]

# ── 权限声明（可选，缺省允许访问 repository 域名）──────────────
permissions = {
    "http": ["api.github.com", "github.com"],
}

# ── 版本获取（必需）────────────────────────────────────────────
fetch_versions = github_releases("example", "mytool")

# ── 下载 URL（强烈推荐）────────────────────────────────────────
download_url = github_asset("example", "mytool", "mytool-{triple}.{ext}")

# ── 安装布局（可选，缺省 archive + 自动探测可执行文件）──────────
def install_layout(ctx, version):
    return {
        "type":         "archive",
        "strip_prefix": "mytool-{version}".format(version = version),
    }

# ── 环境变量（可选）────────────────────────────────────────────
def environment(ctx, version, install_dir):
    return {"PATH": install_dir + "/bin"}
```

### 层级 2：完整自定义 Provider

```python
# provider.star - 完整自定义 Provider（展示所有可用字段和函数）

load("@vx//stdlib:github.star", "github_releases")
load("@vx//stdlib:install.star", "set_permissions", "run_command", "ensure_dependencies")

# ============================================================
# 静态元数据（顶层变量）
# 所有字段均可被同名函数动态覆盖
# ============================================================

name        = "mytool"
version     = "1.0.0"          # Provider 自身版本（非工具版本）
description = "My awesome tool"
homepage    = "https://example.com"
repository  = "https://github.com/example/mytool"
license     = "MIT"
ecosystem   = "custom"         # nodejs | python | rust | go | system | custom
authors     = ["vx team"]

# Provider 级平台约束（缺省不限制）
platforms = {"os": ["windows", "linux", "macos"]}

# ============================================================
# Runtime 定义
#
# 缺省规则：
#   - 若省略 runtimes，自动推断为 [{"name": name, "executable": name}]
#   - executable 缺省等于 name
#   - description 缺省等于 Provider 的 description
#   - priority 缺省为 100
#   - auto_installable 缺省为 True
# ============================================================

runtimes = [
    {
        # 必需
        "name":       "mytool",

        # 可选（有缺省值）
        "executable":  "mytool",       # 缺省 = name
        "description": "My tool",      # 缺省 = provider description
        "aliases":     ["mt"],         # 缺省 = []
        "priority":    100,            # 缺省 = 100

        # 平台约束（可选）
        "platform_constraint": {"os": ["windows", "linux", "macos"]},

        # 系统检测（可选）—— 用于发现系统已安装的版本
        "detection": {
            "command": "mytool --version",
            "pattern": r"(\d+\.\d+\.\d+)",
        },

        # 系统安装策略（可选）—— 通过系统包管理器安装
        "system_install": [
            {"manager": "brew",   "package": "mytool"},
            {"manager": "winget", "package": "Example.MyTool"},
            {"manager": "choco",  "package": "mytool"},
        ],

        # 版本约束（可选）—— 声明对其他 runtime 的依赖
        "requires": [
            {"runtime": "node", "version": ">=18"},
        ],

        # 镜像（可选）—— 下载加速
        "mirrors": [
            "https://mirror.example.com/mytool/{version}/{filename}",
        ],
    },
]

# ============================================================
# 权限声明（可选）
# 缺省：允许访问 repository 字段中的域名
# ============================================================

permissions = {
    "http": ["api.github.com", "github.com"],
}

# ============================================================
# 核心函数
# ============================================================

# ── fetch_versions（必需）──────────────────────────────────────
# ctx 字段：
#   ctx.http.get_json(url)   → dict/list
#   ctx.http.get_text(url)   → str
#   ctx.platform.os          → "windows" | "linux" | "macos"
#   ctx.platform.arch        → "x64" | "arm64" | "x86"
#   ctx.cache.get(key)       → value | None
#   ctx.cache.set(key, val)  → None

def fetch_versions(ctx):
    """Fetch available versions from upstream.

    Returns:
        List of version dicts. Required field: "version".
        Optional fields: "prerelease" (bool), "lts" (bool), "channel" (str).
    """
    releases = ctx.http.get_json(
        "https://api.github.com/repos/example/mytool/releases?per_page=50"
    )
    return [
        {
            "version":    r["tag_name"].lstrip("v"),
            "prerelease": r.get("prerelease", False),
        }
        for r in releases
    ]

# ── download_url（强烈推荐）────────────────────────────────────
# 返回 None 表示该平台不支持自动安装

def download_url(ctx, version):
    """Build platform-specific download URL.

    Args:
        ctx:     Provider context（同 fetch_versions）
        version: 版本字符串，如 "1.2.3"

    Returns:
        URL string，或 None（不支持该平台）
    """
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    triple = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-gnu",
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

    if not triple:
        return None

    return "https://github.com/example/mytool/releases/download/v{}/mytool-{}-{}.{}".format(
        version, version, triple, ext
    )

# ── install_layout（可选）──────────────────────────────────────
# 缺省：archive 类型，自动探测可执行文件

def install_layout(ctx, version):
    """Describe how to extract the downloaded archive.

    Returns dict with:
        type:             "archive" | "binary" | "git-clone"
        strip_prefix:     (archive only) 解压时去掉的前缀目录
        executable_paths: 可执行文件相对路径列表（缺省自动探测）
    """
    return {
        "type":         "archive",
        "strip_prefix": "mytool-{}".format(version),
        # executable_paths 省略 → 自动探测
    }

# ── environment（可选）─────────────────────────────────────────
# 缺省：将 install_dir/bin 加入 PATH（Unix），install_dir 加入 PATH（Windows）

def environment(ctx, version, install_dir):
    """Return env vars to inject when running this tool.

    Returns dict. Special key "PATH" 会被 prepend 到系统 PATH。
    """
    if ctx.platform.os == "windows":
        return {"PATH": install_dir}
    return {"PATH": install_dir + "/bin"}

# ── post_install（可选）────────────────────────────────────────
# 安装完成后执行的动作（设置权限、运行初始化命令等）

def post_install(ctx, version, install_dir):
    """Post-installation actions.

    Returns list of action dicts（由 vx//stdlib:install.star 的辅助函数生成）。
    """
    if ctx.platform.os == "windows":
        return []
    return [set_permissions("bin/mytool", "755")]

# ── pre_run（可选）─────────────────────────────────────────────
# 每次执行前的检查（如确保依赖已安装）

def pre_run(ctx, args):
    """Pre-execution hook.

    Args:
        ctx:  Provider context
        args: 传给工具的命令行参数列表

    Returns list of action dicts，或空列表。
    """
    return []

# ── deps（可选）────────────────────────────────────────────────
# 动态依赖声明（覆盖 runtimes[].requires 静态声明）

def deps(ctx, version):
    """Dynamic dependency declarations.

    Returns list of {"runtime": str, "version": str} dicts.
    Use runtimes[].requires for static deps; use this only when
    deps depend on runtime conditions (e.g., platform, version).
    """
    return []

# ── constraints（可选）─────────────────────────────────────────
# 版本兼容性约束

def constraints(ctx, version):
    """Version compatibility constraints.

    Returns list of constraint dicts.
    """
    return []

# ── system_install（可选）──────────────────────────────────────
# 动态系统安装策略（覆盖 runtimes[].system_install 静态声明）

def system_install(ctx):
    """Dynamic system install strategies.

    Returns list of {"manager": str, "package": str} dicts,
    or None to use static runtimes[].system_install.
    """
    return None

# ── mirrors（可选）─────────────────────────────────────────────

def mirrors(ctx, version):
    """Return mirror URLs for download acceleration.

    Returns list of URL template strings.
    {version} and {filename} are substituted automatically.
    """
    return []

# ── health_check（可选）────────────────────────────────────────

def health_check(ctx, version, install_dir):
    """Verify the installation is functional.

    Returns {"ok": bool, "message": str}.
    缺省：运行 `{executable} --version` 并检查退出码。
    """
    return None  # 使用缺省检查
```

---

## justfile 风格：模板内联与环境变量渲染（Draft v5 新增）

参考 [just](https://github.com/casey/just) 的设计，在 `provider.star` 中引入**模板内联**和**环境变量渲染**能力，让 Provider 可以直接在字符串中嵌入变量、读取环境变量、执行命令捕获输出，大幅减少重复的字符串拼接代码。

### 设计动机

当前 `provider.star` 中大量的字符串拼接代码：

```python
# 当前：繁琐的字符串拼接
def download_url(ctx, version):
    return "https://github.com/example/mytool/releases/download/v{}/mytool-{}-{}-{}.{}".format(
        version, version, ctx.platform.os, ctx.platform.arch, "zip" if ctx.platform.os == "windows" else "tar.gz"
    )

def install_layout(ctx, version):
    return {
        "type":         "archive",
        "strip_prefix": "mytool-{}-{}-{}".format(version, ctx.platform.os, ctx.platform.arch),
    }
```

引入模板内联后：

```python
# 新：模板内联，简洁直观
def download_url(ctx, version):
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return ctx.render("https://github.com/example/mytool/releases/download/v{version}/mytool-{version}-{os}-{arch}.{ext}")

def install_layout(ctx, version):
    return {
        "type":         "archive",
        "strip_prefix": ctx.render("mytool-{version}-{os}-{arch}"),
    }
```

---

### 核心设计：`ctx.render()` 模板渲染

`ctx.render(template)` 是核心渲染函数，支持以下内置变量：

#### 内置模板变量

| 变量 | 含义 | 示例值 |
|------|------|--------|
| `{version}` | 当前工具版本 | `"1.2.3"` |
| `{os}` | 操作系统 | `"windows"` / `"linux"` / `"macos"` |
| `{arch}` | CPU 架构 | `"x64"` / `"arm64"` / `"x86"` |
| `{triple}` | 完整平台三元组 | `"x86_64-pc-windows-msvc"` |
| `{ext}` | 平台默认压缩格式 | `"zip"` (Windows) / `"tar.gz"` (其他) |
| `{exe}` | 可执行文件后缀 | `".exe"` (Windows) / `""` (其他) |
| `{name}` | Provider 名称 | `"mytool"` |
| `{install_dir}` | 安装目录（仅在 `environment`/`post_install` 中可用） | `"/home/user/.vx/store/mytool/1.2.3"` |
| `{store_dir}` | vx store 根目录 | `"/home/user/.vx/store"` |
| `{cache_dir}` | vx cache 目录 | `"/home/user/.vx/cache"` |
| `{home_dir}` | 用户 home 目录 | `"/home/user"` |

#### 平台三元组映射（`{triple}`）

| 平台 | `{triple}` |
|------|-----------|
| Windows x64 | `x86_64-pc-windows-msvc` |
| Windows arm64 | `aarch64-pc-windows-msvc` |
| macOS x64 | `x86_64-apple-darwin` |
| macOS arm64 | `aarch64-apple-darwin` |
| Linux x64 | `x86_64-unknown-linux-gnu` |
| Linux arm64 | `aarch64-unknown-linux-gnu` |
| Linux x86 | `i686-unknown-linux-gnu` |

#### 使用示例

```python
# provider.star - 使用 ctx.render() 简化字符串构建

name       = "mytool"
repository = "https://github.com/example/mytool"

def download_url(ctx, version):
    # {triple} 自动映射到平台三元组，{ext} 自动选择 zip/tar.gz
    return ctx.render("https://github.com/example/mytool/releases/download/v{version}/mytool-{version}-{triple}.{ext}")

def install_layout(ctx, version):
    return {
        "type":         "archive",
        "strip_prefix": ctx.render("mytool-{version}-{triple}"),
    }

def environment(ctx, version, install_dir):
    # {install_dir} 在 environment 函数中可用
    return {
        "MYTOOL_HOME": ctx.render("{install_dir}"),
        "PATH":        ctx.render("{install_dir}/bin"),
    }
```

---

### 环境变量读取：`ctx.env()`

参考 justfile 的 `env()` 函数，`ctx.env()` 允许 Provider 读取宿主机的环境变量：

```python
# 读取环境变量（必须存在，否则报错）
token = ctx.env("GITHUB_TOKEN")

# 读取环境变量（带默认值，不存在时使用默认值）
mirror = ctx.env("VX_MIRROR", "https://github.com")
proxy  = ctx.env("HTTPS_PROXY", "")

# 检查环境变量是否存在
if ctx.env("CI", "") != "":
    # 在 CI 环境中的特殊处理
    pass
```

#### 实际应用场景

```python
# provider.star - 支持镜像加速和私有 token

name = "mytool"

def fetch_versions(ctx):
    # 支持通过环境变量配置 GitHub Token，避免 API 限流
    token = ctx.env("GITHUB_TOKEN", "")
    headers = {}
    if token:
        headers["Authorization"] = "Bearer " + token

    return ctx.http.get_json(
        "https://api.github.com/repos/example/mytool/releases?per_page=50",
        headers = headers,
    )

def download_url(ctx, version):
    # 支持通过环境变量配置镜像源（国内加速）
    mirror = ctx.env("VX_GITHUB_MIRROR", "https://github.com")
    return ctx.render(mirror + "/example/mytool/releases/download/v{version}/mytool-{version}-{triple}.{ext}")

def environment(ctx, version, install_dir):
    # 继承宿主机的代理设置
    env = {"PATH": ctx.render("{install_dir}/bin")}
    https_proxy = ctx.env("HTTPS_PROXY", "")
    if https_proxy:
        env["HTTPS_PROXY"] = https_proxy
    return env
```

---

### 命令捕获：`ctx.shell()`

参考 justfile 的反引号（`` ` ``）语法，`ctx.shell()` 允许执行命令并捕获输出：

```python
# 执行命令并捕获 stdout（去除首尾空白）
current_user = ctx.shell("whoami")
home_dir     = ctx.shell("echo $HOME")

# 带参数的命令
git_root = ctx.shell("git", ["rev-parse", "--show-toplevel"])

# 带环境变量的命令
output = ctx.shell("my-cmd", ["--version"], env = {"MY_VAR": "value"})

# 允许失败（不抛出异常）
result = ctx.shell("which", ["curl"], allow_failure = True)
if result != "":
    # curl 存在
    pass
```

#### 实际应用场景

```python
# provider.star - 使用 ctx.shell() 动态检测环境

name = "mytool"

def fetch_versions(ctx):
    # 检测系统已安装的版本（用于 system 类型 provider）
    installed = ctx.shell("mytool", ["--version"], allow_failure = True)
    versions = []
    if installed:
        versions.append({"version": installed.strip(), "source": "system"})

    # 再从远程获取可用版本
    remote = ctx.http.get_json("https://api.example.com/versions")
    versions += [{"version": v} for v in remote]
    return versions

def post_install(ctx, version, install_dir):
    # 检测系统架构（更精确的检测）
    uname = ctx.shell("uname", ["-m"], allow_failure = True)
    if uname == "aarch64" and ctx.platform.os == "linux":
        # ARM Linux 需要额外的初始化步骤
        return [run_command(ctx.render("{install_dir}/bin/mytool"), ["--init-arm"])]
    return []
```

> **安全说明**：`ctx.shell()` 受 `permissions` 字段约束。若 Provider 未在 `permissions.exec` 中声明允许执行的命令，调用将被拒绝。

---

### `.env` 文件支持：`ctx.dotenv()`

参考 justfile 的 `set dotenv-load` 特性，支持从 `.env` 文件加载变量：

```python
# provider.star - 支持从 .env 文件读取配置

name = "mytool"

def fetch_versions(ctx):
    # 从项目根目录的 .env 文件加载（如果存在）
    # 不存在时静默忽略
    ctx.dotenv(".env")
    ctx.dotenv(".env.local")  # 可以加载多个

    token = ctx.env("MYTOOL_API_TOKEN", "")
    # ...
```

`.env` 文件格式（标准格式）：

```bash
# .env
GITHUB_TOKEN=ghp_xxxxxxxxxxxx
VX_GITHUB_MIRROR=https://ghproxy.com/https://github.com
MYTOOL_API_TOKEN=my-secret-token
```

---

### 字符串辅助函数

参考 justfile 的内置字符串函数，在 `ctx` 上提供常用字符串处理：

```python
# 字符串处理
ctx.trim("  hello  ")              # → "hello"
ctx.trim_start("  hello  ")        # → "hello  "
ctx.trim_end("  hello  ")          # → "  hello"
ctx.replace("hello world", "world", "vx")  # → "hello vx"
ctx.starts_with("v1.2.3", "v")     # → True
ctx.ends_with("file.tar.gz", ".gz") # → True
ctx.split("a,b,c", ",")            # → ["a", "b", "c"]
ctx.join(["a", "b", "c"], "/")     # → "a/b/c"

# 版本处理
ctx.semver_major("1.2.3")          # → "1"
ctx.semver_minor("1.2.3")          # → "2"
ctx.semver_patch("1.2.3")          # → "3"
ctx.semver_compare("1.2.3", "1.3.0")  # → -1 (小于)
ctx.strip_v("v1.2.3")              # → "1.2.3"（去掉 v 前缀）

# 路径处理
ctx.path_join("a", "b", "c")       # → "a/b/c" (Unix) 或 "a\b\c" (Windows)
ctx.path_basename("/usr/bin/node") # → "node"
ctx.path_dirname("/usr/bin/node")  # → "/usr/bin"
```

---

### 完整示例：使用所有新特性

```python
# provider.star - 展示 justfile 风格的模板内联和环境变量渲染

load("@vx//stdlib:install.star", "set_permissions", "run_command")

name        = "mytool"
description = "My awesome tool with full template support"
repository  = "https://github.com/example/mytool"

# ── 权限声明（声明允许执行的命令）────────────────────────────
permissions = {
    "http": ["api.github.com", "github.com"],
    "exec": ["mytool", "uname"],   # 允许 ctx.shell() 执行的命令
    "env":  ["GITHUB_TOKEN", "VX_GITHUB_MIRROR", "HTTPS_PROXY", "CI"],  # 允许读取的环境变量
}

# ── 版本获取（使用 ctx.env() 支持 token 和镜像）──────────────
def fetch_versions(ctx):
    token = ctx.env("GITHUB_TOKEN", "")
    headers = {"Authorization": "Bearer " + token} if token else {}

    releases = ctx.http.get_json(
        "https://api.github.com/repos/example/mytool/releases?per_page=50",
        headers = headers,
    )
    return [
        {
            "version":    ctx.strip_v(r["tag_name"]),
            "prerelease": r.get("prerelease", False),
        }
        for r in releases
        if not r.get("draft", False)
    ]

# ── 下载 URL（使用 ctx.render() + ctx.env() 支持镜像）────────
def download_url(ctx, version):
    # 支持通过环境变量配置镜像源
    mirror = ctx.env("VX_GITHUB_MIRROR", "https://github.com")

    # ctx.render() 自动展开 {version}, {triple}, {ext}
    return ctx.render(
        mirror + "/example/mytool/releases/download/v{version}/mytool-{version}-{triple}.{ext}"
    )

# ── 安装布局（使用 ctx.render() 简化前缀构建）────────────────
def install_layout(ctx, version):
    return {
        "type":         "archive",
        "strip_prefix": ctx.render("mytool-{version}-{triple}"),
    }

# ── 环境变量（使用 ctx.render() + ctx.env() 继承代理）────────
def environment(ctx, version, install_dir):
    env = {
        "MYTOOL_HOME": ctx.render("{install_dir}"),
        "PATH":        ctx.render("{install_dir}/bin"),
    }
    # 继承宿主机的代理设置
    for proxy_var in ["HTTPS_PROXY", "HTTP_PROXY", "NO_PROXY"]:
        val = ctx.env(proxy_var, "")
        if val:
            env[proxy_var] = val
    return env

# ── 安装后处理（使用 ctx.shell() 动态检测）───────────────────
def post_install(ctx, version, install_dir):
    actions = []
    if ctx.platform.os != "windows":
        actions.append(set_permissions(ctx.render("{install_dir}/bin/mytool"), "755"))

    # 在 CI 环境中跳过初始化
    if ctx.env("CI", "") == "":
        actions.append(run_command(
            ctx.render("{install_dir}/bin/mytool"),
            ["--init"],
            env = {"MYTOOL_HOME": ctx.render("{install_dir}")},
        ))
    return actions
```

---

### `permissions.env` 安全模型

为了防止 Provider 随意读取宿主机的敏感环境变量，引入 `permissions.env` 白名单：

```python
permissions = {
    "http": ["api.github.com"],
    "exec": ["git", "curl"],
    "env":  [
        "GITHUB_TOKEN",          # 精确匹配
        "VX_*",                  # 通配符：所有 VX_ 前缀的变量
        "HTTPS_PROXY",
        "HTTP_PROXY",
        "NO_PROXY",
        "CI",
        "HOME",
    ],
}
```

| 权限类型 | 说明 | 缺省行为 |
|---------|------|---------|
| `permissions.http` | 允许访问的 HTTP 域名 | 从 `repository` 字段推断 |
| `permissions.exec` | 允许 `ctx.shell()` 执行的命令 | 空（不允许执行任何命令） |
| `permissions.env` | 允许 `ctx.env()` 读取的环境变量 | `["VX_*", "HOME", "PATH"]`（基础变量） |
| `permissions.fs` | 允许访问的文件系统路径 | `["{store_dir}", "{cache_dir}"]` |

> **注意**：`ctx.render()` 中的内置变量（`{version}`, `{os}`, `{arch}` 等）不受 `permissions.env` 约束，它们由 vx 引擎直接提供，不来自宿主机环境变量。

---

### 与 justfile 的对比

| 特性 | justfile | provider.star |
|------|---------|--------------|
| 字符串插值 | `{{variable}}` | `ctx.render("{variable}")` |
| 环境变量 | `env('VAR')` / `env_var('VAR', 'default')` | `ctx.env("VAR")` / `ctx.env("VAR", "default")` |
| 命令捕获 | `` `command` `` / `shell('cmd')` | `ctx.shell("cmd", args)` |
| `.env` 文件 | `set dotenv-load := true` | `ctx.dotenv(".env")` |
| 字符串函数 | `trim(s)`, `replace(s, a, b)` | `ctx.trim(s)`, `ctx.replace(s, a, b)` |
| 安全模型 | 无（完全信任） | `permissions` 白名单 |
| 执行环境 | Shell 脚本 | Starlark（沙箱） |

**关键区别**：justfile 直接在 shell 中执行，而 `provider.star` 运行在 Starlark 沙箱中，所有外部访问都需要通过 `permissions` 声明，安全性更高。

---

### 实施路线图（Draft v5 新增）

#### Phase 1.5：模板渲染引擎（v0.16.0，与 Phase 1 并行）

- [ ] 实现 `ctx.render(template)` — 内置变量展开（`{version}`, `{os}`, `{arch}`, `{triple}`, `{ext}`, `{exe}`, `{name}`, `{install_dir}`, `{store_dir}`, `{cache_dir}`, `{home_dir}`）
- [ ] 实现 `ctx.env(name)` / `ctx.env(name, default)` — 环境变量读取（受 `permissions.env` 约束）
- [ ] 实现 `ctx.shell(cmd, args, env, allow_failure)` — 命令捕获（受 `permissions.exec` 约束）
- [ ] 实现 `ctx.dotenv(path)` — `.env` 文件加载
- [ ] 实现字符串辅助函数：`ctx.trim`, `ctx.replace`, `ctx.split`, `ctx.join`, `ctx.starts_with`, `ctx.ends_with`
- [ ] 实现版本辅助函数：`ctx.strip_v`, `ctx.semver_major`, `ctx.semver_minor`, `ctx.semver_patch`, `ctx.semver_compare`
- [ ] 实现路径辅助函数：`ctx.path_join`, `ctx.path_basename`, `ctx.path_dirname`
- [ ] 实现 `permissions.env` 白名单校验
- [ ] 实现 `permissions.exec` 白名单校验
- [ ] 迁移所有内置 Provider 使用 `ctx.render()` 替代手动字符串拼接

---

## Spack 借鉴分析（Draft v4 新增）

研究 [Spack](https://github.com/spack/spack) 的 `package.py` 设计后，以下概念值得借鉴或明确不借鉴的原因。

### ✅ 借鉴：`variants`（构建变体）

Spack 的 `variant` 是其最独特的设计——同一个包可以用不同的"特性开关"构建出不同的版本：

```python
# Spack package.py
variant("mpi",  default=True,  description="Enable MPI support")
variant("cuda", default=False, description="Enable CUDA support", when="@1.1.0:")
```

**vx 的对应场景**：Provider 的某些功能是可选的，或者同一工具有多种"风味"（flavor）。

**借鉴方案**：在 `provider.star` 中引入 `variants` 声明，用于描述 Provider 的可选特性：

```python
# provider.star - 带 variants 的 Provider
variants = {
    "lts": {
        "default":     True,
        "description": "Use LTS (Long Term Support) version",
    },
    "nightly": {
        "default":     False,
        "description": "Use nightly build channel",
    },
}

def fetch_versions(ctx):
    if ctx.variant("lts"):
        # 只返回 LTS 版本
        return [v for v in all_versions if v.get("lts")]
    return all_versions
```

用户使用：
```bash
vx node +lts          # 使用 LTS 版本（默认）
vx node ~lts          # 使用最新版（非 LTS）
vx node +nightly      # 使用 nightly 构建
```

**优先级**：低（v0.17.0+，不阻塞当前 RFC）

---

### ✅ 借鉴：`conflicts`（冲突声明）

Spack 的 `conflicts` 允许 Provider 主动声明不兼容的配置：

```python
# Spack
conflicts("+cuda", when="platform=darwin", msg="CUDA not supported on macOS")
```

**vx 的对应场景**：某些版本组合或平台组合是不支持的，Provider 应该主动声明而非让用户踩坑。

**借鉴方案**：在 `provider.star` 中引入 `conflicts` 声明：

```python
# provider.star
conflicts = [
    {
        "when":    {"platform": {"os": "windows"}},
        "message": "This tool does not support Windows",
    },
    {
        "version": "<2.0",
        "when":    {"platform": {"arch": "arm64"}},
        "message": "ARM64 support requires version 2.0+",
    },
]
```

这比当前 `download_url` 返回 `None` 更明确——用户会得到清晰的错误信息而非神秘的安装失败。

**优先级**：中（v0.16.0 可以加入，作为 `platforms` 约束的补充）

---

### ✅ 借鉴：`provides`（虚拟包/能力声明）

Spack 的 `provides` 允许一个包声明它满足某个"虚拟依赖"：

```python
# Spack - openmpi 声明它提供 mpi 接口
provides("mpi", when="+mpi")
```

**vx 的对应场景**：`bun` 可以替代 `node`，`micromamba` 可以替代 `conda`。

**借鉴方案**：在 `provider.star` 中引入 `provides` 声明：

```python
# bun/provider.star
name     = "bun"
provides = ["node", "npm", "npx"]  # bun 可以满足对 node/npm/npx 的依赖

# 当其他 provider 声明 requires = ["node>=18"] 时，
# 如果环境中有 bun，可以用 bun 来满足该依赖
```

用户使用：
```bash
vx --with bun my-tool   # 用 bun 满足 my-tool 对 node 的依赖
```

**优先级**：中（v0.17.0，依赖解析引擎完成后）

---

### ✅ 借鉴：`when` 条件语法（统一条件表达式）

Spack 的 `when` 参数提供了统一的条件表达式语法，可以用在 `depends_on`、`conflicts`、`variant` 等任何地方：

```python
# Spack
depends_on("cuda@10.0:", when="+cuda")
depends_on("python@3.7:", when="@2.0:")
conflicts("+cuda", when="platform=darwin")
```

**vx 的对应场景**：当前 `requires` 的条件逻辑分散在 `def requires(ctx, version):` 函数中，不够声明式。

**借鉴方案**：为 `requires` 列表中的每个条目支持 `when` 字段：

```python
requires = [
    # 无条件依赖
    "node>=18",

    # 条件依赖（when 语法）
    {"runtime": "msvc",   "when": {"platform": {"os": "windows"}}},
    {"runtime": "python", "version": ">=3.11", "when": {"version": ">=2.0"}},
    {"runtime": "cuda",   "version": ">=11.0", "when": {"variant": "cuda"}},
]
```

这样大多数条件依赖可以用声明式写法，只有复杂逻辑才需要 `def requires(ctx, version):`。

**优先级**：中（v0.16.0 可以加入基础 `when` 支持）

---

### ✅ 借鉴：`deprecated` 版本标记

Spack 支持将某些版本标记为 deprecated：

```python
version('1.1.5', sha256='...', deprecated=True)
```

**vx 的对应场景**：`fetch_versions` 返回的版本列表中，某些版本有安全漏洞或已知问题。

**借鉴方案**：在 `fetch_versions` 返回的版本对象中支持 `deprecated` 和 `deprecated_reason` 字段：

```python
def fetch_versions(ctx):
    return [
        {"version": "1.2.0"},
        {"version": "1.1.5", "deprecated": True, "deprecated_reason": "Security vulnerability CVE-2024-XXXX"},
        {"version": "1.0.0", "deprecated": True},
    ]
```

**优先级**：低（v0.17.0+）

---

### ❌ 不借鉴：`patches`（补丁机制）

Spack 的 `patch` 用于在编译时修改源代码：

```python
patch('fix-memory-leak.patch', when='@1.0.0')
```

**不借鉴原因**：vx 分发的是**预编译二进制**，不需要编译时补丁。这是 Spack（源码包管理器）和 vx（二进制工具管理器）的根本区别。

---

### ❌ 不借鉴：`resource`（额外资源下载）

Spack 的 `resource` 用于在构建时下载额外的数据文件。

**不借鉴原因**：vx 不做编译，不需要构建时资源。如果工具需要额外数据，应该在 `post_install` 中处理。

---

### ❌ 不借鉴：`sha256` 校验和（强制）

Spack 对每个版本都要求 sha256 校验和，因为它从源码构建。

**不借鉴原因**：vx 下载预编译二进制，校验和由上游发布者管理（GitHub Releases 等）。vx 可以**可选地**支持校验和验证，但不强制要求 Provider 声明。

---

### 📋 Spack 借鉴总结

| Spack 概念 | vx 借鉴 | 优先级 | 版本 |
|-----------|---------|--------|------|
| `variant` | `variants` 构建变体 | 低 | v0.17.0+ |
| `conflicts` | `conflicts` 冲突声明 | 中 | v0.16.0 |
| `provides` | `provides` 能力声明 | 中 | v0.17.0 |
| `when` 条件 | `requires[].when` 条件依赖 | 中 | v0.16.0 |
| `deprecated` | `fetch_versions` 中的 `deprecated` 字段 | 低 | v0.17.0+ |
| `patches` | ❌ 不借鉴（vx 是二进制管理器） | - | - |
| `resource` | ❌ 不借鉴（无编译步骤） | - | - |
| `sha256` | 可选支持，不强制 | 低 | v0.17.0+ |

---

## 依赖解析设计

这是本 RFC 最重要的新增内容。参考 rez 的 `requires` 机制，vx 的 `provider.star` 支持**声明式依赖**，引擎自动完成传递依赖解析、版本约束求解和安装排序，大幅减少用户手动使用 `--with` 的场景。

### 设计目标

```
当前（手动）：vx --with node@20 --with python@3.12 my-tool
目标（自动）：vx my-tool   ← 引擎自动解析并安装 node@20 + python@3.12
```

### 依赖声明语法

#### 静态依赖（顶层变量，推荐）

```python
# provider.star - 声明静态依赖

name = "my-tool"

# 顶层 requires：Provider 级别的依赖，所有 runtime 共享
requires = [
    "node>=18",          # 需要 node 18+
    "python>=3.10,<4",   # 需要 python 3.10 到 4.0（不含）
    "uv",                # 需要 uv（任意版本）
]
```

#### Runtime 级别依赖（精细控制）

```python
runtimes = [
    {
        "name": "my-tool",
        # runtime 级别的 requires 只影响该 runtime
        "requires": [
            "node>=18",
        ],
    },
    {
        "name": "my-tool-python",
        "requires": [
            "python>=3.10",
            "uv",
        ],
    },
]
```

#### 动态依赖（函数形式，按需计算）

```python
# 当依赖取决于版本或平台时，使用函数形式
def requires(ctx, version):
    """Dynamic dependency declarations.

    Args:
        ctx:     Provider context
        version: 当前工具版本（如 "2.1.0"）

    Returns:
        List of requirement strings（与静态 requires 格式相同）
    """
    deps = ["node>=18"]

    # 旧版本需要额外依赖
    if version < "2.0.0":
        deps.append("python>=3.8")

    # Windows 需要额外工具
    if ctx.platform.os == "windows":
        deps.append("msvc")

    return deps
```

### 版本约束语法

参考 rez 和 PEP 440，支持以下约束语法：

| 语法 | 含义 | 示例 |
|------|------|------|
| `name` | 任意版本 | `"node"` |
| `name>=X` | 大于等于 X | `"node>=18"` |
| `name>X` | 大于 X | `"node>18"` |
| `name<=X` | 小于等于 X | `"node<=20"` |
| `name<X` | 小于 X | `"node<21"` |
| `name==X` | 精确版本 | `"node==20.0.0"` |
| `name!=X` | 排除版本 | `"node!=19"` |
| `name~=X.Y` | 兼容版本（~= 语义） | `"node~=20.0"` → `>=20.0,<21` |
| `name>=X,<Y` | 版本范围 | `"python>=3.10,<4"` |
| `name@X` | 精确版本（简写） | `"node@20.0.0"` |

### 传递依赖解析算法

引擎使用**约束传播 + 拓扑排序**算法，类似 rez 的 solver：

```
输入：vx my-tool

Step 1: 加载 my-tool/provider.star
  → requires = ["node>=18", "python>=3.10"]

Step 2: 递归解析依赖
  → node/provider.star: requires = []（无依赖）
  → python/provider.star: requires = ["uv"]（python 依赖 uv）
  → uv/provider.star: requires = []（无依赖）

Step 3: 构建依赖图
  my-tool
  ├── node (>=18)
  └── python (>=3.10)
      └── uv (any)

Step 4: 拓扑排序（安装顺序）
  [uv, node, python, my-tool]

Step 5: 版本约束求解
  uv:     latest（无约束）
  node:   20.x.x（满足 >=18，选最新稳定版）
  python: 3.12.x（满足 >=3.10,<4，选最新稳定版）

Step 6: 检查已安装版本
  uv:     已安装 0.5.0 ✓
  node:   未安装 → 自动安装 20.11.0
  python: 已安装 3.11.0 ✓（满足 >=3.10）

Step 7: 执行
  PATH = [uv/bin, node/bin, python/bin, my-tool/bin, ...]
```

### 冲突检测与解决

```python
# 场景：两个 provider 对同一 runtime 有冲突约束

# tool-a/provider.star
requires = ["node>=18,<20"]   # 需要 node 18.x

# tool-b/provider.star
requires = ["node>=20"]       # 需要 node 20+

# 同时运行 tool-a 和 tool-b 时：
# vx --with tool-a tool-b  → 冲突！
# 引擎报错：
#   Dependency conflict: node
#     tool-a requires: >=18,<20
#     tool-b requires: >=20
#     No version satisfies all constraints
```

冲突解决策略（按优先级）：

1. **用户显式指定**：`vx --with node@20 my-tool`（覆盖自动解析）
2. **宽松约束优先**：选择满足所有约束的最新版本
3. **报错提示**：无法自动解决时，给出清晰的冲突说明

### 弱依赖（可选依赖）

参考 rez 的 `~` 弱引用语法：

```python
requires = [
    "node>=18",          # 强依赖：必须安装
    "~python>=3.10",     # 弱依赖：如果 python 已安装，则要求 >=3.10；否则不安装
]
```

弱依赖的语义：
- 如果该 runtime 已在环境中（已安装或被其他依赖引入），则应用版本约束
- 如果该 runtime 不在环境中，**不会**触发自动安装

### 依赖类型分类（三类）

vx 的 Provider 依赖分为三类，语义完全不同：

| 字段 | 何时生效 | 是否注入 PATH | 是否传递给依赖者 | 典型场景 |
|------|---------|--------------|----------------|---------|
| `requires` | **运行时**（执行工具时） | ✅ 是 | ✅ 是 | `node>=18`、`python>=3.10` |
| `fetch_requires` | **安装阶段**（下载/解压/post_install） | ❌ 否（安装后丢弃） | ❌ 否 | `git`、`7zip`、`curl` |
| `build_requires` | **编译阶段**（从源码构建时） | ❌ 否 | ❌ 否 | `cmake`、`ninja`（vx 较少用） |

#### `fetch_requires`：安装阶段工具依赖

这是 vx 最常见的非运行时依赖场景。当 Provider 的安装过程需要调用外部工具时（如 `git clone`、`7zip` 解压），使用 `fetch_requires` 声明：

```python
# provider.star - 需要 git 和 7zip 才能安装的工具

name = "mytool"

# ── 安装阶段依赖（仅在 fetch_versions/download_url/post_install 中需要）──
# vx 会在执行安装步骤前自动确保这些工具可用，安装完成后不注入到运行时 PATH
fetch_requires = [
    "git",          # 需要 git clone 获取源码或资源
    "7zip",         # 需要解压 .7z 格式的压缩包
]

# ── 运行时依赖（执行 mytool 时需要）──────────────────────────
requires = [
    "node>=18",     # mytool 运行时需要 node
]

def fetch_versions(ctx):
    # 此处可以使用 git（已由 fetch_requires 保证可用）
    tags = ctx.shell("git", ["ls-remote", "--tags", "https://github.com/example/mytool"])
    return parse_tags(tags)

def post_install(ctx, version, install_dir):
    # 此处可以使用 7zip（已由 fetch_requires 保证可用）
    archive = ctx.render("{cache_dir}/mytool-{version}.7z")
    return [run_command("7z", ["x", archive, ctx.render("-o{install_dir}")])]
```

#### `fetch_requires` vs `permissions.exec` 的区别

这是两个**不同维度**的声明，互相补充：

```python
permissions = {
    # permissions.exec = 安全白名单：声明允许调用哪些命令（防止恶意 Provider）
    "exec": ["git", "7z", "curl"],
}

# fetch_requires = 依赖声明：声明 vx 需要自动安装哪些工具
fetch_requires = ["git", "7zip"]
```

| 维度 | `permissions.exec` | `fetch_requires` |
|------|-------------------|-----------------|
| **目的** | 安全审计（用户可见白名单） | 依赖管理（自动安装） |
| **作用** | 限制 `ctx.shell()` 可调用的命令 | 告诉 vx 安装前需要准备哪些工具 |
| **缺省行为** | 空（不允许执行任何命令） | 空（无额外工具依赖） |
| **是否影响 PATH** | 否 | 否（安装后丢弃） |

> **最佳实践**：`fetch_requires` 中声明的工具，也应该出现在 `permissions.exec` 中。vx 引擎可以自动从 `fetch_requires` 推断 `permissions.exec` 的部分内容，但显式声明更清晰。

#### 完整的三类依赖示例

```python
# provider.star - 展示三类依赖的完整用法

name = "complex-tool"

# ── 1. 运行时依赖：执行 complex-tool 时需要 ──────────────────
requires = [
    "node>=18",          # 运行时需要 node
    "~python>=3.10",     # 弱依赖：如果有 python 则要求 >=3.10
]

# ── 2. 安装阶段依赖：下载/解压/post_install 时需要 ───────────
fetch_requires = [
    "git",               # post_install 中需要 git 初始化子模块
    "7zip",              # 下载的是 .7z 格式
]

# ── 3. 构建时依赖（较少用，仅从源码编译时）────────────────────
# build_requires = ["cmake>=3.20", "ninja"]

# ── 权限声明（安全白名单）────────────────────────────────────
permissions = {
    "http": ["api.github.com", "github.com"],
    "exec": ["git", "7z"],   # fetch_requires 中的工具也需要在此声明
    "env":  ["GITHUB_TOKEN", "VX_GITHUB_MIRROR"],
}
```

#### 常见的 `fetch_requires` 场景

| 场景 | `fetch_requires` | 说明 |
|------|-----------------|------|
| 通过 `git clone` 安装 | `["git"]` | 如 spack、nvm 等通过 git 分发的工具 |
| 下载 `.7z` 格式 | `["7zip"]` | 部分 Windows 工具使用 7z 压缩 |
| `post_install` 需要 `curl` | `["curl"]` | 安装后需要额外下载资源 |
| 需要 `tar` 解压（非内置） | `["tar"]` | 极少数情况，vx 内置 tar 支持 |
| 需要 `unzip` | 无需声明 | vx 内置 zip 支持，不需要外部 unzip |
| 需要 `python` 运行安装脚本 | `["python"]` | 安装脚本是 Python 写的 |

> **注意**：vx 内置支持 `.zip`、`.tar.gz`、`.tar.xz`、`.tar.bz2` 的解压，这些格式**不需要** `fetch_requires`。只有 vx 不原生支持的格式（如 `.7z`）才需要声明外部工具依赖。

#### 参考 rez 的 `build_requires`

```python
# 运行时依赖（默认）：执行时需要
requires = ["node>=18"]

# 安装阶段依赖（vx 新增）：仅安装时需要，运行时不注入 PATH
fetch_requires = ["git", "7zip"]

# 构建时依赖（参考 rez）：仅从源码编译时需要，运行时不注入 PATH
build_requires = ["cmake>=3.20", "ninja"]

# 私有构建依赖：不传递给依赖者
private_build_requires = ["doxygen"]
```

### 完整示例：复杂依赖场景

```python
# provider.star - 一个需要多个运行时的复杂工具

load("@vx//stdlib:github.star", "github_releases", "github_asset")

name        = "my-fullstack-tool"
description = "A tool that needs Node.js, Python, and Rust"
repository  = "https://github.com/example/my-fullstack-tool"

# ── 静态依赖声明 ─────────────────────────────────────────────
requires = [
    "node>=18,<22",      # 前端构建
    "python>=3.10",      # 脚本运行
    "~rust>=1.70",       # 可选：如果有 rust 则要求 >=1.70
]

# ── 动态依赖（覆盖静态声明）──────────────────────────────────
def requires(ctx, version):
    deps = [
        "node>=18,<22",
        "python>=3.10",
    ]
    # v2.0+ 需要更新的 node
    if version >= "2.0.0":
        deps = ["node>=20,<22", "python>=3.11"]
    # Windows 需要 MSVC
    if ctx.platform.os == "windows":
        deps.append("msvc")
    return deps

# ── 版本获取 ─────────────────────────────────────────────────
fetch_versions = github_releases("example", "my-fullstack-tool")
download_url   = github_asset("example", "my-fullstack-tool", "tool-{triple}.{ext}")
```

### 与 `--with` 的关系

`--with` 语法**不会被删除**，而是成为自动依赖解析的**补充和覆盖机制**：

```bash
# 自动解析（推荐）：provider.star 声明了依赖，引擎自动处理
vx my-tool

# 手动补充：临时添加未在 provider.star 中声明的依赖
vx --with bun@1.1 my-tool

# 版本覆盖：覆盖 provider.star 中声明的版本约束
vx --with node@18 my-tool   # 强制使用 node 18，即使 provider 要求 >=20

# 完全手动（调试用）：忽略 provider.star 的依赖声明
vx --no-auto-deps --with node@20 --with python@3.12 my-tool
```

| 场景 | 推荐方式 |
|------|---------|
| Provider 的固定依赖 | `requires = [...]`（静态声明） |
| 依赖取决于版本/平台 | `def requires(ctx, version):` |
| 临时测试不同版本 | `vx --with node@18 my-tool` |
| 调试依赖问题 | `vx --no-auto-deps ...` |

### 依赖解析的 ctx 扩展

`requires` 函数中的 `ctx` 新增以下字段：

```python
ctx.resolved          # 当前已解析的依赖图（只读）
ctx.resolved.has("node")          # → bool
ctx.resolved.version("node")      # → "20.11.0" | None
ctx.resolved.satisfies("node>=18") # → bool
```

这允许依赖声明基于已解析的环境做决策：

```python
def requires(ctx, version):
    deps = ["node>=18"]
    # 如果环境中已有 bun，则不需要 node（bun 可以替代）
    if ctx.resolved.has("bun"):
        deps = []
    return deps
```

---

## 关键设计决策

### 决策 1：ctx 改为对象访问（`ctx.http.get_json`）而非字典访问（`ctx["http"]["get_json"]`）

**当前**：`ctx["http"]["get_json"](url)` — 字典嵌套，冗余的引号
**新设计**：`ctx.http.get_json(url)` — 对象属性访问，更简洁

Starlark 支持通过 `struct` 实现属性访问，这是更自然的写法。

```python
# 旧（当前）
releases = ctx["http"]["get_json"]("https://...")
os = ctx["platform"]["os"]

# 新
releases = ctx.http.get_json("https://...")
os = ctx.platform.os
```

### 决策 2：`fetch_versions` 和 `download_url` 支持直接赋值（不只是函数）

对于标准 GitHub Provider，允许直接赋值：

```python
# 函数形式（完全自定义）
def fetch_versions(ctx):
    ...

# 赋值形式（使用 stdlib 辅助函数，更简洁）
fetch_versions = github_releases("owner", "repo")
download_url   = github_asset("owner", "repo", "tool-{triple}.{ext}")
```

两种形式等价，引擎统一处理。

### 决策 3：消除 `post_extract` / `post_install` 重复

只保留 `post_install`，语义为"安装完成后执行"。

### 决策 4：`runtimes[].system_install` 改为列表而非嵌套对象

**旧（provider.toml 风格，嵌套复杂）**：
```python
"system_install": {
    "strategies": [
        {"type": "package_manager", "manager": "brew", "package": "mytool", "priority": 90},
    ]
}
```

**新（扁平列表，简洁）**：
```python
"system_install": [
    {"manager": "brew",   "package": "mytool"},
    {"manager": "winget", "package": "Example.MyTool"},
]
```

优先级由列表顺序决定，不需要额外的 `priority` 字段。

### 决策 5：`runtimes[].requires` 替代 `runtimes[].constraints`

`constraints` 这个词语义模糊。改为 `requires`，与 rez 保持一致：

```python
"requires": [
    {"runtime": "node", "version": ">=18"},
]
```

### 决策 6：`runtimes[]` 的缺省推断规则

| 字段 | 缺省值 |
|------|--------|
| `executable` | 等于 `name` |
| `description` | 等于 Provider 的 `description` |
| `aliases` | `[]` |
| `priority` | `100` |
| `auto_installable` | `True` |
| `platform_constraint` | 无限制 |

若整个 `runtimes` 省略，自动推断为：
```python
[{"name": name, "executable": name}]
```

### 决策 7：`permissions` 的缺省推断

若省略 `permissions`，自动从 `repository` 字段提取域名：
```python
# repository = "https://github.com/example/mytool"
# 自动推断：
permissions = {"http": ["api.github.com", "github.com"]}
```

---

## 实际迁移示例

### node provider（迁移前 → 迁移后）

**迁移前**（当前 301 行）：

```python
def name():
    return "node"

def description():
    return "Node.js - JavaScript runtime built on Chrome's V8 engine"

def homepage():
    return "https://nodejs.org"

def ecosystem():
    return "nodejs"

# ... 大量样板代码
```

**迁移后**（约 120 行，减少 60%）：

```python
# provider.star - Node.js provider

load("@vx//stdlib:install.star", "set_permissions", "ensure_dependencies")

# ── 元数据 ──────────────────────────────────────────────────
name        = "node"
description = "Node.js - JavaScript runtime built on Chrome's V8 engine"
homepage    = "https://nodejs.org"
repository  = "https://github.com/nodejs/node"
license     = "MIT"
ecosystem   = "nodejs"

# ── Runtimes ─────────────────────────────────────────────────
runtimes = [
    {
        "name":       "node",
        "executable": "node",
        "aliases":    ["nodejs"],
        "priority":   100,
        "detection":  {"command": "node --version", "pattern": r"v(\d+\.\d+\.\d+)"},
    },
    {"name": "npm",  "executable": "npm",  "bundled_with": "node"},
    {"name": "npx",  "executable": "npx",  "bundled_with": "node"},
]

permissions = {"http": ["nodejs.org"]}

# ── 版本获取 ─────────────────────────────────────────────────
def fetch_versions(ctx):
    releases = ctx.http.get_json("https://nodejs.org/dist/index.json")
    return [
        {
            "version": r["version"].lstrip("v"),
            "lts":     r.get("lts", False) != False,
        }
        for r in releases
    ]

# ── 下载 URL ─────────────────────────────────────────────────
def download_url(ctx, version):
    _platforms = {
        "windows/x64":  ("win",    "x64",   "zip"),
        "windows/x86":  ("win",    "x86",   "zip"),
        "macos/x64":    ("darwin", "x64",   "tar.xz"),
        "macos/arm64":  ("darwin", "arm64", "tar.xz"),
        "linux/x64":    ("linux",  "x64",   "tar.xz"),
        "linux/arm64":  ("linux",  "arm64", "tar.xz"),
    }
    p = _platforms.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))
    if not p:
        return None
    os_str, arch_str, ext = p[0], p[1], p[2]
    return "https://nodejs.org/dist/v{}/node-v{}-{}-{}.{}".format(
        version, version, os_str, arch_str, ext
    )

# ── 安装布局 ─────────────────────────────────────────────────
def install_layout(ctx, version):
    _platforms = {
        "windows/x64":  ("win",    "x64"),
        "macos/x64":    ("darwin", "x64"),
        "macos/arm64":  ("darwin", "arm64"),
        "linux/x64":    ("linux",  "x64"),
        "linux/arm64":  ("linux",  "arm64"),
    }
    p = _platforms.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))
    if not p:
        return {"type": "archive"}
    os_str, arch_str = p[0], p[1]
    return {
        "type":         "archive",
        "strip_prefix": "node-v{}-{}-{}".format(version, os_str, arch_str),
    }

# ── 环境变量 ─────────────────────────────────────────────────
def environment(ctx, version, install_dir):
    if ctx.platform.os == "windows":
        return {"PATH": install_dir}
    return {"PATH": install_dir + "/bin"}

# ── 安装后处理 ───────────────────────────────────────────────
def post_install(ctx, version, install_dir):
    if ctx.platform.os == "windows":
        return []
    return [set_permissions("bin/" + t, "755") for t in ["node", "npm", "npx", "corepack"]]

# ── 执行前钩子 ───────────────────────────────────────────────
def pre_run(ctx, args):
    if args and args[0] in ("run", "run-script"):
        return [ensure_dependencies("npm", check_file="package.json", install_dir="node_modules")]
    return []
```

### uv provider（迁移前 → 迁移后）

**迁移前**（当前 147 行）：

```python
_p = make_github_provider("astral-sh", "uv", "uv-{triple}.{ext}")
fetch_versions = _p["fetch_versions"]
download_url   = _p["download_url"]
```

**迁移后**（约 40 行，减少 73%）：

```python
# provider.star - uv provider

load("@vx//stdlib:github.star", "github_releases", "github_asset")
load("@vx//stdlib:install.star", "ensure_dependencies")

name        = "uv"
description = "An extremely fast Python package installer and resolver"
homepage    = "https://github.com/astral-sh/uv"
repository  = "https://github.com/astral-sh/uv"
license     = "MIT OR Apache-2.0"
ecosystem   = "python"

runtimes = [
    {"name": "uv",  "executable": "uv",  "priority": 100},
    {"name": "uvx", "executable": "uvx", "bundled_with": "uv"},
]

# 直接赋值形式（最简洁）
fetch_versions = github_releases("astral-sh", "uv")
download_url   = github_asset("astral-sh", "uv", "uv-{triple}.{ext}")

def install_layout(ctx, version):
    return {"type": "archive", "strip_prefix": ""}

def environment(ctx, version, install_dir):
    return {"PATH": install_dir}

def pre_run(ctx, args):
    if args and args[0] == "run":
        return [ensure_dependencies("uv", check_file="pyproject.toml", install_dir=".venv")]
    return []
```

---

## stdlib 辅助函数规范

### `@vx//stdlib:github.star`

```python
# 获取 GitHub Releases 版本列表
# 返回可直接赋值给 fetch_versions 的可调用对象
github_releases(owner, repo, prerelease = False)

# 构建 GitHub Release 资产下载 URL
# asset 模板变量：{version}, {triple}, {ext}, {os}, {arch}
# triple 示例：x86_64-pc-windows-msvc, aarch64-apple-darwin
# ext 自动根据平台选择：windows→zip, 其他→tar.gz
github_asset(owner, repo, asset_template)
```

### `@vx//stdlib:install.star`

```python
# 设置文件权限（Unix only，Windows 忽略）
set_permissions(path, mode)
# 示例：set_permissions("bin/mytool", "755")

# 运行命令（用于安装后初始化）
run_command(executable, args = [], env = {}, on_failure = "error")

# 确保依赖已安装（用于 pre_run）
ensure_dependencies(manager, check_file, install_dir)
# 示例：ensure_dependencies("npm", check_file="package.json", install_dir="node_modules")
```

---

## ctx 对象规范

所有函数接收统一的 `ctx` 对象（Starlark struct）：

```python
ctx.platform.os      # "windows" | "linux" | "macos"
ctx.platform.arch    # "x64" | "arm64" | "x86"

ctx.http.get_json(url)   # → dict/list，自动解析 JSON
ctx.http.get_text(url)   # → str

ctx.paths.store_dir      # vx store 根目录
ctx.paths.cache_dir      # vx cache 目录

ctx.cache.get(key)       # → value | None（跨调用缓存）
ctx.cache.set(key, val)  # → None
```

---

## 架构变化

### 消除 `star_to_manifest` 补丁

`vx-manifest/src/loader.rs` 中的 `star_to_manifest` 函数（~300 行）将被删除。

```
改造前：
  ManifestLoader
    ├── load provider.toml → ProviderManifest
    └── load provider.star → star_to_manifest() → ProviderManifest（补丁）

改造后：
  StarlarkRegistry
    └── load provider.star → StarlarkProvider → ProviderHandle
```

### 依赖关系简化

```
改造前：
  vx-manifest ──(star_to_manifest)──▶ 解析 .star 文件
  vx-starlark ──▶ vx-runtime ──▶ vx-manifest（循环风险）

改造后：
  vx-starlark ──▶ 独立处理所有 .star 文件
  vx-runtime  ──▶ vx-starlark（单向依赖）
  vx-manifest ──▶ 仅作内部数据结构（过渡期）
```

---

## 实施路线图

### Phase 1：API 统一（v0.16.0）

- [ ] 将 `ctx["platform"]["os"]` 改为 `ctx.platform.os`（Starlark struct）
- [ ] 将元数据函数（`def name():`）改为顶层变量（`name = "..."`）
- [ ] 统一 `post_extract` / `post_install` → 只保留 `post_install`
- [ ] 更新 `pre_run` 签名：去掉 `executable` 参数（可从 ctx 获取）
- [ ] 实现 `github_releases()` 和 `github_asset()` 直接赋值支持
- [ ] 实现 `runtimes[]` 缺省推断逻辑

### Phase 2：依赖解析引擎（v0.16.0）

- [ ] 实现 `requires` 顶层变量解析（静态依赖声明）
- [ ] 实现 `def requires(ctx, version):` 动态依赖函数
- [ ] 实现版本约束语法解析（`>=`, `<`, `~=`, `!=`, `,` 组合）
- [ ] 实现传递依赖图构建（递归解析所有 provider.star）
- [ ] 实现拓扑排序（确定安装顺序）
- [ ] 实现版本约束求解（选择满足所有约束的最新版本）
- [ ] 实现冲突检测与报错（清晰的冲突说明）
- [ ] 实现弱依赖（`~name>=X` 语法）
- [ ] 实现 `fetch_requires`：安装阶段工具依赖（下载/解压/post_install 时自动准备，安装后丢弃）
- [ ] 实现 `build_requires` 和 `private_build_requires`（从源码编译场景，较低优先级）
- [ ] 扩展 `ctx.resolved` 对象（供 `requires` 函数查询已解析环境）
- [ ] `--with` 支持覆盖自动解析的版本约束
- [ ] `--no-auto-deps` 标志禁用自动依赖解析
- [ ] **[Spack 借鉴]** 实现 `requires[].when` 条件依赖语法（声明式条件，减少函数形式）
- [ ] **[Spack 借鉴]** 实现 `conflicts` 声明（主动声明不兼容配置，给出清晰错误信息）

### Phase 3：内置 Provider 迁移（v0.16.0）

- [ ] 迁移所有内置 Provider 到新 API（预计每个 Provider 减少 40-60% 代码量）
- [ ] 为所有内置 Provider 补充 `requires` 声明
- [ ] 删除所有 `provider.toml` 文件
- [ ] 删除 `star_to_manifest` 函数

### Phase 4：架构清理（v0.17.0）

- [ ] `vx-manifest` 降级为内部数据结构
- [ ] `ProviderHandle` 成为唯一的 Provider 表示
- [ ] 更新文档和迁移指南
- [ ] **[Spack 借鉴]** 实现 `provides` 能力声明（如 bun 可满足 node 依赖）
- [ ] **[Spack 借鉴]** 实现 `variants` 构建变体（如 node +lts / ~lts）
- [ ] **[Spack 借鉴]** `fetch_versions` 支持 `deprecated` 版本标记

---

## 向后兼容性

### 内置 Provider 迁移

所有内置 Provider 在 v0.16.0 完成迁移，用户无感知。

### 自定义 Provider 迁移

提供 `vx provider migrate` 命令自动转换：

```bash
# 将旧格式 provider.star 转换为新格式
vx provider migrate ~/.vx/providers/my-tool/provider.star

# 验证格式
vx provider validate ~/.vx/providers/my-tool/provider.star
```

旧格式（函数形式的元数据）在 v0.16.0 继续支持，v0.17.0 废弃。

---

## 参考资料

- [rez package.py 文档](https://rez.readthedocs.io/en/latest/package_definition.html)
- [RFC 0036: Starlark Provider Support](./0036-starlark-provider-support.md)
- [RFC 0037: Provider.star Unified Facade](./0037-provider-star-unified-facade.md)
- [starlark-rust](https://github.com/facebookexperimental/starlark-rust)

---

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-02-22 | Draft v1 | 初始草案（基于 provider.toml 字段映射） |
| 2026-02-22 | Draft v2 | 重新设计：简洁优先，缺省友好，统一签名 |
| 2026-02-22 | Draft v3 | 新增依赖解析设计：requires 声明、传递依赖图、版本约束求解、--with 关系 |
| 2026-02-22 | Draft v4 | 新增 Spack 借鉴分析：conflicts、provides、variants、when 条件依赖、deprecated 版本标记 |
| 2026-02-22 | Draft v5 | 新增 justfile 风格模板内联与环境变量渲染：ctx.render()、ctx.env()、ctx.shell()、ctx.dotenv()、字符串/版本/路径辅助函数、permissions 安全模型 |
| 2026-02-22 | Draft v5.1 | 新增 fetch_requires：安装阶段工具依赖（区别于运行时 requires 和编译时 build_requires），明确三类依赖语义及与 permissions.exec 的关系 |
