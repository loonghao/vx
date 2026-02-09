# 声明式 Provider（Manifest-Driven Providers）

vx 使用**声明式清单系统**来定义工具及其行为。无需为每个工具编写 Rust 代码，只需创建一个 `provider.toml` 文件来描述 vx 需要了解的关于工具的一切。

## 概述

声明式 Provider 由一个 `provider.toml` 文件定义，包含：

- **Provider 元数据**：名称、描述、主页、生态系统
- **Runtime 定义**：可执行文件、别名、版本源
- **平台配置**：操作系统特定设置、下载 URL
- **依赖关系**：需要或推荐的其他工具
- **检测规则**：如何发现现有安装

这种方式使得：
- 无需编写代码即可添加新工具
- 通过配置自定义工具行为
- 在团队间共享工具定义
- 保持一致的工具管理

## 快速开始

### 使用内置 Provider

vx 内置了 40+ 个流行工具的 Provider，直接使用即可：

```bash
# 这些工具已在 vx 中定义
vx node --version      # Node.js
vx go version          # Go
vx jq --help           # jq JSON 处理器
vx ffmpeg -version     # FFmpeg 媒体工具包
```

### 创建自定义 Provider

要添加新工具，创建一个 `provider.toml` 文件：

```bash
# 创建 provider 目录
mkdir -p ~/.vx/providers/mytool

# 创建清单文件
cat > ~/.vx/providers/mytool/provider.toml << 'EOF'
[provider]
name = "mytool"
description = "我的工具"
homepage = "https://mytool.example.com"
ecosystem = "devtools"

[[runtimes]]
name = "mytool"
description = "我的工具运行时"
executable = "mytool"

[runtimes.versions]
source = "github-releases"
owner = "myorg"
repo = "mytool"

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []
EOF

# 现在可以使用了！
vx mytool --version
```

## Provider 清单格式

### Provider 部分

`[provider]` 部分定义基本元数据：

```toml
[provider]
name = "jq"                                          # 必需：Provider 名称
description = "轻量级命令行 JSON 处理器"              # 必需：描述
homepage = "https://jqlang.github.io/jq/"            # 可选：主页 URL
repository = "https://github.com/jqlang/jq"          # 可选：源代码仓库
ecosystem = "devtools"                               # 可选：生态系统分类
```

**生态系统值：**
- `nodejs` - Node.js 生态系统（npm、yarn 等）
- `python` - Python 生态系统（pip、uv 等）
- `rust` - Rust 生态系统（cargo 等）
- `go` - Go 生态系统
- `system` - 系统工具（ffmpeg、git 等）
- `devtools` - 开发工具（jq、fzf 等）
- `cloud` - 云 CLI 工具（aws、gcloud 等）

### Runtime 部分

每个 `[[runtimes]]` 条目定义一个可执行文件：

```toml
[[runtimes]]
name = "jq"                              # 必需：Runtime 名称
description = "jq JSON 处理器"           # 必需：描述
executable = "jq"                        # 必需：可执行文件名
aliases = ["jqp"]                        # 可选：替代名称
priority = 100                           # 可选：优先级（越高越优先）
auto_installable = true                  # 可选：是否可自动安装（默认：true）
bundled_with = "node"                    # 可选：与另一个 runtime 捆绑
```

### 版本源

`[runtimes.versions]` 部分定义从哪里获取版本信息：

#### GitHub Releases

```toml
[runtimes.versions]
source = "github-releases"
owner = "jqlang"
repo = "jq"
strip_v_prefix = true                    # 从版本标签中移除 'v'
```

#### Node.js 官方

```toml
[runtimes.versions]
source = "nodejs-org"
lts_pattern = "lts/*"
```

#### 系统检测

```toml
[runtimes.versions]
source = "system"                        # 从系统安装检测
```

### 平台配置

定义平台特定设置：

```toml
[runtimes.platforms.windows]
executable_extensions = [".exe", ".cmd"]
search_paths = ["C:\\Program Files\\tool\\bin"]

[runtimes.platforms.unix]
executable_extensions = []
search_paths = ["/usr/bin", "/usr/local/bin"]

[runtimes.platforms.linux]
executable_extensions = []

[runtimes.platforms.macos]
executable_extensions = []
```

### 下载布局

配置下载文件的结构：

#### 二进制下载

用于单文件可执行文件：

```toml
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "jq-windows-amd64.exe"
target_name = "jq.exe"
target_dir = "bin"

[runtimes.layout.binary."linux-x86_64"]
source_name = "jq-linux-amd64"
target_name = "jq"
target_dir = "bin"
target_permissions = "755"
```

#### 归档下载

用于以归档形式分发的工具：

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "node-v{version}-{platform}-{arch}"
executable_paths = [
    "bin/node.exe",   # Windows
    "bin/node"        # Unix
]
```

### 检测配置

定义如何检测现有安装：

```toml
[runtimes.detection]
command = "{executable} --version"
pattern = "v?(\\d+\\.\\d+\\.\\d+)"
system_paths = [
    "/usr/bin/node",
    "/usr/local/bin/node",
    "C:\\Program Files\\nodejs\\node.exe"
]
env_hints = ["NODE_HOME", "NVM_DIR"]
```

### 依赖和约束

定义工具之间的关系：

```toml
# 使用 node 时推荐 npm
[[runtimes.constraints]]
when = "*"
recommends = [
    { runtime = "npm", version = "*", reason = "默认包管理器" }
]

# npm 需要 node
[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=14", recommended = "20", reason = "npm 需要 Node.js" }
]

# 版本特定约束
[[runtimes.constraints]]
when = ">=9"
requires = [
    { runtime = "node", version = ">=14", reason = "npm 9.x+ 需要 Node.js 14+" }
]
```

### 环境变量

配置环境变量：

```toml
[runtimes.env]
vars = { PATH = "{install_dir}/bin" }

# 版本条件环境变量
[runtimes.env.conditional]
">=18" = { NODE_OPTIONS = "--experimental-vm-modules" }
```

**模板变量：**

环境变量值支持以下模板变量：

| 变量 | 描述 | 示例 |
|------|------|------|
| `{install_dir}` | 运行时的安装目录 | `~/.vx/store/python/3.11.0` |
| `{version}` | 运行时版本 | `3.11.0` |
| `{executable}` | 可执行文件路径 | `~/.vx/store/python/3.11.0/bin/python` |

这些变量在运行时被自动展开。

### 镜像配置

为不同地区定义下载镜像：

```toml
[[runtimes.mirrors]]
name = "taobao"
region = "cn"
url = "https://npmmirror.com/mirrors/node"
priority = 100

[[runtimes.mirrors]]
name = "ustc"
region = "cn"
url = "https://mirrors.ustc.edu.cn/node"
priority = 90
```

### 健康检查

定义健康检查命令：

```toml
[runtimes.health]
check_command = "{executable} --version"
expected_pattern = "v\\d+\\.\\d+\\.\\d+"
exit_code = 0
timeout_ms = 5000
check_on = ["install", "activate"]
```

### 下载配置

配置下载行为：

```toml
[runtimes.download]
timeout_ms = 900000           # 大文件 15 分钟
max_retries = 5
resume_enabled = true
execution_timeout_ms = 60000  # 执行超时 1 分钟
```

### 测试配置

为 provider 定义自动化测试：

```toml
[runtimes.test]
timeout_ms = 30000
functional_commands = [
    { command = "{executable} --version", expect_success = true, expected_output = "v\\d+", name = "version_check" },
    { command = "{executable} -e \"console.log('test')\"", expect_success = true, expected_output = "test", name = "eval_test" }
]
install_verification = [
    { command = "{executable} --version", expect_success = true }
]
```

## 实际示例

### 简单二进制工具（jq）

```toml
[provider]
name = "jq"
description = "轻量级灵活的命令行 JSON 处理器"
homepage = "https://jqlang.github.io/jq/"
repository = "https://github.com/jqlang/jq"
ecosystem = "devtools"

[[runtimes]]
name = "jq"
description = "jq - 命令行 JSON 处理器"
executable = "jq"

[runtimes.versions]
source = "github-releases"
owner = "jqlang"
repo = "jq"
strip_v_prefix = true

[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "jq-windows-amd64.exe"
target_name = "jq.exe"
target_dir = "bin"

[runtimes.layout.binary."linux-x86_64"]
source_name = "jq-linux-amd64"
target_name = "jq"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."macos-x86_64"]
source_name = "jq-macos-amd64"
target_name = "jq"
target_dir = "bin"
target_permissions = "755"

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []

[[runtimes.constraints]]
when = "*"
recommends = []
```

### 多 Runtime Provider（Node.js）

```toml
[provider]
name = "node"
description = "基于 Chrome V8 引擎的 JavaScript 运行时"
homepage = "https://nodejs.org"
ecosystem = "nodejs"

# 主 runtime
[[runtimes]]
name = "node"
description = "Node.js 运行时"
executable = "node"
aliases = ["nodejs"]
priority = 100
auto_installable = true

[runtimes.versions]
source = "nodejs-org"
lts_pattern = "lts/*"

[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "node-v{version}-{platform}-{arch}"
executable_paths = ["bin/node.exe", "bin/node"]

[[runtimes.constraints]]
when = "*"
recommends = [
    { runtime = "npm", version = "*", reason = "默认包管理器" }
]

# 捆绑的 npm
[[runtimes]]
name = "npm"
description = "Node 包管理器"
executable = "npm"
bundled_with = "node"

[runtimes.platforms.windows]
executable_extensions = [".cmd", ".exe"]

[runtimes.platforms.unix]
executable_extensions = []

[[runtimes.constraints]]
when = ">=9"
requires = [
    { runtime = "node", version = ">=14", reason = "npm 9.x+ 需要 Node.js 14+" }
]

# 捆绑的 npx
[[runtimes]]
name = "npx"
description = "Node 包执行器"
executable = "npx"
bundled_with = "node"

[runtimes.platforms.windows]
executable_extensions = [".cmd", ".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

### 系统工具（systemctl）

```toml
[provider]
name = "systemctl"
description = "systemd 系统和服务管理器"
homepage = "https://systemd.io"
ecosystem = "system"

# 平台限制
[provider.platforms]
os = ["linux"]

[[runtimes]]
name = "systemctl"
description = "控制 systemd 服务和单元"
executable = "systemctl"
auto_installable = false  # 不能自动安装

[runtimes.versions]
source = "system"

[runtimes.detection]
command = "{executable} --version"
pattern = "systemd ([\\d.]+)"
system_paths = ["/usr/bin/systemctl", "/bin/systemctl"]

[runtimes.platforms.linux]
executable_extensions = []
search_paths = ["/usr/bin", "/bin"]

# 捆绑工具
[[runtimes]]
name = "journalctl"
description = "查看 systemd 日志"
executable = "journalctl"
bundled_with = "systemctl"
auto_installable = false

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "systemctl", version = "*", reason = "journalctl 是 systemd 的一部分" }
]
```

## Provider 目录结构

vx 从多个位置加载 provider：

```
~/.vx/providers/          # 用户定义的 provider（最高优先级）
├── mytool/
│   └── provider.toml
└── custom-node/
    └── provider.toml

$VX_PROVIDERS_PATH/       # 环境变量路径
└── team-tools/
    └── provider.toml

内置 provider             # 最低优先级
```

**加载优先级：**
1. `~/.vx/providers/*/provider.toml`（用户本地，最高）
2. `$VX_PROVIDERS_PATH/*/provider.toml`（环境变量）
3. 内置 provider（最低）

## 最佳实践

### 1. 使用描述性名称

```toml
# 好的
name = "ripgrep"
description = "快速的面向行的搜索工具，递归搜索目录"

# 避免
name = "rg"
description = "搜索工具"
```

### 2. 定义所有平台

```toml
# 支持所有主要平台
[runtimes.layout.binary."windows-x86_64"]
source_name = "tool-windows-amd64.exe"

[runtimes.layout.binary."linux-x86_64"]
source_name = "tool-linux-amd64"

[runtimes.layout.binary."linux-aarch64"]
source_name = "tool-linux-arm64"

[runtimes.layout.binary."macos-x86_64"]
source_name = "tool-darwin-amd64"

[runtimes.layout.binary."macos-aarch64"]
source_name = "tool-darwin-arm64"
```

### 3. 设置适当的超时

```toml
# 小工具（< 10MB）
[runtimes.download]
timeout_ms = 60000  # 1 分钟

# 大工具（> 100MB 如 FFmpeg）
[runtimes.download]
timeout_ms = 900000  # 15 分钟
resume_enabled = true
```

### 4. 记录依赖关系

```toml
[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=14", reason = "需要 Node.js 14+ 以支持 ES 模块" }
]
recommends = [
    { runtime = "npm", version = "*", reason = "推荐用于包管理" }
]
```

### 5. 添加健康检查

```toml
[runtimes.health]
check_command = "{executable} --version"
expected_pattern = "\\d+\\.\\d+\\.\\d+"
timeout_ms = 5000
```

## 故障排除

### Provider 未找到

```bash
# 检查 provider 是否已加载
vx list

# 验证 provider.toml 位置
ls ~/.vx/providers/mytool/provider.toml
```

### 版本检测失败

```bash
# 手动测试检测模式
mytool --version

# 检查检测配置
cat ~/.vx/providers/mytool/provider.toml | grep -A5 "\[runtimes.detection\]"
```

### 下载失败

1. 检查网络连接
2. 验证清单中的下载 URL 格式
3. 尝试增加超时：
   ```toml
   [runtimes.download]
   timeout_ms = 300000
   max_retries = 5
   ```

## 另请参阅

- [Provider 开发指南](../advanced/extension-development.md) - 基于 Rust 的 provider
- [配置参考](../config/vx-toml.md) - 项目配置
- [CLI 命令](../cli/overview.md) - 命令参考
