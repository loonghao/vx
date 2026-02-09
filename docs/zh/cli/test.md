# test 命令

测试运行时可用性和 Provider 功能。适合 CI/CD 管道中的验证。

## 语法

```bash
vx test [RUNTIME] [OPTIONS]
```

## 测试模式

### 单运行时测试

```bash
# 测试特定运行时
vx test node
vx test python
vx test go
```

### 所有 Provider 测试

```bash
# 测试所有注册的 provider
vx test --all
```

### 本地 Provider 测试

```bash
# 仅测试本地 manifest provider
vx test --local
```

### 远程扩展测试

```bash
# 测试远程扩展
vx test --extension my-ext
```

## 选项

| 选项 | 描述 |
|------|------|
| `--all` | 测试所有 provider |
| `--local` | 仅测试本地 provider |
| `--extension <NAME>` | 测试指定扩展 |
| `--platform-only` | 仅平台兼容性检查 |
| `--functional` | 功能测试 |
| `--install` | 安装测试 |
| `--installed` | 检查已安装工具 |
| `--system` | 检查系统 PATH |
| `--quiet, -q` | 静默模式 |
| `--json` | JSON 输出 |
| `--verbose, -v` | 详细步骤 |

## 退出码

| 退出码 | 含义 |
|--------|------|
| `0` | 全部测试通过 |
| `1` | 有测试失败 |

## CI/CD 集成

```yaml
# GitHub Actions
- name: 验证工具
  run: vx test --all --quiet

# 或验证特定工具
- name: 验证 Node.js
  run: vx test node
```

## 参见

- [install](./install) - 安装工具
- [list](./list) - 列出工具
- [info](./info) - 系统信息
