# Starlark Providers - 高级指南

本指南介绍 Starlark Provider 的高级功能，包括多运行时 Provider、自定义版本源、系统集成和扩展模式。

## 概述

Starlark Provider 提供了超越基本工具安装的强大功能：

| 功能 | 描述 |
|------|------|
| **多运行时 Provider** | 一个 Provider 管理多个工具 |
| **自定义版本源** | 从任意 API 获取版本信息 |
| **系统集成** | 检测并使用系统安装的工具 |
| **安装后钩子** | 安装后运行自定义设置 |
| **动态依赖** | 基于版本的依赖解析 |
| **Shell 集成** | 定义 Shell 特定行为 |

## 多运行时 Provider

### 什么是多运行时 Provider？

多运行时 Provider 在单个 Provider 下管理多个相关工具。例如，`shell-tools` Provider 包含 `starship`、`atuin`、`yazi` 等 Shell 工具。

### 结构

```python
# provider.star - 多运行时 Provider 示例
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

name        = "shell-tools"
description = "现代 Shell 工具集合"
homepage    = "https://github.com/vx-dev/shell-tools"
license     = "MIT"
ecosystem   = "devtools"

# 定义多个运行时
runtimes = [
    {
        "name":        "starship",
        "executable":  "starship",
        "description": "跨 Shell 提示符",
        "aliases":     [],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":        "atuin",
        "executable":  "atuin",
        "description": "神奇的 Shell 历史记录",
        "aliases":     [],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":        "yazi",
        "executable":  "yazi",
        "description": "极速终端文件管理器",
        "aliases":     ["ya"],  # ya 是 CLI 辅助工具
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

# 每个运行时的仓库映射
_RUNTIME_REPOS = {
    "starship": ("starship", "starship"),
    "atuin":    ("atuinsh",  "atuin"),
    "yazi":     ("sxyazi",   "yazi"),
}
```

### 按运行时名称分发

多运行时 Provider 的关键是基于 `ctx.runtime_name` 进行分发：

```python
def fetch_versions(ctx):
    """获取特定运行时的版本。"""
    runtime = ctx.runtime_name  # "starship", "atuin", 或 "yazi"
    owner, repo = _RUNTIME_REPOS.get(runtime, (None, None))
    if not owner:
        return []

    # 使用标准库辅助函数
    return make_fetch_versions(owner, repo)(ctx)

def download_url(ctx, version):
    """获取特定运行时的下载 URL。"""
    runtime = ctx.runtime_name
    owner, repo = _RUNTIME_REPOS.get(runtime, (None, None))
    if not owner:
        return None

    os, arch = ctx.platform.os, ctx.platform.arch

    # 构建平台特定的资源名称
    if runtime == "starship":
        triples = {
            "windows/x64":  "x86_64-pc-windows-msvc",
            "macos/x64":    "x86_64-apple-darwin",
            "macos/arm64":  "aarch64-apple-darwin",
            "linux/x64":    "x86_64-unknown-linux-gnu",
            "linux/arm64":  "aarch64-unknown-linux-gnu",
        }
    elif runtime == "atuin":
        triples = {
            "windows/x64":  "x86_64-pc-windows-msvc",
            "macos/x64":    "x86_64-apple-darwin",
            "macos/arm64":  "aarch64-apple-darwin",
            "linux/x64":    "x86_64-unknown-linux-gnu",
            "linux/arm64":  "aarch64-unknown-linux-gnu",
        }
    elif runtime == "yazi":
        triples = {
            "windows/x64":  "x86_64-pc-windows-msvc",
            "macos/x64":    "x86_64-apple-darwin",
            "macos/arm64":  "aarch64-apple-darwin",
            "linux/x64":    "x86_64-unknown-linux-gnu",
            "linux/arm64":  "aarch64-unknown-linux-gnu",
        }
    else:
        return None

    triple = triples.get(f"{os}/{arch}")
    if not triple:
        return None

    ext = "zip" if os == "windows" else "tar.gz"
    asset = f"{runtime}-{version}-{triple}.{ext}"

    return github_asset_url(owner, repo, f"v{version}", asset)

def install_layout(ctx, version):
    """特定运行时的安装布局。"""
    runtime = ctx.runtime_name
    os = ctx.platform.os
    exe = f"{runtime}.exe" if os == "windows" else runtime

    return {
        "type":             "archive",
        "strip_prefix":     f"{runtime}-{version}",
        "executable_paths": [exe, runtime],
    }

def store_root(ctx):
    """每个运行时都有自己的存储目录。"""
    runtime = ctx.runtime_name
    return f"{ctx.vx_home}/store/{runtime}"

def get_execute_path(ctx, version):
    """可执行文件路径。"""
    os = ctx.platform.os
    runtime = ctx.runtime_name
    exe = f"{runtime}.exe" if os == "windows" else runtime
    return f"{ctx.install_dir}/{exe}"

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

### 使用多运行时 Provider

```bash
# 每个运行时都可以直接访问
vx starship --version
vx atuin --version
vx yazi --version

# 列出 Provider 中的所有运行时
vx list shell-tools

# 安装特定运行时
vx install starship@1.20.0
```

## 自定义版本源

### 非 GitHub 版本源

对于不在 GitHub 上的工具，实现自定义 `fetch_versions`：

```python
load("@vx//stdlib:http.star", "http_get")

def fetch_versions(ctx):
    """从自定义 API 获取版本。"""
    # 示例：Go 官方 API
    url = "https://go.dev/dl/?mode=json"
    response = http_get(ctx, url)

    versions = []
    for release in response:
        version = release.get("version", "").lstrip("go")
        if version:
            versions.append({
                "version": version,
                "stable":  release.get("stable", True),
                "date":    release.get("published", ""),
                "lts":     False,
            })

    return versions
```

### PyPI 版本源

```python
load("@vx//stdlib:http.star", "http_get")

def fetch_versions(ctx):
    """从 PyPI 获取版本。"""
    package = "ruff"  # 你的包名
    url = f"https://pypi.org/pypi/{package}/json"

    response = http_get(ctx, url)
    releases = response.get("releases", {})

    versions = []
    for version, files in releases.items():
        # 跳过预发布版本
        is_prerelease = any(c.isdigit() and i > 0 and version[i-1] in 'abrc'
                           for i, c in enumerate(version))

        versions.append({
            "version": version,
            "stable":  not is_prerelease,
            "date":    files[0].get("upload_time", "") if files else "",
            "lts":     False,
        })

    # 按语义版本排序（最新优先）
    return sorted(versions, key=lambda v: v["version"], reverse=True)
```

### Node.js 官方 API

```python
load("@vx//stdlib:http.star", "http_get")

def fetch_versions(ctx):
    """从 Node.js 官方 API 获取版本。"""
    url = "https://nodejs.org/dist/index.json"
    response = http_get(ctx, url)

    versions = []
    for release in response:
        version = release.get("version", "").lstrip("v")
        versions.append({
            "version": version,
            "stable":  release.get("lts", False) is False,
            "date":    release.get("date", ""),
            "lts":     release.get("lts", False) is not False,
        })

    return versions
```

## 系统集成

### 检测系统安装的工具

使用 `system_install` 定义回退策略：

```python
def system_install(ctx):
    """定义如何通过系统包管理器安装。"""
    os = ctx.platform.os

    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "Microsoft.MyTool", "priority": 95},
                {"manager": "choco",  "package": "mytool",            "priority": 80},
                {"manager": "scoop",  "package": "mytool",            "priority": 70},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "mytool", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "apt",  "package": "mytool", "priority": 85},
                {"manager": "dnf",  "package": "mytool", "priority": 80},
                {"manager": "snap", "package": "mytool", "priority": 75},
            ],
        }

    return {}
```

### 平台约束

限制工具只能在特定平台上运行：

```python
# Provider 级别的平台约束
platforms = {
    "os": ["windows", "linux"]  # macOS 上不可用
}

# 或者在 runtimes 列表中
runtimes = [
    {
        "name":        "mytool",
        "executable":  "mytool",
        "platform_os": ["windows"],  # 仅限 Windows
        # ...
    },
]
```

### 系统路径检测

定义在哪里查找系统安装的工具：

```python
runtimes = [
    {
        "name":        "mytool",
        "executable":  "mytool",
        "system_paths": [
            "/usr/local/bin/mytool",
            "/usr/bin/mytool",
            "C:\\Program Files\\MyTool\\mytool.exe",
        ],
        "env_hints": ["MYTOOL_HOME"],  # 检查 MYTOOL_HOME/bin
        # ...
    },
]
```

## 安装后钩子

### 运行设置命令

```python
def post_install(ctx, version):
    """安装后运行设置。"""
    return {
        "commands": [
            {
                "executable": f"{ctx.install_dir}/mytool",
                "args":       ["init", "--install"],
                "cwd":        ctx.install_dir,
            },
        ],
    }
```

### 创建符号链接

```python
def post_install(ctx, version):
    """安装后创建符号链接。"""
    os = ctx.platform.os

    if os != "windows":  # Windows 上符号链接需要管理员权限
        return {
            "symlinks": [
                {
                    "source": f"{ctx.install_dir}/bin/tool",
                    "target": f"{ctx.vx_home}/bin/tool",  # 全局 shim
                },
            ],
        }

    return None
```

### 设置权限

```python
def post_install(ctx, version):
    """在 Unix 上设置可执行权限。"""
    os = ctx.platform.os

    if os != "windows":
        return {
            "permissions": [
                {
                    "path": f"{ctx.install_dir}/mytool",
                    "mode": "755",
                },
            ],
        }

    return None
```

## 动态依赖

### 基于版本的依赖

```python
def deps(ctx, version):
    """根据版本返回依赖。"""
    import re

    # 解析主版本号
    match = re.match(r"(\d+)", version)
    major = int(match.group(1)) if match else 0

    if major >= 20:
        return [
            {"runtime": "node", "version": ">=20", "reason": "需要 Node.js 20+ API"},
        ]
    elif major >= 18:
        return [
            {"runtime": "node", "version": ">=18", "reason": "需要 Node.js 18+ API"},
        ]
    else:
        return [
            {"runtime": "node", "version": ">=16", "reason": "最低 Node.js 16"},
        ]
```

### 可选依赖

```python
def deps(ctx, version):
    """必需和推荐的依赖。"""
    return [
        # 必需依赖
        {"runtime": "node", "version": ">=18", "reason": "运行时依赖", "optional": False},

        # 可选但推荐
        {"runtime": "npm", "version": "*", "reason": "用于包管理", "optional": True},
        {"runtime": "yarn", "version": "*", "reason": "替代包管理器", "optional": True},
    ]
```

## Shell 集成

### 定义 Shell 行为

对于与 Shell 集成的工具（如 `starship`、`atuin`）：

```python
runtimes = [
    {
        "name":        "myshell",
        "executable":  "myshell",
        "description": "Shell 集成工具",
        "shells": {
            "bash":     "~/.bashrc",
            "zsh":      "~/.zshrc",
            "fish":     "~/.config/fish/config.fish",
            "powershell": "$PROFILE",
        },
        # ...
    },
]
```

### Shell 初始化脚本

```python
def shell_init(ctx, shell):
    """生成 Shell 初始化脚本。"""
    if shell == "bash":
        return 'eval "$(myshell init bash)"'
    elif shell == "zsh":
        return 'eval "$(myshell init zsh)"'
    elif shell == "fish":
        return 'myshell init fish | source'
    elif shell == "powershell":
        return 'Invoke-Expression (&myshell init powershell)'

    return None
```

## 安装布局类型

### 归档（tar.gz, zip）

```python
def install_layout(ctx, version):
    return {
        "type":             "archive",
        "url":              "https://example.com/tool.tar.gz",  # 可选，如果设置了 download_url
        "strip_prefix":     "tool-v1.0.0",  # 从路径中移除此前缀
        "executable_paths": ["bin/tool", "tool"],  # 可能的可执行文件位置
    }
```

### 单二进制

```python
def install_layout(ctx, version):
    os = ctx.platform.os
    exe = "tool.exe" if os == "windows" else "tool"

    return {
        "type":            "binary",
        "url":             "https://example.com/tool-binary",  # 直接二进制下载
        "executable_name": exe,  # 下载的二进制文件名
        "permissions":     "755",  # Unix 权限
    }
```

### MSI（仅 Windows）

```python
def install_layout(ctx, version):
    return {
        "type":             "msi",
        "url":              "https://example.com/installer.msi",
        "executable_paths": ["bin/tool.exe", "tool.exe"],
        "strip_prefix":     "Tool",  # MSI 产品名称前缀
        "extra_args":       ["/quiet", "/norestart"],  # 额外的 MSI 参数
    }
```

### 系统查找（检测系统安装）

```python
def install_layout(ctx, version):
    return {
        "type":         "system_find",
        "executable":   "tool",  # 要查找的可执行文件名
        "system_paths": [
            "/usr/local/bin/tool",
            "C:\\Program Files\\Tool\\tool.exe",
        ],
        "hint":         "通过 'winget install Tool' 或 'brew install tool' 安装",
    }
```

## 测试 Provider 脚本

### 使用 vx test

```bash
# 测试 Provider 脚本
vx test provider.star

# 详细输出
vx test provider.star --verbose

# 测试特定函数
vx test provider.star --function fetch_versions
```

### 检查 Provider 脚本

```bash
# 检查问题
vx lint provider.star

# 自动修复问题
vx lint provider.star --fix
```

## 最佳实践

### 1. 使用标准库函数

优先使用标准库辅助函数而非自定义实现：

```python
# 推荐：使用标准库
load("@vx//stdlib:github.star", "make_fetch_versions")
fetch_versions = make_fetch_versions("owner", "repo")

# 避免：自定义实现
def fetch_versions(ctx):
    # ... 50 行 HTTP 解析代码
```

### 2. 优雅处理平台差异

```python
def download_url(ctx, version):
    os, arch = ctx.platform.os, ctx.platform.arch

    # 提供回退
    key = f"{os}/{arch}"
    triple = PLATFORM_TRIPLES.get(key)

    if not triple:
        # 返回 None 表示不支持该平台
        return None

    # ... 构建 URL
```

### 3. 使用有意义的错误消息

```python
def download_url(ctx, version):
    triple = get_triple(ctx)
    if not triple:
        # 记录为什么不支持该平台
        return None

    if not version:
        fail("download_url 需要 version 参数")

    # ... 其余实现
```

### 4. 保持元数据准确

```python
name        = "mytool"           # 必须与目录名匹配
description = "我的工具"          # 清晰、简洁的描述
homepage    = "https://..."      # 项目网站
repository  = "https://..."      # 源代码位置
license     = "MIT"              # SPDX 标识符
ecosystem   = "devtools"         # 分组类别
```

## 调试

### 启用调试日志

```bash
# 启用详细输出
vx --verbose mytool --version

# 查看 Starlark 执行
VX_DEBUG=starlark vx mytool --version
```

### 测试特定函数

```python
# 添加到 provider.star 用于测试
def _test():
    """用于调试的自测函数。"""
    ctx = {
        "platform": {"os": "linux", "arch": "x64"},
        "version":  "1.0.0",
    }

    print("测试 fetch_versions...")
    versions = fetch_versions(ctx)
    print(f"找到 {len(versions)} 个版本")

    print("测试 download_url...")
    url = download_url(ctx, "1.0.0")
    print(f"下载 URL: {url}")
```

## 相关链接

- [声明式 Provider](./manifest-driven-providers.md) - 基本 Provider 结构
- [Provider 函数参考](./manifest-driven-providers.md#provider-函数参考) - 完整 API 参考
- [标准库参考](./manifest-driven-providers.md#标准库) - 可用的标准库函数
