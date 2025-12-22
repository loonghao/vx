# VX CLI 完整参考手册

VX 是一个通用的版本管理工具，提供透明的代理和项目配置功能。本文档详细描述了所有可用的命令和选项。

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

## 🛠️ 工具执行

### 直接执行工具

```bash
# 完全透明的使用体验
vx node --version                    # 自动使用项目配置的版本
vx uv pip install requests          # 自动安装并使用配置的uv版本
vx go build                         # 自动使用项目配置的go版本

# 自动安装并运行（✨ 新功能）
vx uv pip install requests          # 如果uv未安装，自动下载并使用
vx python -c "print('Hello')"       # 如果python未安装，自动安装最新版本
vx npm install                      # 如果npm未安装，自动安装最新版本

# 自动安装过程示例
$ vx node --version
# 🔍 检测到工具 'node' 未安装
# 📦 正在获取最新版本信息...
# ⬇️  正在下载 Node.js v20.10.0...
# 📁 正在安装到 ~/.vx/tools/node/20.10.0/...
# ✅ 安装完成！
# v20.10.0
```

### 执行流程（包含自动安装）

1. 用户运行 `vx node --version`
2. 查找项目配置文件（`.vx.toml`）
3. 解析版本需求（`18.17.0` 或 `latest`）
4. 检查工具是否已安装：
   - ✅ 已安装：直接使用
   - ❌ 未安装：触发自动安装流程
5. **自动安装流程**：
   - 检查自动安装是否启用（默认启用）
   - 获取工具的可用版本列表
   - 选择最新稳定版本（跳过预发布版本）
   - 下载并安装到 `~/.vx/tools/<tool>/<version>/`
   - 验证安装是否成功
6. 透明代理到正确版本执行

### 自动安装配置

```toml
# ~/.vx/config.toml
[auto_install]
enabled = true                    # 启用自动安装
include_prerelease = false        # 是否包含预发布版本
timeout = 300                     # 安装超时时间（秒）
confirm_before_install = false    # 安装前是否需要确认

# 项目级配置 .vx.toml
[auto_install]
enabled = true                    # 项目级别开关
```

## 📦 工具管理

### 安装工具

```bash
# 安装特定版本
vx install node@18.17.0
vx install uv@latest
vx install go@1.21.6

# 安装多个工具
vx install node@18.17.0 uv@latest go@1.21.6

# 强制重新安装
vx install node@18.17.0 --force
```

### 列出工具

```bash
# 列出支持的工具
vx list

# 列出特定工具的版本
vx list node

# 显示安装状态
vx list --status
```

### 更新工具

```bash
# 更新所有工具到最新版本
vx update

# 更新特定工具
vx update node

# 自动应用更新
vx update --apply
```

### 移除工具

```bash
# 移除特定版本
vx remove node@18.17.0

# 移除工具的所有版本
vx remove node --all

# 强制移除（忽略依赖检查）
vx remove node --force
```

### 搜索工具

```bash
# 搜索可用工具
vx search python

# 搜索特定类别
vx search --category python
```

### 切换版本

```bash
# 临时切换版本
vx switch node@20.10.0

# 设为全局默认
vx switch node@20.10.0 --global
```

## 🌍 虚拟环境管理

### 创建虚拟环境

```bash
# 创建空的虚拟环境
vx venv create myproject

# 创建并指定工具
vx venv create myproject --tools node@18.17.0,uv@latest

# 基于当前项目配置创建
vx venv create myproject --from-config
```

### 使用虚拟环境

```bash
# 激活虚拟环境（设置当前shell）
vx venv use myproject

# 在虚拟环境中运行命令
vx venv run myproject node --version

# 在虚拟环境中运行shell
vx venv shell myproject
```

### 管理虚拟环境

```bash
# 列出所有虚拟环境
vx venv list

# 显示当前激活的虚拟环境
vx venv current

# 向虚拟环境添加工具
vx venv add myproject uv@latest

# 从虚拟环境移除工具
vx venv remove-tool myproject uv

# 删除虚拟环境
vx venv remove myproject

# 强制删除
vx venv remove myproject --force
```

## 🌐 全局工具管理

### 全局工具操作

```bash
# 列出全局安装的工具
vx global list

# 显示详细信息
vx global list --verbose

# 显示特定工具信息
vx global info node

# 移除全局工具（仅当无虚拟环境引用时）
vx global remove node

# 强制移除（忽略虚拟环境引用）
vx global remove node --force

# 显示工具的依赖关系
vx global dependents node

# 清理未使用的全局工具
vx global cleanup

# 预览清理操作
vx global cleanup --dry-run
```

## 🔧 项目管理

### 初始化项目

```bash
# 在当前目录初始化vx配置
vx init

# 交互式初始化
vx init --interactive

# 基于模板初始化
vx init --template node
```

### 项目同步

```bash
# 同步安装项目所需的所有工具
vx sync

# 仅检查不安装
vx sync --check

# 强制重新安装所有工具
vx sync --force
```

### 配置管理

```bash
# 显示当前配置
vx config

# 显示配置来源
vx config --sources

# 编辑全局配置
vx config edit

# 编辑项目配置
vx config edit --local
```

## 🧹 维护命令

### 清理操作

```bash
# 清理孤立的包和缓存
vx cleanup

# 预览清理操作
vx cleanup --dry-run

# 清理特定类型
vx cleanup --cache-only
vx cleanup --orphaned-only
```

### 统计信息

```bash
# 显示包统计和磁盘使用
vx stats

# 显示详细统计
vx stats --detailed

# 按工具分组显示
vx stats --by-tool
```

## 🔌 插件管理

### 插件操作

```bash
# 列出所有插件
vx plugin list

# 显示已启用的插件
vx plugin list --enabled

# 按类别过滤
vx plugin list --category python

# 显示插件信息
vx plugin info uv

# 启用插件
vx plugin enable uv

# 禁用插件
vx plugin disable uv

# 搜索插件
vx plugin search python

# 显示插件统计
vx plugin stats
```

## 📝 使用示例

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

### 虚拟环境工作流

```bash
# 1. 创建虚拟环境
vx venv create myproject --tools node@18.17.0,uv@latest

# 2. 激活虚拟环境
vx venv use myproject

# 3. 在环境中工作
node --version  # 使用虚拟环境中的版本
uv --version

# 4. 或者直接在环境中运行命令
vx venv run myproject npm install
```

### 全局工具管理

```bash
# 查看已安装的工具
vx global list

# 清理未使用的工具
vx global cleanup

# 查看工具依赖关系
vx global dependents node
```

## ⚠️ 注意事项

1. **自动安装**: 首次使用工具时会自动下载，可能需要网络连接
2. **版本缓存**: 工具版本信息会缓存，使用 `vx update` 刷新
3. **权限要求**: 某些操作可能需要管理员权限
4. **网络代理**: 支持通过配置文件设置代理
5. **跨平台**: 在Windows、macOS和Linux上行为一致

## � 故障排除

### 常见问题

#### 工具安装失败

```bash
# 检查网络连接
vx --verbose install node@18.17.0

# 清理缓存重试
vx cleanup --cache-only
vx install node@18.17.0 --force

# 使用系统PATH作为后备
vx --use-system-path node --version
```

#### 虚拟环境问题

```bash
# 检查虚拟环境状态
vx venv list
vx venv current

# 重新创建虚拟环境
vx venv remove myproject --force
vx venv create myproject --tools node@18.17.0

# 手动激活虚拟环境
eval "$(vx venv activate myproject)"
```

#### 配置问题

```bash
# 验证配置文件
vx config validate

# 显示配置来源
vx config --sources

# 重置配置
mv ~/.config/vx/config.toml ~/.config/vx/config.toml.backup
vx config init
```

### 调试技巧

#### 启用详细日志

```bash
# 全局启用详细输出
export VX_VERBOSE=true
vx node --version

# 单次命令启用
vx --verbose install node@18.17.0
```

#### 检查工具路径

```bash
# 显示工具实际路径
vx which node
vx which uv

# 显示工具版本信息
vx version node
vx version --all
```

#### 网络问题诊断

```bash
# 测试网络连接
vx test-connection

# 使用代理
export VX_HTTP_PROXY="http://proxy:8080"
vx install node@18.17.0

# 使用镜像源
vx config set registries.node.url "https://npmmirror.com/mirrors/node/"
```

## 🔧 高级用法

### Shell 集成

#### Bash/Zsh 集成

```bash
# 添加到 ~/.bashrc 或 ~/.zshrc
eval "$(vx shell-init)"

# 或手动添加
export PATH="$HOME/.vx/bin:$PATH"
source <(vx completion bash)  # 或 zsh
```

#### Fish Shell 集成

```fish
# 添加到 ~/.config/fish/config.fish
vx shell-init | source
vx completion fish | source
```

#### PowerShell 集成

```powershell
# 添加到 PowerShell 配置文件
Invoke-Expression (vx shell-init)
vx completion powershell | Out-String | Invoke-Expression
```

### 自动补全

```bash
# 安装自动补全
vx completion bash > /etc/bash_completion.d/vx
vx completion zsh > /usr/local/share/zsh/site-functions/_vx
vx completion fish > ~/.config/fish/completions/vx.fish
```

### 钩子脚本

#### 项目钩子

```bash
# .vx/hooks/pre-install
#!/bin/bash
echo "准备安装工具..."

# .vx/hooks/post-install
#!/bin/bash
echo "工具安装完成，运行初始化脚本..."
npm install
```

#### 全局钩子

```bash
# ~/.config/vx/hooks/pre-tool-switch
#!/bin/bash
echo "切换工具版本: $VX_TOOL $VX_OLD_VERSION -> $VX_NEW_VERSION"

# ~/.config/vx/hooks/post-venv-activate
#!/bin/bash
echo "虚拟环境已激活: $VX_VENV_NAME"
```

### 批量操作

#### 批量安装工具

```bash
# 从文件安装
vx install --from-file tools.txt

# tools.txt 内容:
# node@18.17.0
# uv@latest
# go@1.21.6
# rust@1.75.0

# 批量更新
vx update --all --apply
```

#### 批量虚拟环境管理

```bash
# 批量创建虚拟环境
for project in project1 project2 project3; do
    vx venv create $project --tools node@18.17.0,uv@latest
done

# 批量清理
vx venv list --format=names | xargs -I {} vx venv remove {} --force
```

## 🔗 相关文档

- [安装指南](INSTALLATION.md)
- [配置参考](CONFIG_REFERENCE.md)
- [架构设计](architecture.md)
- [插件开发](PLUGIN_DEVELOPMENT.md)
- [故障排除指南](cli/troubleshooting.md)
- [CLI 概览](cli/overview.md)
