# Plugin 命令

管理 vx 插件（内部称为 "providers"）。

> **注意**: 在 CLI 中我们使用 "plugin" 以便用户理解。在代码库中，这些被称为 "Providers" - 提供一个或多个运行时的模块。详见 [核心概念](/zh/guide/concepts)。

## 概述

```bash
vx plugin <command>
```

## 命令

### 列出插件

列出所有可用插件：

```bash
vx plugin list

# 仅显示已启用的插件
vx plugin list --enabled

# 按类别筛选
vx plugin list --category devops
```

### 插件信息

显示插件的详细信息：

```bash
vx plugin info node
```

输出：
```
Plugin: node
  Provider: NodeProvider
  Runtimes: node, npm, npx
  Ecosystem: NodeJs
  Description: Node.js JavaScript runtime
```

### 插件统计

显示插件统计信息：

```bash
vx plugin stats
```

输出：
```
Plugin Statistics:
  Total providers: 33
  Total runtimes: 39

  Providers:
    node (3 runtimes)
    go (1 runtimes)
    rust (3 runtimes)
    python (1 runtimes)
    ...
```

### 启用/禁用插件

```bash
# 启用插件
vx plugin enable node

# 禁用插件
vx plugin disable node
```

### 搜索插件

按名称或描述搜索插件：

```bash
vx plugin search python
```

## 插件类别

| 类别 | 示例 |
|------|------|
| **语言运行时** | node, go, rust, java, zig, python |
| **包管理器** | uv, pnpm, yarn, bun |
| **构建工具** | vite, just, task, cmake, ninja |
| **DevOps** | docker, terraform, kubectl, helm |
| **云 CLI** | awscli, azcli, gcloud |
| **代码质量** | pre-commit |

## 参见

- [核心概念](/zh/guide/concepts) - 理解插件和 providers
- [支持的工具](/zh/tools/overview) - 完整的支持工具列表
