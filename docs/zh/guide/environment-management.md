# 环境管理

vx 允许你创建和管理多个隔离的环境，用于不同的项目或用途。

## 理解环境

环境是一组协同工作的工具版本。每个环境都是隔离的，所以你可以：

- 在一个环境中使用 Node.js 18
- 在另一个环境中使用 Node.js 20
- 为不同项目使用不同的 Go 版本

## 列出环境

```bash
vx env list
```

输出：

```
Environments:

* default (active)
  project-a
  project-b
```

获取详细信息：

```bash
vx env list --detailed
```

## 创建环境

### 基本创建

```bash
vx env create my-env
```

### 从现有环境克隆

```bash
vx env create new-env --from existing-env
```

### 设为默认

```bash
vx env create my-env --set-default
```

## 向环境添加工具

```bash
# 添加到当前环境
vx env add node@20

# 添加到特定环境
vx env add node@20 --env my-env
vx env add go@1.21 --env my-env
vx env add uv@latest --env my-env
```

## 从环境删除工具

```bash
vx env remove node
vx env remove node --env my-env
```

## 切换环境

### 临时切换

```bash
vx env use my-env
```

这会打印适用于你的 shell 的激活指令。

### 设为全局默认

```bash
vx env use my-env --global
```

## 显示环境详情

```bash
# 显示当前环境
vx env show

# 显示特定环境
vx env show my-env
```

输出：

```
Environment: my-env
Path: /home/user/.local/share/vx/envs/my-env
Active: yes

Runtimes:
  node -> /home/user/.local/share/vx/store/node/20.10.0
  go -> /home/user/.local/share/vx/store/go/1.21.5
```

## 导出环境

导出环境配置以便共享：

```bash
# 导出为 TOML（默认）
vx env export my-env -o my-env.toml

# 导出为 JSON
vx env export my-env -o my-env.json --format json

# 导出为 YAML
vx env export my-env -o my-env.yaml --format yaml

# 导出 shell 脚本
vx env export my-env --format shell
```

### 导出格式

```toml
name = "my-env"
exported_at = "2024-01-15T10:30:00Z"

[runtimes]
node = "20.10.0"
go = "1.21.5"
uv = "0.1.24"
```

## 导入环境

从文件导入环境：

```bash
# 使用相同名称导入
vx env import my-env.toml

# 使用不同名称导入
vx env import my-env.toml --name new-env

# 强制覆盖现有
vx env import my-env.toml --force
```

这将：

1. 创建环境
2. 安装任何缺失的工具
3. 将工具添加到环境

## 激活环境

生成 shell 激活脚本：

::: code-group

```bash [Bash/Zsh]
eval "$(vx env activate my-env)"
```

```fish [Fish]
vx env activate my-env --shell fish | source
```

```powershell [PowerShell]
Invoke-Expression (vx env activate my-env --shell powershell)
```

:::

## 删除环境

```bash
# 带确认
vx env delete my-env

# 强制删除
vx env delete my-env --force
```

::: warning
你不能删除 `default` 环境。
:::

## 环境变量

当环境激活时，会设置这些变量：

| 变量 | 描述 |
|------|------|
| `VX_ENV` | 当前环境名称 |
| `VX_ENV_DIR` | 环境目录路径 |
| `PATH` | 更新以包含环境工具 |

## 使用场景

### 每项目环境

```bash
# 为每个项目创建环境
vx env create project-a
vx env add node@18 --env project-a

vx env create project-b
vx env add node@20 --env project-b

# 在处理不同项目时切换
cd project-a && vx env use project-a
cd project-b && vx env use project-b
```

### 测试不同版本

```bash
# 使用不同 Node 版本测试代码
vx env create test-node18
vx env add node@18 --env test-node18

vx env create test-node20
vx env add node@20 --env test-node20

# 在每个环境中运行测试
eval "$(vx env activate test-node18)"
npm test

eval "$(vx env activate test-node20)"
npm test
```

### 共享团队环境

```bash
# 导出团队环境
vx env export team-env -o team-env.toml

# 共享文件（git、邮件等）

# 团队成员导入
vx env import team-env.toml
```

## 最佳实践

1. **使用描述性名称**：`project-name` 或 `purpose-version`
2. **重大更改前导出**：备份你的环境
3. **使用项目配置**：对于项目特定环境，优先使用 `vx.toml`
4. **清理未使用的环境**：`vx env delete old-env`

## 下一步

- [Shell 集成](/zh/guide/shell-integration) - 自动环境激活
- [CLI 参考](/zh/cli/env) - 完整 env 命令参考
