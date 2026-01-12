# VX Test Framework - 通用 Provider 测试框架

## 概述

`vx test` 是一个通用的 Provider 测试框架，支持：

1. **内置 Provider 测试**（E2E）- 测试所有注册的 provider
2. **本地 Provider 测试**（开发）- 测试开发中的 provider
3. **远程 Provider 测试**（扩展）- 测试第三方扩展 provider

## 设计原则

### 统一测试接口

所有测试模式使用相同的测试框架和输出格式，确保：
- CI/CD 可以复用测试逻辑
- 开发者可以在本地验证 provider
- 社区可以测试第三方扩展

### 测试分层

```
┌────────────────────────────────────────┐
│  vx test --all (E2E)                    │  ← CI/CD 完整测试
├────────────────────────────────────────┤
│  vx test --local ./provider (Dev)      │  ← 开发者自测
├────────────────────────────────────────┤
│  vx test --extension <url> (Ext)       │  ← 社区扩展测试
├────────────────────────────────────────┤
│  vx test <runtime> (Runtime)           │  ← 单个 runtime 测试
└────────────────────────────────────────┘
```

## 使用场景

### 1. CI/CD 测试所有 Providers

```bash
# GitHub Actions
- name: Test all providers
  run: vx test --all --json > test-results.json

# 只测试平台支持（快速检查）
- name: Check platform support
  run: vx test --all --platform-only --quiet || exit 1
```

### 2. 开发新 Provider

```bash
# 在 provider 目录下测试
cd crates/vx-providers/my-new-tool
vx test --local . --verbose

# 测试平台支持
vx test --local . --platform-only

# 详细输出
vx test --local . --detailed
```

### 3. 测试第三方扩展

```bash
# 测试 GitHub 上的 provider
vx test --extension https://github.com/user/vx-provider-foo

# 测试后自动缓存到 ~/.vx/cache/extensions/
```

### 4. 测试单个 Runtime

```bash
# 基本测试
vx test yarn

# 平台检查
vx test yarn --platform-only

# 功能测试
vx test yarn --functional

# JSON 输出
vx test yarn --json
```

## 命令选项

### 测试目标

| 选项 | 说明 | 示例 |
|------|------|------|
| `<runtime>` | 测试指定 runtime | `vx test node` |
| `--all` | 测试所有 provider | `vx test --all` |
| `--local <path>` | 测试本地 provider | `vx test --local ./my-provider` |
| `--extension <url>` | 测试远程扩展 | `vx test --extension https://...` |

### 测试模式

| 选项 | 说明 | 用途 |
|------|------|------|
| `--platform-only` | 只测试平台支持 | 快速检查，无需安装 |
| `--functional` | 运行功能测试 | 执行 `--version` 等命令 |
| `--install` | 测试安装流程 | 验证下载和安装 |
| `--installed` | 检查 vx store 安装 | 只检查 ~/.vx/store |
| `--system` | 检查系统 PATH | 只检查系统安装 |

### 输出控制

| 选项 | 说明 | 用途 |
|------|------|------|
| `--quiet` | 静默模式 | CI/CD，只返回退出码 |
| `--json` | JSON 输出 | CI/CD，结构化数据 |
| `--verbose` | 详细输出 | 调试，显示所有步骤 |
| `--detailed` | 详细信息 | 显示路径、版本等 |

## 测试流程

### Runtime 测试流程

```
1. 查找 Provider
   ↓
2. 测试平台支持
   ├─ 不支持 → 退出（Exit 1）
   └─ 支持 → 继续
   ↓
3. 检查安装状态
   ├─ vx store
   └─ System PATH
   ↓
4. 功能测试（可选）
   └─ 执行 --version
   ↓
5. 安装测试（可选）
   └─ 测试下载和安装
   ↓
6. 输出结果
   └─ 退出码：0 (通过) / 1 (失败)
```

### Provider 测试流程

```
1. 加载 provider.toml
   ↓
2. 验证配置
   ├─ 名称和描述
   ├─ Runtime 定义
   └─ 平台支持
   ↓
3. 测试每个 Runtime
   ├─ 平台支持
   ├─ 下载 URL 格式
   └─ 版本检测
   ↓
4. 汇总结果
   └─ 总数、通过、失败、跳过
```

## JSON 输出格式

### 单个 Runtime 测试

```json
{
  "runtime": "node",
  "platform_supported": true,
  "vx_installed": true,
  "system_available": false,
  "available": true,
  "passed": true,
  "installed_versions": ["20.0.0", "18.16.0"],
  "functional_test": true
}
```

### 批量测试汇总

```json
{
  "total": 25,
  "passed": 23,
  "failed": 2,
  "skipped": 3,
  "errors": [
    ["unknown-tool", "Provider not found"]
  ],
  "results": [
    {
      "runtime": "node",
      "platform_supported": true,
      "passed": true
    },
    ...
  ]
}
```

## E2E 测试集成

### 项目结构

```
tests/
├── e2e_test_command.rs       # vx test 命令的 E2E 测试
├── e2e_install_tests.rs      # 安装测试
├── e2e_workflow_tests.rs     # 工作流测试
└── ...

测试覆盖：
- ✅ 单个 runtime 测试
- ✅ 批量测试所有 provider
- ✅ 本地 provider 测试
- ✅ 扩展 provider 测试
- ✅ JSON 输出验证
- ✅ 错误处理
- ✅ CI/CD 场景
```

### 运行 E2E 测试

```bash
# 运行所有 E2E 测试
cargo test --test e2e_test_command

# 运行特定测试
cargo test --test e2e_test_command test_single_runtime_platform_check

# 带输出
cargo test --test e2e_test_command -- --nocapture
```

## Provider 开发最佳实践

### 1. 创建 provider.toml

```toml
name = "mytool"
description = "My awesome tool"
version = "0.1.0"

[[runtimes]]
name = "mytool"

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

[runtimes.download_urls]
windows-x86_64 = "https://example.com/mytool-{version}-win-x64.zip"
linux-x86_64 = "https://example.com/mytool-{version}-linux-x64.tar.gz"
macos-x86_64 = "https://example.com/mytool-{version}-darwin-x64.tar.gz"
macos-aarch64 = "https://example.com/mytool-{version}-darwin-arm64.tar.gz"
```

### 2. 测试流程

```bash
# 1. 创建 provider 目录
mkdir -p crates/vx-providers/mytool
cd crates/vx-providers/mytool

# 2. 编写 provider.toml
vim provider.toml

# 3. 测试配置
vx test --local . --verbose

# 4. 测试平台支持
vx test --local . --platform-only

# 5. 提交前测试
vx test --local . --json > test-results.json
```

### 3. CI 集成

```yaml
# .github/workflows/test-provider.yml
name: Test Provider

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - name: Install vx
        run: |
          curl -fsSL https://vx.dev/install.sh | sh
      
      - name: Test provider
        run: |
          vx test --local ./crates/vx-providers/mytool --json > results.json
          
      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: test-results-${{ matrix.os }}
          path: results.json
```

## 扩展生态系统

### 发布第三方 Provider

```bash
# 1. 创建 GitHub 仓库
# 2. 添加 provider.toml
# 3. 测试
vx test --local . --all

# 4. 用户可以直接测试
vx test --extension https://github.com/user/vx-provider-mytool
```

### Provider 注册表（未来）

```bash
# 搜索社区 provider
vx search mytool --extensions

# 安装并测试
vx extension install mytool
vx test mytool
```

## 总结

`vx test` 提供了一个统一的测试框架，使得：

1. **CI/CD** 可以自动化测试所有 provider
2. **开发者** 可以快速验证新 provider
3. **社区** 可以贡献和测试第三方扩展
4. **用户** 可以在安装前验证工具可用性

这个设计实现了本地开发、CI 测试和生产使用的无缝衔接。
