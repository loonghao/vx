# 声明式 Provider（Manifest-Driven Providers）

vx 使用 **`provider.star`**（Starlark）作为所有 Provider 逻辑的唯一来源。
无需为每个工具编写 Rust 代码，只需创建一个 `provider.star` 文件来描述 vx 需要的一切：
元数据、版本获取、下载 URL、安装布局、环境变量以及系统包管理器回退。

## 概述

声明式 Provider 使用单一文件：

| 文件 | 用途 |
|------|------|
| `provider.star` | **所有逻辑和元数据** — name、description、下载 URL、安装布局、环境变量、system_install |

这种方式使得：
- 无需编写 Rust 代码即可添加新工具
- 通过 Starlark 脚本自定义工具行为
- 在团队间共享工具定义
- 保持一致的工具管理

## 快速开始

### 使用内置 Provider

vx 内置了 60+ 个流行工具的 Provider：

```bash
vx node --version      # Node.js
vx go version          # Go
vx jq --help           # jq JSON 处理器
vx ffmpeg -version     # FFmpeg 媒体工具包
vx rg --version        # ripgrep
```

### 创建自定义 Provider

在 `~/.vx/providers/mytool/` 中创建 `provider.star` 文件：

```bash
mkdir -p ~/.vx/providers/mytool
```

```python
# ~/.vx/providers/mytool/provider.star
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# 元数据
# ---------------------------------------------------------------------------
name        = "mytool"
description = "我的工具"
homepage    = "https://github.com/myorg/mytool"
repository  = "https://github.com/myorg/mytool"
license     = "MIT"
ecosystem   = "devtools"

runtimes = [
    {
        "name":        "mytool",
        "executable":  "mytool",
        "description": "我的工具运行时",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
]

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# 版本获取
# ---------------------------------------------------------------------------
fetch_versions = make_fetch_versions("myorg", "mytool")

# ---------------------------------------------------------------------------
# 下载 URL
# ---------------------------------------------------------------------------
def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    triples = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }
    triple = triples.get("{}/{}".format(os, arch))
    if not triple:
        return None
    ext   = "zip" if os == "windows" else "tar.gz"
    asset = "mytool-{}-{}.{}".format(version, triple, ext)
    return github_asset_url("myorg", "mytool", "v" + version, asset)

# ---------------------------------------------------------------------------
# 安装布局
# ---------------------------------------------------------------------------
def install_layout(ctx, version):
    triple = _triple(ctx)
    os  = ctx.platform.os
    exe = "rg.exe" if os == "windows" else "rg"
    return {
        "type":             "archive",
        "strip_prefix":     "mytool-{}".format(version),
        "executable_paths": [exe, "mytool"],
    }

def store_root(ctx):
    return ctx.vx_home + "/store/mytool"

def get_execute_path(ctx, version):
    os  = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# 环境变量
# ---------------------------------------------------------------------------
def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

现在可以使用了：

```bash
vx mytool --version
```

## provider.star 结构

### 元数据变量（顶层）

所有元数据声明为**顶层变量**（不是函数）：

```python
name        = "ripgrep"                              # 必需
description = "快速正则搜索工具"                      # 必需
homepage    = "https://github.com/BurntSushi/ripgrep"
repository  = "https://github.com/BurntSushi/ripgrep"
license     = "MIT OR Unlicense"                     # 必需（SPDX 标识符）
ecosystem   = "devtools"                             # 必需
aliases     = ["rg"]                                 # 可选
```

**生态系统值：**
`nodejs`、`python`、`rust`、`go`、`ruby`、`java`、`dotnet`、`devtools`、
`container`、`cloud`、`ai`、`cpp`、`zig`、`system`

### runtimes 列表

定义此 Provider 提供的可执行文件：

```python
runtimes = [
    {
        "name":        "ripgrep",      # Runtime 名称
        "executable":  "rg",           # 实际可执行文件名
        "description": "快速正则搜索工具",
        "aliases":     ["rg"],         # 替代名称
        "priority":    100,
        "test_commands": [
            {
                "command":         "{executable} --version",
                "name":            "version_check",
                "expected_output": "ripgrep \\d+",
            },
        ],
    },
]
```

### permissions

声明 Provider 允许访问的资源：

```python
permissions = {
    "http": ["api.github.com", "github.com"],  # 允许的 HTTP 主机
    "fs":   [],                                 # 允许的文件系统路径
    "exec": [],                                 # 允许调用的可执行文件
}
```

### ctx 对象参考

`ctx` 对象由 vx 运行时注入（对象式访问）：

```python
ctx.platform.os      # "windows" | "macos" | "linux"
ctx.platform.arch    # "x64" | "arm64" | "x86"
ctx.platform.target  # "x86_64-pc-windows-msvc" | "aarch64-apple-darwin" | ...
ctx.install_dir      # "/path/to/install/dir"
ctx.vx_home          # "~/.vx" (VX_HOME)
ctx.version          # 当前安装的版本
```

## 标准库

从 vx stdlib 加载辅助函数：

```python
load("@vx//stdlib:github.star",   "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:platform.star", "is_windows", "is_macos", "platform_triple", "exe_ext")
load("@vx//stdlib:install.star",  "msi_install", "archive_install", "binary_install")
load("@vx//stdlib:env.star",      "env_prepend", "env_set", "env_append")
load("@vx//stdlib:http.star",     "fetch_json_versions")
load("@vx//stdlib:semver.star",   "semver_sort")
```

### github.star

| 函数 | 描述 |
|------|------|
| `make_fetch_versions(owner, repo)` | 返回 GitHub releases 的 `fetch_versions` 函数 |
| `github_asset_url(owner, repo, tag, asset)` | 构建 GitHub release asset URL |
| `make_download_url(owner, repo, template)` | 从 URL 模板返回 `download_url` 函数 |

### platform.star

| 函数 | 描述 |
|------|------|
| `is_windows(ctx)` | Windows 时返回 `True` |
| `is_macos(ctx)` | macOS 时返回 `True` |
| `is_linux(ctx)` | Linux 时返回 `True` |
| `platform_triple(ctx)` | 返回 Rust target triple 字符串 |
| `exe_ext(ctx)` | Windows 返回 `".exe"`，其他返回 `""` |

### install.star

| 函数 | 描述 |
|------|------|
| `archive_install(url, strip_prefix, executable_paths)` | 归档安装描述符 |
| `binary_install(url, executable_name)` | 单二进制安装描述符 |
| `msi_install(url, executable_paths)` | MSI 安装描述符（Windows） |
| `platform_install(ctx, windows_url, macos_url, linux_url, ...)` | 按平台安装 |

### env.star

| 函数 | 描述 |
|------|------|
| `env_prepend(key, value)` | 在 PATH 类变量前追加值 |
| `env_set(key, value)` | 设置环境变量 |
| `env_append(key, value)` | 在变量后追加值 |
| `env_unset(key)` | 取消设置环境变量 |

## Provider 函数参考

### fetch_versions(ctx)

返回可用版本列表。通常从 `make_fetch_versions` 继承：

```python
# 最简单：完全继承
fetch_versions = make_fetch_versions("owner", "repo")

# 自定义：非 GitHub 来源
load("@vx//stdlib:http.star", "fetch_json_versions")

def fetch_versions(ctx):
    return fetch_json_versions(
        ctx,
        "https://go.dev/dl/?mode=json",
        lambda releases: [r["version"].lstrip("go") for r in releases],
    )
```

### download_url(ctx, version)

返回给定版本和平台的下载 URL：

```python
def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    triples = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }
    triple = triples.get("{}/{}".format(os, arch))
    if not triple:
        return None
    ext   = "zip" if os == "windows" else "tar.gz"
    asset = "tool-{}-{}.{}".format(version, triple, ext)
    return github_asset_url("owner", "repo", "v" + version, asset)
```

### install_layout(ctx, version)

返回安装描述符字典：

```python
# 归档布局
def install_layout(ctx, version):
    return {
        "type":             "archive",
        "strip_prefix":     "tool-{}".format(version),
        "executable_paths": ["bin/tool.exe", "bin/tool"],
    }

# 二进制布局
def install_layout(ctx, version):
    os  = ctx.platform.os
    exe = "tool.exe" if os == "windows" else "tool"
    return {
        "type":            "binary",
        "executable_name": exe,
    }
```

| 布局类型 | 必需字段 | 可选字段 |
|---------|---------|---------|
| `"archive"` | `type` | `strip_prefix`、`executable_paths` |
| `"binary"` | `type` | `executable_name`、`source_name`、`permissions` |
| `"msi"` | `type`、`url` | `executable_paths`、`strip_prefix` |

### environment(ctx, version)

返回环境操作**列表**（不是字典）：

```python
load("@vx//stdlib:env.star", "env_prepend", "env_set")

def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir),
        env_set("TOOL_HOME", ctx.install_dir),  # 可选
    ]
```

### store_root / get_execute_path / post_install

所有 Provider 必需的路径查询函数：

```python
def store_root(ctx):
    return ctx.vx_home + "/store/mytool"

def get_execute_path(ctx, version):
    os  = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None  # 无需后置操作时返回 None
```

### system_install(ctx)

返回包管理器回退策略：

```python
def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "Publisher.MyTool", "priority": 95},
                {"manager": "choco",  "package": "mytool",           "priority": 80},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "mytool", "priority": 90},
            ],
        }
    return {}
```

### deps(ctx, version)

返回运行时依赖：

```python
def deps(_ctx, _version):
    return [
        {"runtime": "node", "version": ">=18",
         "reason": "需要 Node.js 运行时"},
    ]
```

## 实际示例

### 标准 GitHub 二进制工具（ripgrep）

```python
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

name        = "ripgrep"
description = "ripgrep (rg) - 递归搜索目录中的正则模式"
homepage    = "https://github.com/BurntSushi/ripgrep"
repository  = "https://github.com/BurntSushi/ripgrep"
license     = "MIT OR Unlicense"
ecosystem   = "devtools"
aliases     = ["rg"]

runtimes = [
    {
        "name":        "ripgrep",
        "executable":  "rg",
        "description": "快速正则搜索工具",
        "aliases":     ["rg"],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "ripgrep \\d+"},
        ],
    },
]

permissions = {"http": ["api.github.com", "github.com"], "fs": [], "exec": []}

fetch_versions = make_fetch_versions("BurntSushi", "ripgrep")

def _triple(ctx):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    return {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    triple = _triple(ctx)
    if not triple:
        return None
    os    = ctx.platform.os
    ext   = "zip" if os == "windows" else "tar.gz"
    asset = "ripgrep-{}-{}.{}".format(version, triple, ext)
    return github_asset_url("BurntSushi", "ripgrep", version, asset)  # 无 'v' 前缀

def install_layout(ctx, version):
    triple = _triple(ctx)
    os  = ctx.platform.os
    exe = "rg.exe" if os == "windows" else "rg"
    return {
        "type":             "archive",
        "strip_prefix":     "ripgrep-{}-{}".format(version, triple) if triple else "",
        "executable_paths": [exe, "rg"],
    }

def store_root(ctx):
    return ctx.vx_home + "/store/ripgrep"

def get_execute_path(ctx, version):
    os  = ctx.platform.os
    exe = "rg.exe" if os == "windows" else "rg"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

### PyPI 包别名（meson）

对于通过 PyPI 分发的工具，使用 `package_alias` 路由到 `uvx`：

```python
name        = "meson"
description = "Meson - 极快且用户友好的构建系统"
homepage    = "https://mesonbuild.com"
repository  = "https://github.com/mesonbuild/meson"
license     = "Apache-2.0"
ecosystem   = "python"
aliases     = ["mesonbuild"]

# RFC 0033：将 `vx meson` 路由到 `vx uvx:meson`
package_alias = {"ecosystem": "uvx", "package": "meson"}

runtimes = [
    {
        "name":        "meson",
        "executable":  "meson",
        "description": "Meson 构建系统",
        "aliases":     ["mesonbuild"],
        "priority":    100,
    },
]

permissions = {"http": ["pypi.org"], "fs": [], "exec": ["uvx", "uv"]}

def download_url(_ctx, _version):
    return None  # 通过 uvx 运行，无需直接下载

def deps(_ctx, _version):
    return [{"runtime": "uv", "version": "*", "reason": "工具通过 uv 安装和运行"}]
```

### 混合 Provider（imagemagick）

Linux 直接下载，Windows/macOS 使用系统包管理器：

```python
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

name        = "imagemagick"
description = "ImageMagick - 图像处理软件"
homepage    = "https://imagemagick.org"
repository  = "https://github.com/ImageMagick/ImageMagick"
license     = "ImageMagick"
ecosystem   = "devtools"
aliases     = ["magick", "convert", "mogrify"]

runtimes = [
    {
        "name":        "imagemagick",
        "executable":  "magick",
        "description": "ImageMagick 图像处理",
        "aliases":     ["magick", "convert", "mogrify"],
        "priority":    100,
    },
]

permissions = {"http": ["api.github.com", "github.com", "imagemagick.org"], "fs": [], "exec": []}

fetch_versions = make_fetch_versions("ImageMagick", "ImageMagick")

def download_url(ctx, version):
    os = ctx.platform.os
    if os == "linux":
        arch   = ctx.platform.arch
        suffix = "x86_64" if arch == "x64" else "aarch64"
        asset  = "ImageMagick--gcc-{}.AppImage".format(suffix)
        return github_asset_url("ImageMagick", "ImageMagick",
                                "refs/tags/" + version, asset)
    return None  # Windows/macOS 使用 system_install

def install_layout(ctx, version):
    os = ctx.platform.os
    if os == "linux":
        return {"type": "binary", "executable_name": "magick", "permissions": "755"}
    return None

def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "ImageMagick.ImageMagick", "priority": 95},
                {"manager": "choco",  "package": "imagemagick",             "priority": 80},
            ],
        }
    elif os == "macos":
        return {"strategies": [{"manager": "brew", "package": "imagemagick", "priority": 90}]}
    return {}

def store_root(ctx):
    return ctx.vx_home + "/store/imagemagick"

def get_execute_path(ctx, version):
    os  = ctx.platform.os
    exe = "magick.exe" if os == "windows" else "magick"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

## provider.toml（仅元数据）

创建 `provider.star` 后，`provider.toml` 只需包含元数据 — **无布局字段**：

```toml
[provider]
name        = "mytool"
description = "我的工具"
homepage    = "https://example.com"
repository  = "https://github.com/myorg/mytool"
ecosystem   = "devtools"
license     = "MIT"
```

不需要 `[runtimes.layout]`、`download_type`、`strip_prefix` — 所有安装逻辑都在 `provider.star` 中。

## Provider 目录结构

vx 从多个位置加载 Provider：

```
~/.vx/providers/          # 用户定义的 Provider（最高优先级）
├── mytool/
│   ├── provider.star     # 所有逻辑（必需）
│   └── provider.toml     # 仅元数据（可选）
└── custom-node/
    ├── provider.star
    └── provider.toml

$VX_PROVIDERS_PATH/       # 环境变量路径
└── team-tools/
    ├── provider.star
    └── provider.toml

内置 Provider             # 最低优先级（crates/vx-providers/*）
```

**加载优先级：**
1. `~/.vx/providers/*/provider.star`（用户本地，最高）
2. `$VX_PROVIDERS_PATH/*/provider.star`（环境变量）
3. 内置 Provider（最低）

## 包别名（npm/PyPI 工具）

对于以 npm 或 PyPI 包形式分发的工具，使用 `package_alias` 通过生态系统的包运行器路由：

| 语法 | 路由到 | 安装器 | 依赖 |
|------|--------|--------|------|
| `vx meson@1.5.0` | `vx uvx:meson@1.5.0` | `UvxInstaller` | `uv` |
| `vx ruff@0.9.0` | `vx uvx:ruff@0.9.0` | `UvxInstaller` | `uv` |
| `vx vite@5.0` | `vx npx:vite@5.0` | `NpmInstaller` | `node` |

```python
# PyPI 工具
package_alias = {"ecosystem": "uvx", "package": "ruff"}

# npm 工具
package_alias = {"ecosystem": "npx", "package": "vite"}
```

## 最佳实践

### 1. 始终声明 license

```python
license = "MIT"          # SPDX 标识符 — 必需
```

被封锁的许可证（AGPL-3.0、SSPL、CC BY-NC）不得集成。

### 2. 覆盖所有主要平台

```python
triples = {
    "windows/x64":  "x86_64-pc-windows-msvc",
    "macos/x64":    "x86_64-apple-darwin",
    "macos/arm64":  "aarch64-apple-darwin",
    "linux/x64":    "x86_64-unknown-linux-musl",
    "linux/arm64":  "aarch64-unknown-linux-gnu",
}
```

### 3. 添加 test_commands

```python
runtimes = [
    {
        "name": "mytool",
        "executable": "mytool",
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    },
]
```

### 4. 系统工具使用 system_install

```python
def system_install(ctx):
    os = ctx.platform.os
    if os == "macos":
        return {"strategies": [{"manager": "brew", "package": "mytool", "priority": 90}]}
    elif os == "windows":
        return {"strategies": [{"manager": "winget", "package": "Org.MyTool", "priority": 95}]}
    return {}
```

### 5. 使用描述性名称

```python
# 好的
name        = "ripgrep"
description = "快速的面向行的搜索工具，递归搜索目录"

# 避免
name        = "rg"
description = "搜索工具"
```

## 故障排除

### Provider 未找到

```bash
# 检查 Provider 是否已加载
vx list

# 验证 provider.star 位置
ls ~/.vx/providers/mytool/provider.star
```

### 版本检测失败

```bash
# 手动测试
mytool --version

# 检查 provider.star 中的 fetch_versions
grep -A5 "fetch_versions" ~/.vx/providers/mytool/provider.star
```

### 下载失败

1. 检查网络连接
2. 验证 `download_url()` 对您的平台返回正确的 URL
3. 手动测试 URL：`curl -I <url>`

### macOS/Windows 上"无下载 URL"

添加带有包管理器回退的 `system_install()`：

```python
def system_install(ctx):
    os = ctx.platform.os
    if os == "macos":
        return {"strategies": [{"manager": "brew", "package": "mytool", "priority": 90}]}
    return {}
```

### 安装后找不到可执行文件

检查 `install_layout()` — 验证 `strip_prefix` 和 `executable_paths` 与实际归档结构匹配。
手动下载归档并检查：

```bash
tar -tzf tool-1.0.0-linux.tar.gz | head -20
```

## 高级主题

更多高级用法，请参阅：

- **[Starlark Providers - 高级指南](./starlark-providers.md)** — 多运行时 Provider、自定义版本源、系统集成和扩展模式

## 另请参阅

- [Provider 开发指南](../advanced/plugin-development.md) — 带自定义 Rust 代码的 Provider
- [配置参考](../config/vx-toml.md) — 项目配置
- [CLI 命令](../cli/overview.md) — 命令参考
