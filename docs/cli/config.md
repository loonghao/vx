# vx config - 配置管理

管理VX的全局和项目配置。

## 语法

```bash
vx config [subcommand] [options]
```

## 子命令

- `show` - 显示当前配置（默认）
- `edit` - 编辑配置文件
- `set` - 设置配置项
- `get` - 获取配置项
- `validate` - 验证配置文件
- `init` - 初始化配置文件
- `sources` - 显示配置来源

## vx config show

显示当前有效配置。

### 语法

```bash
vx config [show] [options]
```

### 选项

- `--sources` - 显示配置来源
- `--format <format>` - 输出格式：`toml`, `json`, `yaml`
- `--local` - 仅显示项目配置
- `--global` - 仅显示全局配置

### 示例

```bash
# 显示当前配置
vx config

# 显示配置来源
vx config --sources

# 以JSON格式显示
vx config --format json
```

## vx config edit

编辑配置文件。

### 语法

```bash
vx config edit [options]
```

### 选项

- `--local` - 编辑项目配置（.vx.toml）
- `--global` - 编辑全局配置
- `--editor <editor>` - 指定编辑器

### 示例

```bash
# 编辑全局配置
vx config edit

# 编辑项目配置
vx config edit --local

# 使用指定编辑器
vx config edit --editor vim
```

## vx config set

设置配置项。

### 语法

```bash
vx config set <key> <value> [options]
```

### 选项

- `--local` - 设置项目配置
- `--global` - 设置全局配置（默认）

### 示例

```bash
# 设置全局配置
vx config set defaults.auto_install true
vx config set registries.node.url "https://nodejs.org/dist/"

# 设置项目配置
vx config set tools.node "18.17.0" --local
vx config set settings.auto_install false --local
```

## vx config get

获取配置项。

### 语法

```bash
vx config get <key> [options]
```

### 示例

```bash
# 获取配置项
vx config get defaults.auto_install
vx config get tools.node
vx config get registries.node.url
```

## vx config validate

验证配置文件语法和内容。

### 语法

```bash
vx config validate [options]
```

### 选项

- `--local` - 验证项目配置
- `--global` - 验证全局配置
- `--strict` - 严格模式验证

### 示例

```bash
# 验证所有配置
vx config validate

# 验证项目配置
vx config validate --local

# 严格模式验证
vx config validate --strict
```

## vx config init

初始化配置文件。

### 语法

```bash
vx config init [options]
```

### 选项

- `--local` - 初始化项目配置
- `--global` - 初始化全局配置
- `--template <template>` - 使用模板
- `--interactive` - 交互式初始化

### 示例

```bash
# 初始化项目配置
vx config init --local

# 交互式初始化
vx config init --interactive

# 使用模板
vx config init --template node
```

## 配置文件位置

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

## 配置层次结构

VX 使用分层配置系统，按以下优先级顺序合并：

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

## 配置格式

### 全局配置示例

```toml
[defaults]
auto_install = true
check_updates = true
update_interval = "24h"
cache_duration = "7d"
use_system_path = false

[directories]
install_dir = "~/.vx/tools"
cache_dir = "~/.vx/cache"
config_dir = "~/.vx/config"

[network]
timeout = "30s"
retry_count = 3
user_agent = "vx/0.1.0"

[network.proxy]
http = "http://proxy:8080"
https = "https://proxy:8080"
no_proxy = ["localhost", "127.0.0.1"]

[tools.node]
version = "20.11.0"
install_method = "official"
registry = "https://nodejs.org/dist/"

[registries]
node = "https://nodejs.org/dist/"
python = "https://www.python.org/ftp/python/"
go = "https://golang.org/dl/"
```

### 项目配置示例

```toml
[tools]
node = "18.17.0"          # 精确版本
uv = "latest"             # 最新版本
go = "^1.21.0"            # 语义化版本范围
python = "3.11"           # 主版本

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

## 环境变量

VX 支持通过环境变量覆盖配置：

```bash
# 基本配置
export VX_AUTO_INSTALL=true
export VX_VERBOSE=true
export VX_USE_SYSTEM_PATH=false

# 网络配置
export VX_TIMEOUT="60s"
export VX_RETRY_COUNT=5

# 代理设置
export HTTP_PROXY="http://proxy:8080"
export HTTPS_PROXY="https://proxy:8080"
export NO_PROXY="localhost,127.0.0.1"

# 工具特定
export VX_TOOLS_NODE_VERSION="18.17.0"
export VX_TOOLS_PYTHON_VERSION="3.11"
```

## 故障排除

### 配置文件错误

```bash
# 验证配置文件
vx config validate

# 显示配置来源
vx config --sources

# 重置配置
mv ~/.config/vx/config.toml ~/.config/vx/config.toml.backup
vx config init
```

### 配置不生效

```bash
# 检查配置层次
vx config --sources

# 检查环境变量
env | grep VX_

# 重新加载配置
vx config validate
```

## 相关命令

- [init](./init.md) - 初始化项目
- [sync](./sync.md) - 同步项目配置
- [global](./global.md) - 全局工具管理
