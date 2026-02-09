# 虚拟环境隔离

当存在 `vx.toml` 文件时，vx 会自动提供虚拟环境隔离功能。这确保子进程使用项目配置中指定的确切工具版本，防止全局工具版本影响您的项目。

## 问题描述

当运行 `vx npm run build` 或 `vx just gallery-pack` 等命令时，vx 需要为子进程设置 PATH 环境变量。如果没有项目感知的隔离功能，vx 会使用每个工具的**最新安装版本**，这可能导致：

1. **版本不匹配**：您的项目指定了 `node = "20"`，但实际使用的是全局安装的 `node 24`
2. **依赖损坏**：使用较新 Node.js 编译的 npm 包可能与项目指定的版本不兼容
3. **环境不一致**：不同团队成员因全局安装的工具版本不同而获得不同的行为
4. **难以调试的问题**：由于 Node.js 版本不兼容导致的 `Cannot find module` 等错误

## 解决方案：项目配置优先

当 vx 检测到项目中（或父目录）存在 `vx.toml` 文件时，它会自动优先使用该配置中指定的工具版本。

### 工作原理

1. **检测**：执行任何命令时，vx 从当前目录向上搜索 `vx.toml`
2. **版本选择**：对于 PATH 中的每个工具，vx 使用以下优先级：
   - **首选**：`vx.toml` 中指定的版本（如果已安装）
   - **回退**：最新安装的版本（如果指定版本未安装）
   - **警告**：如果指定版本未安装，vx 会发出警告并建议安装

### 示例

假设有以下 `vx.toml`：

```toml
[tools]
node = "20"
go = "1.21"
uv = "latest"
```

以及以下已安装版本：
- Node.js：18.0.0、20.0.0、22.0.0、24.0.0
- Go：1.20.0、1.21.0、1.22.0
- uv：0.4.0、0.5.0

当您运行 `vx npm run build` 时：
- 使用 Node.js **20.0.0**（来自 `vx.toml`）
- 使用 Go **1.21.0**（来自 `vx.toml`）
- 使用 uv **0.5.0**（最新版，按指定）

如果没有 `vx.toml`：
- 将使用 Node.js **24.0.0**（最新版）
- 将使用 Go **1.22.0**（最新版）
- 将使用 uv **0.5.0**（最新版）

## 版本匹配

vx 支持灵活的版本匹配：

### 精确版本
```toml
[tools]
node = "20.10.0"  # 精确匹配 20.10.0
```

### 主版本号
```toml
[tools]
node = "20"  # 匹配最新的 20.x.x（如 20.10.0）
```

### 主.次版本号
```toml
[tools]
node = "20.10"  # 匹配最新的 20.10.x
```

## 配置

### 启用/禁用 PATH 继承

默认情况下，vx 会将所有托管工具传递给子进程。您可以控制此行为：

```toml
[settings]
inherit_vx_path = true  # 默认：启用
```

或通过命令行：

```bash
vx --no-inherit-vx-path npm run build
```

### 严格模式

为了获得最大的可重现性，请使用精确版本：

```toml
[tools]
node = "20.10.0"
go = "1.21.5"
uv = "0.5.0"
```

## 常见用例

### 任务运行器（Just、Make）

使用 `just` 等任务运行器时：

```makefile
# justfile
build:
    npm run build  # 使用 vx.toml 中的 node 版本
    
test:
    uvx pytest     # 使用 vx.toml 中的 uv 版本
```

运行 `vx just build` - 所有工具使用项目指定的版本。

### CI/CD 流水线

```yaml
# .github/workflows/ci.yml
jobs:
  build:
    steps:
      - uses: actions/checkout@v6
      - uses: loonghao/vx-action@v1
      - run: vx setup
      - run: vx npm run build  # 使用 vx.toml 中的版本
```

### Monorepo

每个子目录可以有自己的 `vx.toml`：

```
monorepo/
├── vx.toml           # node = "20"
├── frontend/
│   └── vx.toml       # node = "22"（不同版本）
└── backend/
    └── vx.toml       # node = "18"（另一个版本）
```

当您执行 `cd frontend && vx npm run dev` 时，会使用 node 22。

## 故障排除

### 版本未找到警告

如果您看到：
```
Warning: Version 20 specified in vx.toml for node is not installed.
Using latest installed version instead.
Run 'vx install node@20' to install the specified version.
```

运行建议的命令来安装缺失的版本：
```bash
vx install node@20
```

### 验证当前使用的版本

检查正在使用哪些版本：

```bash
vx node --version
vx npm --version
vx go version
```

### 调试模式

获取版本选择的详细信息：

```bash
VX_LOG=debug vx npm run build
```

## 最佳实践

1. **始终将 `vx.toml` 提交**到版本控制
2. **生产项目使用特定版本**
3. **克隆后运行 `vx setup`** 安装所有指定版本
4. **使用 `vx lock`** 锁定精确版本以确保可重现性
5. **在 README 中记录版本要求**

## 相关文档

- [项目环境](/zh/guide/project-environments) - 完整的项目设置指南
- [版本管理](/zh/guide/version-management) - 管理工具版本
- [vx.toml 参考](/zh/config/vx-toml) - 配置文件参考
