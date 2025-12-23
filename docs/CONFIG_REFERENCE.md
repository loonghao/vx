# VX 配置参考手册

VX 使用分层配置系统，支持多种配置来源和格式。本文档详细描述了所有配置选项和使用方法。

## 🏗️ 配置层次结构

VX 使用 Figment 实现分层配置，按以下优先级顺序合并：

```
环境变量 (VX_*)              ← 最高优先级
         ↓
项目配置 (.vx.toml)
         ↓
项目检测 (pyproject.toml, Cargo.toml, etc.)
         ↓
用户配置 (~/.config/vx/config.toml)
         ↓
内置默认值                    ← 最低优先级
```

## 📁 配置文件位置

### 全局配置

```bash
# Linux/macOS
~/.config/vx/config.toml

# Windows
%APPDATA%\vx\config.toml
```

### 项目配置

```bash
# 项目根目录
.vx.toml
```

### 自动检测的项目文件

VX 会自动检测以下文件并提取工具版本信息：

- `package.json` (Node.js)
- `pyproject.toml` (Python)
- `Cargo.toml` (Rust)
- `go.mod` (Go)
- `.nvmrc` (Node.js版本)
- `.python-version` (Python版本)

## 🔧 全局配置 (config.toml)

### 完整配置示例

```toml
[defaults]
auto_install = true        # 自动安装缺失的工具
check_updates = true       # 检查更新
update_interval = "24h"    # 更新检查频率
cache_duration = "7d"      # 版本缓存时间
use_system_path = false    # 是否使用系统PATH作为后备

[directories]
install_dir = "~/.vx/tools"    # 工具安装目录
cache_dir = "~/.vx/cache"      # 缓存目录
config_dir = "~/.vx/config"    # 配置目录

[network]
timeout = "30s"           # 网络请求超时
retry_count = 3           # 重试次数
user_agent = "vx/0.1.0"   # User-Agent

[network.proxy]
http = "http://proxy:8080"
https = "https://proxy:8080"
no_proxy = ["localhost", "127.0.0.1"]

[tools.uv]
version = "0.5.26"
install_method = "official"
auto_update = true

[tools.node]
version = "20.11.0"
install_method = "official"
registry = "https://nodejs.org/dist/"

[tools.go]
version = "1.21.6"
install_method = "official"

[tools.rust]
version = "1.75.0"
install_method = "rustup"

[registries.npm]
url = "https://registry.npmjs.org/"
auth_token = "${NPM_TOKEN}"

[registries.pypi]
url = "https://pypi.org/simple/"
trusted_hosts = ["pypi.org"]

[isolation]
level = "project"         # project, global, strict
allow_system_fallback = true

[telemetry]
enabled = false
anonymous = true
endpoint = "https://telemetry.vx.dev"

[plugins]
enabled_plugins = ["uv", "node", "go", "rust"]
plugin_dirs = ["~/.vx/plugins"]
auto_discover = true
```

### 配置选项详解

#### [defaults] 部分

- `auto_install`: 是否自动安装缺失的工具
- `check_updates`: 是否检查工具更新
- `update_interval`: 更新检查间隔（支持: "1h", "24h", "7d"）
- `cache_duration`: 版本信息缓存时间
- `use_system_path`: 是否使用系统PATH作为后备

#### [directories] 部分

- `install_dir`: 工具安装根目录
- `cache_dir`: 缓存文件目录
- `config_dir`: 配置文件目录

#### [network] 部分

- `timeout`: 网络请求超时时间
- `retry_count`: 失败重试次数
- `user_agent`: HTTP请求的User-Agent

#### [network.proxy] 部分

- `http`: HTTP代理地址
- `https`: HTTPS代理地址
- `no_proxy`: 不使用代理的地址列表

#### [tools.*] 部分

每个工具的特定配置：

- `version`: 默认版本
- `install_method`: 安装方法（official, github, custom）
- `auto_update`: 是否自动更新
- `registry`: 自定义下载源

#### [registries.*] 部分

包管理器注册表配置：

- `url`: 注册表URL
- `auth_token`: 认证令牌
- `trusted_hosts`: 信任的主机列表

#### [isolation] 部分

- `level`: 隔离级别
  - `project`: 项目级隔离
  - `global`: 全局共享
  - `strict`: 严格隔离
- `allow_system_fallback`: 是否允许回退到系统工具

#### [telemetry] 部分

- `enabled`: 是否启用遥测
- `anonymous`: 是否匿名发送
- `endpoint`: 遥测数据端点

#### [plugins] 部分

- `enabled_plugins`: 启用的插件列表
- `plugin_dirs`: 插件搜索目录
- `auto_discover`: 是否自动发现插件

## 📋 项目配置 (.vx.toml)

### 基本项目配置

```toml
[tools]
node = "18.17.0"          # 精确版本
uv = "latest"             # 最新版本
go = "^1.21.0"            # 语义化版本范围
python = "3.11"           # 主版本
rust = "~1.75.0"          # 补丁版本范围

[settings]
auto_install = true       # 覆盖全局设置
cache_duration = "1d"     # 项目特定缓存时间

[scripts]
dev = "vx node server.js"
test = "vx uv run pytest"
build = "vx go build -o bin/app"

[env]
NODE_ENV = "development"
PYTHONPATH = "./src"
```

### 版本规范

#### 精确版本

```toml
node = "18.17.0"          # 必须是这个版本
```

#### 语义化版本范围

```toml
go = "^1.21.0"            # >=1.21.0 <2.0.0
rust = "~1.75.0"          # >=1.75.0 <1.76.0
python = ">=3.9,<3.12"    # 范围指定
```

#### 特殊版本标识

```toml
uv = "latest"             # 最新稳定版
node = "lts"              # 最新LTS版本
rust = "beta"             # Beta版本
go = "rc"                 # Release Candidate
```

### 高级项目配置

#### 条件配置

```toml
[tools]
node = "18.17.0"

# 开发环境特定配置
[tools.dev]
node = "20.10.0"          # 开发时使用更新版本

# 生产环境配置
[tools.prod]
node = "18.17.0"          # 生产环境使用稳定版本

# 平台特定配置
[tools.windows]
python = "3.11.0"

[tools.linux]
python = "3.11.5"
```

#### 工具特定配置

```toml
[tools.node]
version = "18.17.0"
registry = "https://registry.npmmirror.com/"
install_args = ["--no-optional"]

[tools.python]
version = "3.11"
implementation = "cpython"  # cpython, pypy
install_args = ["--enable-optimizations"]

[tools.uv]
version = "latest"
features = ["all"]
```

#### 虚拟环境配置

```toml
[venv]
default_tools = ["node@18.17.0", "uv@latest"]
auto_activate = true
path = "./venv"

[venv.env]
NODE_ENV = "development"
PYTHONPATH = "./src"
```

## 🌍 环境变量

### VX 环境变量

```bash
# 覆盖配置目录
export VX_CONFIG_DIR="/custom/config/path"

# 覆盖安装目录
export VX_INSTALL_DIR="/custom/install/path"

# 启用详细日志
export VX_VERBOSE=true

# 禁用自动安装
export VX_AUTO_INSTALL=false

# 设置代理
export VX_HTTP_PROXY="http://proxy:8080"
export VX_HTTPS_PROXY="https://proxy:8080"

# 工具特定版本
export VX_NODE_VERSION="20.10.0"
export VX_UV_VERSION="latest"
export VX_GO_VERSION="1.21.6"

# 网络配置
export VX_TIMEOUT="60s"
export VX_RETRY_COUNT=5
```

### 标准环境变量

VX 也会读取标准的环境变量：

```bash
# 代理设置
export HTTP_PROXY="http://proxy:8080"
export HTTPS_PROXY="https://proxy:8080"
export NO_PROXY="localhost,127.0.0.1"

# 工具特定
export NODE_VERSION="18.17.0"    # Node.js版本
export PYTHON_VERSION="3.11"     # Python版本
export GO_VERSION="1.21.6"       # Go版本
```

## 🔍 配置验证

### 检查配置

```bash
# 显示当前有效配置
vx config

# 显示配置来源
vx config --sources

# 验证配置文件
vx config validate

# 显示特定工具配置
vx config show node
```

### 配置调试

```bash
# 启用详细日志查看配置加载过程
vx --verbose config

# 显示配置合并过程
vx config debug
```

## 📝 配置示例

### 企业环境配置

```toml
# ~/.config/vx/config.toml
[defaults]
auto_install = false      # 企业环境禁用自动安装
check_updates = false     # 禁用更新检查

[network.proxy]
http = "http://corporate-proxy:8080"
https = "https://corporate-proxy:8080"
no_proxy = ["*.internal.com", "localhost"]

[registries.npm]
url = "https://npm.internal.com/"
auth_token = "${NPM_INTERNAL_TOKEN}"

[tools.node]
version = "18.17.0"       # 企业标准版本
registry = "https://nodejs.internal.com/"
```

### 开发团队配置

```toml
# 项目 .vx.toml
[tools]
node = "18.17.0"          # 团队统一版本
uv = "0.5.26"
go = "1.21.6"

[scripts]
dev = "vx node server.js"
test = "vx uv run pytest tests/"
lint = "vx node eslint src/"
build = "vx go build -o dist/app"

[env]
NODE_ENV = "development"
LOG_LEVEL = "debug"
```

### 多环境配置

```toml
# .vx.toml
[tools]
node = "18.17.0"

[tools.dev]
node = "20.10.0"          # 开发环境使用最新版本

[tools.test]
node = "18.17.0"          # 测试环境使用稳定版本

[tools.prod]
node = "18.17.0"          # 生产环境使用稳定版本

[scripts.dev]
start = "vx node --inspect server.js"

[scripts.prod]
start = "vx node server.js"
```

## 🔗 相关文档

- [CLI参考](CLI_REFERENCE.md)
- [安装指南](INSTALLATION.md)
- [架构设计](architecture.md)
- [插件开发](PLUGIN_DEVELOPMENT.md)
