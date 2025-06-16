# VX CLI 命令参考

VX 是一个通用的版本管理工具，提供透明的代理和项目配置功能。

## 🚀 核心概念

### 透明代理
用户只需要 `vx <tool>` 就能自动处理一切：
- 自动检测项目配置（`.vx.toml`）
- 自动安装缺失工具
- 自动使用正确版本
- 透明代理到正确版本执行

### 项目配置驱动
基于 `.vx.toml` 自动管理工具版本：
```toml
[tools]
node = "18.17.0"      # 精确版本
uv = "latest"         # 最新版本
go = "^1.21.0"        # 语义化版本
python = "3.11"       # 主版本

[settings]
auto_install = true   # 自动安装缺失工具
cache_duration = "7d" # 版本缓存时间
```

### 简化的架构
```
~/.vx/
├── tools/            # 全局工具存储
│   ├── node/
│   │   ├── 18.17.0/
│   │   └── 20.10.0/
│   ├── uv/
│   │   └── 0.1.0/
│   └── go/
│       └── 1.21.6/
└── config/
    └── global.toml
```

## 📋 全局选项

所有命令都支持以下全局选项：

```bash
vx [OPTIONS] [COMMAND]

OPTIONS:
    --use-system-path    使用系统PATH查找工具而非vx管理的版本
    -v, --verbose        启用详细输出和日志
    -h, --help          显示帮助信息
    -V, --version       显示版本信息
```

## 🔗 命令别名

VX 提供常用命令的便捷别名：

```bash
# 工具管理别名
vx i <tool>         # vx install <tool>
vx rm <tool>        # vx remove <tool> / vx uninstall <tool>
vx ls               # vx list
vx up               # vx update

# 配置管理别名
vx cfg              # vx config

# 清理别名
vx clean            # vx cleanup (推荐使用)

# 标准命令
vx uninstall        # 卸载工具
vx which            # 显示工具位置
vx versions         # 显示可用版本
vx clean            # 清理系统
```

## 📚 命令分类

### 🛠️ 工具执行
- [execute](./execute.md) - 直接执行工具（透明代理）

### 📦 工具管理
- [install](./install.md) - 安装工具 (别名: `i`)
- [list](./list.md) - 列出工具 (别名: `ls`)
- [update](./update.md) - 更新工具 (别名: `up`)
- [uninstall](./remove.md) - 卸载工具 (别名: `rm`)
- [search](./search.md) - 搜索工具
- [switch](./switch.md) - 切换版本
- [which](./remove.md) - 显示工具位置
- [versions](./remove.md) - 显示可用版本

### 🌍 虚拟环境管理
- [venv](./venv.md) - 虚拟环境管理

### 🌐 全局工具管理
- [global](./global.md) - 全局工具管理

### 🔧 项目管理
- [init](./init.md) - 初始化项目
- [sync](./sync.md) - 项目同步
- [config](./config.md) - 配置管理 (别名: `cfg`)

### 🧹 维护命令
- [clean](./cleanup.md) - 清理系统 (别名: `clean`)
- [stats](./stats.md) - 统计信息

### 🔌 插件管理
- [plugin](./plugin.md) - 插件管理

### 🔧 高级功能
- [shell-integration](./shell-integration.md) - Shell 集成
  - `vx shell init` - 生成Shell初始化脚本
  - `vx shell completions` - 生成Shell补全脚本
- [troubleshooting](./troubleshooting.md) - 故障排除

## 📝 快速开始

### 日常开发工作流
```bash
# 1. 进入项目目录
cd my-project

# 2. 初始化vx配置
vx init

# 3. 编辑.vx.toml指定工具版本
echo '[tools]
node = "18.17.0"
uv = "latest"' > .vx.toml

# 4. 同步安装工具
vx sync

# 5. 直接使用工具（完全透明）
vx node --version
vx uv pip install requests
```

## ⚠️ 注意事项

1. **自动安装**: 首次使用工具时会自动下载，可能需要网络连接
2. **版本缓存**: 工具版本信息会缓存，使用 `vx update` 刷新
3. **权限要求**: 某些操作可能需要管理员权限
4. **网络代理**: 支持通过配置文件设置代理
5. **跨平台**: 在Windows、macOS和Linux上行为一致
