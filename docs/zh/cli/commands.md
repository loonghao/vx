# 命令参考

全部 vx 命令的快速参考。点击命令名称查看详细文档。

## 工具管理

### install

安装工具版本。别名：`i`

```bash
vx install <TOOL>[@VERSION] [OPTIONS]
vx install node@22                    # 安装 Node.js 22
vx install python@3.12 uv@latest     # 安装多个工具
vx install "node@^22"                # 语义化版本范围
vx install node@lts                  # LTS 版本
vx install go@1.23 --force           # 强制重新安装
```

[完整文档 →](./install)

### list

列出已安装工具和可用运行时。别名：`ls`

```bash
vx list                    # 列出所有已知运行时
vx list --installed        # 仅显示已安装工具
vx list --status           # 显示安装状态
vx list node               # 显示特定工具的版本
```

[完整文档 →](./list)

### uninstall

卸载已安装的工具版本。

```bash
vx uninstall node@18       # 移除指定版本
vx uninstall node --all    # 移除所有版本
```

### which / where

显示当前活跃工具版本的路径。

```bash
vx which node              # /home/user/.vx/store/node/22.11.0/bin/node
vx which python            # 显示活跃 Python 路径
```

### versions

显示工具的可用版本。

```bash
vx versions node           # 列出所有可用 Node.js 版本
vx versions python         # 列出可用 Python 版本
```

### switch

切换到不同的已安装版本。

```bash
vx switch node 20          # 切换到 Node.js 20
vx switch python 3.11      # 切换到 Python 3.11
```

### search

搜索可用工具。

```bash
vx search lint             # 搜索代码检查工具
vx search python           # 搜索 Python 相关工具
```

### test

测试运行时可用性和 Provider 功能。CI 友好。

```bash
vx test node               # 测试 Node.js 可用性
vx test --all              # 测试所有 provider
vx test --all --json       # CI 用 JSON 输出
```

[完整文档 →](./test)

### global

管理全局安装的包，完全生态系统隔离。别名：`g`

```bash
vx global install typescript       # 全局安装
vx global install pip:httpie       # 使用生态系统前缀安装
vx global list                     # 列出全局包
vx global uninstall typescript     # 卸载
```

[完整文档 →](./global)

---

## 项目管理

### init

为当前项目初始化 `vx.toml` 配置。

```bash
vx init                    # 交互式初始化
vx init --detect           # 自动检测项目工具
```

### add

向 `vx.toml` 添加工具需求。

```bash
vx add node@22             # 添加 Node.js 22
vx add python@3.12         # 添加 Python 3.12
```

### remove

从 `vx.toml` 移除工具。别名：`rm`

```bash
vx remove node             # 移除 Node.js 需求
```

### sync

同步已安装工具与 `vx.toml` 需求。

```bash
vx sync                    # 安装缺失的、移除多余的工具
```

### lock

生成或更新 `vx.lock` 以实现可重现环境。

```bash
vx lock                    # 生成锁文件
vx lock --update           # 更新锁文件
```

### check

检查版本约束和工具可用性。

```bash
vx check                   # 验证所有工具满足约束
```

### bundle

离线开发环境打包。

```bash
vx bundle create           # 从 vx.lock 创建离线包
vx bundle export           # 导出为便携式归档
vx bundle import pkg.tar.gz # 从归档导入
vx bundle status           # 显示包状态
```

### analyze

分析项目依赖、脚本和所需工具。

```bash
vx analyze                 # 分析当前项目
```

---

## 脚本与环境

### run

运行 `vx.toml` 中定义的脚本，支持增强参数传递和变量插值。

```bash
vx run dev                 # 运行 'dev' 脚本
vx run test -- --coverage  # 传递参数给脚本
vx run --list              # 列出可用脚本
vx run test -H             # 显示脚本帮助
```

[完整文档 →](./run)

### dev

进入隔离的开发环境，包含所有项目工具。

```bash
vx dev                     # 交互式 shell
vx dev -c "node -v"       # 运行单个命令
vx dev --export --format github  # CI 导出
vx dev --info              # 显示环境信息
```

[完整文档 →](./dev)

### setup

安装所有项目工具并运行设置钩子。

```bash
vx setup                   # 安装所有项目工具
vx setup --force           # 强制重新安装
vx setup --dry-run         # 预览而不安装
```

[完整文档 →](./setup)

### env

管理项目和全局虚拟环境。

```bash
vx env create my-env --node=22 --python=3.12
vx env use my-env          # 激活环境
vx env list                # 列出所有环境
vx env show                # 显示当前环境
vx env delete my-env       # 删除环境
vx env sync                # 与 vx.toml 同步
```

[完整文档 →](./env)

---

## 配置与 Shell

### config

管理全局和项目配置。别名：`cfg`

```bash
vx config show             # 显示当前配置
vx config init             # 初始化 vx.toml
vx config set key value    # 设置配置值
vx config get key          # 获取配置值
vx config validate         # 验证 vx.toml
vx config edit             # 在编辑器中打开配置
vx config schema           # 生成 JSON Schema
```

[完整文档 →](./config)

### shell

Shell 集成，用于自动切换和补全。

```bash
vx shell init bash         # 生成 bash 初始化脚本
vx shell init zsh          # 生成 zsh 初始化脚本
vx shell completions bash  # 生成补全脚本
```

[完整文档 →](./shell)

---

## 扩展与插件

### ext

管理 vx 扩展。别名：`extension`

```bash
vx ext list                # 列出已安装扩展
vx ext install <URL>       # 从仓库安装
vx ext dev <PATH>          # 链接本地扩展用于开发
vx ext info <NAME>         # 显示扩展详情
vx ext update              # 更新所有扩展
vx ext uninstall <NAME>    # 移除扩展
```

[完整文档 →](./ext)

### x

执行扩展命令。

```bash
vx x my-extension          # 运行扩展默认命令
vx x my-ext cmd --arg      # 运行特定子命令
```

### plugin

管理 Provider 插件。

```bash
vx plugin list             # 列出插件
vx plugin info <NAME>      # 显示插件详情
vx plugin enable <NAME>    # 启用插件
vx plugin disable <NAME>   # 禁用插件
vx plugin search <QUERY>   # 搜索插件
vx plugin stats            # 插件统计
```

[完整文档 →](./plugin)

---

## 系统与维护

### info

显示系统信息、能力和诊断。

```bash
vx info                    # 人类可读信息
vx info --json             # JSON 输出（用于脚本/AI）
vx info --warnings         # 显示构建诊断
```

[完整文档 →](./info)

### metrics

查看执行性能指标和报告。

```bash
vx metrics                 # 显示性能指标
vx metrics --json          # JSON 格式
```

[完整文档 →](./metrics)

### cache

管理下载和版本缓存。

```bash
vx cache info              # 显示缓存统计
vx cache list              # 列出缓存条目
vx cache prune             # 安全清理过期条目
vx cache purge             # 移除所有缓存（破坏性）
vx cache dir               # 显示缓存目录路径
```

### self-update

更新 vx 到最新版本。

```bash
vx self-update             # 更新到最新
vx self-update --check     # 检查更新
```

### version

显示 vx 版本信息。

```bash
vx version                 # 显示版本
vx --version               # 简短形式
```

### hook

管理生命周期钩子。

```bash
vx hook status             # 显示钩子状态
vx hook run pre-commit     # 运行特定钩子
vx hook install            # 安装钩子
```

### services

管理开发服务。

```bash
vx services start          # 启动所有服务
vx services stop           # 停止所有服务
vx services status         # 服务状态
vx services logs           # 查看日志
```

### container

容器和 Dockerfile 管理。

```bash
vx container generate      # 生成 Dockerfile
vx container build         # 构建容器
vx container push          # 推送到注册表
```

### auth

认证管理。

```bash
vx auth login              # 认证
vx auth logout             # 登出
vx auth status             # 显示认证状态
```

### migrate

从旧格式迁移配置和数据。

```bash
vx migrate                 # 运行迁移
```

---

## 隐式包执行

关于无需显式安装即可运行包的详细信息，请参见[隐式包执行](./implicit-package-execution)。
