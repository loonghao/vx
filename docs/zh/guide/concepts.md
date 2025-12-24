# 核心概念

理解这些核心概念将帮助你充分利用 vx。

## 工具和运行时

在 vx 术语中：

- **工具**：开发工具，如 Node.js、Python、Go 或 Rust
- **运行时**：工具的特定版本（例如 Node.js 20.0.0）
- **提供者**：知道如何安装和管理特定工具的组件

## 版本存储

vx 维护一个**版本存储**，其中保存所有已安装的工具版本：

```
~/.local/share/vx/
├── store/
│   ├── node/
│   │   ├── 18.19.0/
│   │   └── 20.10.0/
│   ├── go/
│   │   └── 1.21.5/
│   └── uv/
│       └── 0.1.24/
├── envs/
│   ├── default/
│   └── my-project/
└── cache/
```

多个版本可以共存而不会冲突。

## 环境

**环境**是一组协同工作的工具版本：

- **默认环境**：当没有项目配置时使用
- **项目环境**：由项目中的 `.vx.toml` 定义
- **命名环境**：你创建的自定义环境

```bash
# 创建命名环境
vx env create my-env

# 向其中添加工具
vx env add node@20 --env my-env
vx env add go@1.21 --env my-env

# 使用它
vx env use my-env
```

## 自动安装

当你通过 vx 运行工具时，它会自动：

1. 检查工具是否已安装
2. 如果缺失则安装（默认需要用户同意）
3. 运行命令

```bash
# 首次运行 - 自动安装 Node.js
vx node --version
# Installing node@20.10.0...
# v20.10.0

# 后续运行 - 使用缓存版本
vx node --version
# v20.10.0
```

## 版本解析

vx 按以下顺序解析工具版本：

1. **显式版本**：`vx node@18 --version`
2. **项目配置**：当前或父目录中的 `.vx.toml`
3. **全局配置**：`~/.config/vx/config.toml`
4. **最新稳定版**：如果未指定版本

### 版本说明符

```toml
[tools]
node = "20"          # 最新 20.x.x
node = "20.10"       # 最新 20.10.x
node = "20.10.0"     # 精确版本
node = "latest"      # 最新稳定版
node = "lts"         # 最新 LTS（对于 Node.js）
node = "stable"      # 稳定频道（对于 Rust）
```

## Shims 与直接执行

vx 支持两种执行模式：

### 直接执行（推荐）

在命令前加上 `vx`：

```bash
vx node script.js
vx npm install
vx go build
```

### Shim 模式

安装拦截工具命令的 shims：

```bash
# 安装 shims
vx shell init bash >> ~/.bashrc

# 现在可以直接运行
node script.js  # 实际上通过 vx 运行
```

## 项目配置

`.vx.toml` 文件定义项目特定的工具需求：

```toml
[project]
name = "my-project"

[tools]
node = "20"
uv = "latest"

[scripts]
dev = "npm run dev"
test = "npm test"
```

当你进入包含 `.vx.toml` 的目录时，vx 会自动使用这些工具版本。

## 依赖解析

某些工具依赖于其他工具。vx 会自动处理：

- `npm` 需要 `node`
- `cargo` 需要 `rust`
- `uvx` 需要 `uv`

当你运行依赖工具时，vx 会确保先安装父工具。

## 缓存

vx 缓存：

- **下载的归档文件**：避免重复下载
- **版本列表**：减少 API 调用
- **解压的二进制文件**：快速启动

缓存位置：`~/.local/share/vx/cache/`

清除缓存：

```bash
vx clean --cache
```

## 下一步

- [直接执行](/zh/guide/direct-execution) - 使用 vx 完成快速任务
- [项目环境](/zh/guide/project-environments) - 设置项目配置
- [环境管理](/zh/guide/environment-management) - 管理多个环境
