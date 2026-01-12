# vx test

测试运行时可用性和 Provider 功能。专为 CI/CD 集成和 Provider 开发设计。

## 概要

```bash
# 测试单个运行时
vx test <runtime> [选项]

# 测试所有 providers
vx test --all [选项]

# 测试本地 provider（用于开发）
vx test --local <路径> [选项]

# 测试远程扩展
vx test --extension <url> [选项]
```

## 描述

`vx test` 命令提供了一个全面的测试框架，用于：

- **运行时测试** - 验证工具是否可用且正常工作
- **Provider 测试** - 批量验证所有已注册的 providers
- **开发测试** - 在开发过程中测试 providers
- **扩展测试** - 验证第三方 provider 扩展

## 快速开始

### 测试单个运行时

```bash
# 基本可用性检查
vx test node

# 快速平台支持检查（无需安装）
vx test node --platform-only

# 脚本中使用静默模式
if vx test node --quiet; then
    echo "Node.js 可用"
fi
```

### 测试所有 Providers

```bash
# 测试所有已注册的 providers
vx test --all

# CI/CD 使用 JSON 输出
vx test --all --json > results.json

# 仅检查平台支持（最快）
vx test --all --platform-only
```

### 开发时测试

```bash
# 测试本地 provider 目录
cd crates/vx-providers/my-tool
vx test --local . --verbose

# 验证 provider.toml 配置
vx test --local . --platform-only
```

## 选项

### 目标选择

| 选项 | 描述 |
|------|------|
| `<runtime>` | 要测试的运行时名称（如 "node", "go"） |
| `--all` | 测试所有已注册的运行时 |
| `--local <路径>` | 测试本地 provider 目录 |
| `--extension <url>` | 从 URL 测试远程 provider |

### 测试模式

| 选项 | 描述 |
|------|------|
| `--platform-only` | 仅检查平台支持（最快） |
| `--functional` | 运行功能测试（执行 --version） |
| `--install` | 测试安装流程 |
| `--installed` | 检查是否已安装在 vx store |
| `--system` | 检查是否在系统 PATH 中可用 |

### 输出控制

| 选项 | 描述 |
|------|------|
| `-q, --quiet` | 静默模式，仅返回退出码 |
| `--json` | JSON 输出格式 |
| `-v, --verbose` | 显示详细测试步骤 |
| `--detailed` | 显示扩展信息 |

## 退出码

| 代码 | 含义 |
|------|------|
| `0` | 所有测试通过 |
| `1` | 一个或多个测试失败 |

## 示例

### CI/CD 集成

**GitHub Actions:**

```yaml
name: 测试 Providers
on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      
      - name: 构建 vx
        run: cargo build --release
      
      - name: 测试所有 providers
        run: ./target/release/vx test --all --json > results.json
      
      - name: 检查结果
        run: |
          if jq -e '.failed > 0' results.json; then
            echo "部分测试失败"
            exit 1
          fi
```

**脚本中的预检查:**

```bash
#!/bin/bash
# 确保所需工具可用

vx test node --quiet || { echo "需要 Node.js"; exit 1; }
vx test yarn --quiet || { echo "需要 Yarn"; exit 1; }

# 运行你的命令
yarn install
yarn build
```

### Provider 开发

```bash
# 步骤 1: 创建 provider 目录
mkdir -p crates/vx-providers/mytool
cd crates/vx-providers/mytool

# 步骤 2: 创建 provider.toml
cat > provider.toml << 'EOF'
name = "mytool"
description = "我的工具"

[[runtimes]]
name = "mytool"
description = "主可执行文件"

[[runtimes.platforms]]
os = "windows"
arch = "x86_64"

[[runtimes.platforms]]
os = "linux"
arch = "x86_64"

[[runtimes.platforms]]
os = "macos"
arch = "x86_64"
arch_variants = ["aarch64"]
EOF

# 步骤 3: 测试 provider
vx test --local . --verbose

# 步骤 4: 提交前验证
vx test --local . --json
```

### JSON 输出

**单个运行时测试:**

```json
{
  "runtime": "node",
  "passed": true,
  "platform_supported": true,
  "vx_installed": true,
  "system_available": false,
  "available": true,
  "installed_versions": ["20.0.0", "18.16.0"],
  "functional_test": true
}
```

**批量测试汇总:**

```json
{
  "total": 25,
  "passed": 23,
  "failed": 0,
  "skipped": 2,
  "results": [...],
  "errors": []
}
```

## 使用场景

### 1. 验证工具可用性

在运行命令前检查工具是否可用：

```bash
if vx test docker --quiet; then
    docker compose up -d
else
    echo "Docker 不可用"
    exit 1
fi
```

### 2. 跨平台测试

无需安装即可测试平台支持：

```bash
# 快速检查工具是否支持当前平台
vx test spack --platform-only --quiet
echo "退出码: $?"  # 0 = 支持, 1 = 不支持
```

### 3. 批量 Provider 验证

在 CI/CD 中测试所有 providers：

```bash
# 运行测试并保存结果
vx test --all --json > test-results.json

# 解析结果
FAILED=$(jq '.failed' test-results.json)
if [ "$FAILED" -gt 0 ]; then
    echo "❌ $FAILED 个测试失败"
    jq '.errors' test-results.json
    exit 1
fi
echo "✅ 所有测试通过"
```

### 4. 开发工作流

在开发过程中测试 provider 变更：

```bash
# 监听文件变更并测试
watchexec -e toml "vx test --local . --quiet && echo '✅ OK' || echo '❌ 失败'"
```

## 最佳实践

1. **使用 `--platform-only` 进行快速检查** - 无需安装
2. **在 CI/CD 中使用 `--json`** - 便于解析和处理
3. **在脚本中使用 `--quiet`** - 仅检查退出码
4. **使用 `--verbose` 进行调试** - 查看所有测试步骤
5. **提交前本地测试** - `vx test --local .`

## 相关命令

- [`vx install`](./install.md) - 安装运行时
- [`vx list`](./list.md) - 列出已安装的运行时
- [`vx run`](./run.md) - 使用特定运行时版本运行命令

## 另请参阅

- [Provider 开发指南](../advanced/extension-development.md)
- [CI/CD 集成](../guides/github-action.md)
