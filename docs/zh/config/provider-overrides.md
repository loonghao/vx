# Provider 覆盖配置

vx 允许你自定义运行时依赖约束，而无需修改内置的 Provider 配置。这在以下场景非常有用：

- 公司有特定的 Node.js 版本要求
- 需要使用与默认不同的版本范围
- 想要为某些工具添加可选依赖

## 工作原理

vx 按以下顺序加载 Provider 配置（后加载的覆盖先加载的）：

1. **内置清单** - vx 自带的默认配置
2. **用户级覆盖** - `~/.vx/providers/*.override.toml`
3. **项目级覆盖** - `<项目>/.vx/providers/*.override.toml`

## 创建覆盖文件

覆盖文件使用简单的 TOML 格式。文件名决定要覆盖哪个 Provider：

```
~/.vx/providers/
├── yarn.override.toml      # yarn 的覆盖配置
├── pnpm.override.toml      # pnpm 的覆盖配置
└── node.override.toml      # node 的覆盖配置
```

## 覆盖文件格式

### 基本结构

```toml
# ~/.vx/providers/yarn.override.toml

# 覆盖主运行时（yarn）的约束
[[constraints]]
when = "^1"  # 当 yarn 版本匹配 ^1（1.x）时
requires = [
    { runtime = "node", version = ">=14, <21" }
]
```

### 版本约束语法

`when` 字段使用标准 semver 语法来匹配运行时版本：

| 语法 | 含义 | 示例 |
|------|------|------|
| `^1.2.3` | 兼容版本 | `>=1.2.3, <2.0.0` |
| `~1.2.3` | 补丁版本 | `>=1.2.3, <1.3.0` |
| `>=1.2.3` | 大于等于 | |
| `<2.0.0` | 小于 | |
| `>=1, <3` | 范围 | |
| `1.2.*` | 通配符 | 匹配 1.2.x |
| `*` | 任意版本 | |

### 依赖定义

`requires` 中的每个依赖可以包含以下字段：

```toml
[[constraints]]
when = "^1"
requires = [
    {
        runtime = "node",           # 必填：运行时名称
        version = ">=14, <21",      # 必填：版本约束
        recommended = "20",         # 可选：推荐版本
        reason = "公司政策",         # 可选：说明原因
        optional = false            # 可选：如果为 true，则非必需
    }
]
```

## 使用示例

### 示例 1：公司 Node.js 政策

公司要求所有项目使用 Node.js 18+：

```toml
# ~/.vx/providers/yarn.override.toml

[[constraints]]
when = "*"  # 所有 yarn 版本
requires = [
    { runtime = "node", version = ">=18", reason = "公司安全政策" }
]
```

### 示例 2：项目特定需求

某个遗留项目需要较旧的 Node.js：

```toml
# <项目>/.vx/providers/yarn.override.toml

[[constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=12, <17", reason = "遗留系统兼容性" }
]
```

### 示例 3：添加可选依赖

将 git 添加为可选依赖：

```toml
# ~/.vx/providers/yarn.override.toml

[[constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=16" },
    { runtime = "git", version = ">=2.0", optional = true }
]
```

### 示例 4：多版本范围

为不同版本设置不同约束：

```toml
# ~/.vx/providers/pnpm.override.toml

[[constraints]]
when = "^7"
requires = [
    { runtime = "node", version = ">=14, <19" }
]

[[constraints]]
when = "^8"
requires = [
    { runtime = "node", version = ">=16, <21" }
]

[[constraints]]
when = ">=9"
requires = [
    { runtime = "node", version = ">=18" }
]
```

### 示例 5：运行时特定覆盖

覆盖 Provider 中特定运行时的约束：

```toml
# ~/.vx/providers/node.override.toml

# 覆盖 npm（与 node 捆绑）的约束
[[runtimes]]
name = "npm"

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=18" }
]
```

## 覆盖行为

### 替换规则

- 覆盖约束会**替换**具有相同 `when` 模式的现有约束
- 新的 `when` 模式会**追加**到现有约束中
- 空的覆盖文件会被忽略

### 优先级

当存在多个覆盖时：

1. 项目级覆盖优先于用户级覆盖
2. 后加载的覆盖会替换先加载的同 `when` 模式约束

## 验证覆盖

检查哪些约束生效：

```bash
# 显示运行时的有效约束
vx info yarn --constraints

# 调试模式显示覆盖来源
VX_DEBUG=1 vx yarn --version
```

## 最佳实践

1. **记录你的覆盖** - 添加注释说明为什么要更改约束
2. **项目级用于特定需求** - 用户级用于通用策略
3. **更改后测试** - 运行 `vx yarn --version` 验证覆盖是否生效
4. **版本控制项目覆盖** - 将 `.vx/providers/` 包含在代码仓库中

## 故障排除

### 覆盖未生效

1. 检查文件名是否与 Provider 名称匹配（如 yarn 对应 `yarn.override.toml`）
2. 确认文件在正确的目录中（`~/.vx/providers/` 或 `<项目>/.vx/providers/`）
3. 检查 TOML 语法错误

### 约束冲突

如果遇到意外行为：

```bash
# 启用调试日志
VX_DEBUG=1 vx yarn install

# 检查已加载的清单
vx debug providers
```

## 相关文档

- [配置指南](/zh/guide/configuration) - 通用配置说明
- [版本管理](/zh/guide/version-management) - vx 如何管理版本
- [Provider 开发](/zh/advanced/plugin-development) - 创建自定义 Provider
