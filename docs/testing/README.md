# VX Testing Guide

本目录包含 VX 测试相关的文档和改进说明。

## 快速开始

### 1. 检查单个工具是否可用

```bash
# 基本用法
vx check node

# 静默模式（只返回退出码）
vx check systemctl --quiet
if [ $? -eq 0 ]; then
    echo "Available"
else
    echo "Not available"
fi

# 详细信息
vx check go --detailed
```

### 2. 运行完整测试

```bash
# 构建项目
cargo build

# 运行测试（会自动跳过不支持的工具）
pwsh scripts/test-all-providers.ps1

# 保留缓存以便调试
pwsh scripts/test-all-providers.ps1 -KeepCache

# 详细输出
pwsh scripts/test-all-providers.ps1 -Verbose

# 只测试特定 provider
pwsh scripts/test-all-providers.ps1 -Filter "node"
```

### 3. 快速测试 check 命令

```bash
# 快速测试 vx check 功能
pwsh scripts/test-check-command.ps1

# 详细输出
pwsh scripts/test-check-command.ps1 -Verbose
```

## 文档

- **[CHECK_COMMAND.md](CHECK_COMMAND.md)** - `vx check` 命令详细文档
  - 命令用法和参数
  - 退出码说明
  - 使用示例
  - 与其他命令的区别

- **[IMPROVED_TESTING.md](IMPROVED_TESTING.md)** - 改进的测试方案
  - 问题描述
  - 解决方案详解
  - 实现细节
  - 使用场景示例

## 测试脚本

### scripts/test-all-providers.ps1

完整的 provider 测试脚本，测试所有已安装的 providers。

**特性**:
- ✅ 自动跳过不支持的平台
- ✅ 详细的测试报告
- ✅ 支持过滤特定 provider
- ✅ 可选的缓存保留

**用法**:
```powershell
# 基本用法
.\scripts\test-all-providers.ps1

# 参数
-KeepCache      # 不删除临时缓存
-Verbose        # 显示详细输出
-Filter "node"  # 只测试包含 "node" 的 providers
```

### scripts/test-check-command.ps1

快速测试 `vx check` 命令的功能。

**特性**:
- ✅ 测试多个场景（可用工具、不可用工具、平台限制等）
- ✅ 验证退出码
- ✅ 跨平台支持

**用法**:
```powershell
# 基本用法
.\scripts\test-check-command.ps1

# 详细输出
.\scripts\test-check-command.ps1 -Verbose
```

## 测试结果示例

### 改进前的输出

```
=== Testing Provider: systemctl ===
  Runtimes: systemctl, journalctl
  Testing: systemctl
  ✓ vx list
  ✗ vx systemctl --version (exit: 1)
    Error: Cannot auto-install 'systemctl'. Please install it manually.

=== Test Summary ===
Total Tests: 100
Passed: 60
Failed: 40  ← 包含大量预期的失败
Success Rate: 60.00%
```

### 改进后的输出

```
=== Testing Provider: systemctl ===
  Runtimes: systemctl, journalctl
  Testing: systemctl
  ⚠ Runtime 'systemctl' is not available on this platform (skipped)
  Testing: journalctl
  ⚠ Runtime 'journalctl' is not available on this platform (skipped)

=== Test Summary ===
Total Tests: 100
Passed: 60
Failed: 5   ← 只有真正的失败
Skipped: 35 ← 明确标记为跳过
Success Rate: 92.31% (60 / 65 可测试项)
```

## CI/CD 集成

### GitHub Actions 示例

```yaml
name: Test All Providers

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Build vx
        run: cargo build --release
      
      - name: Test all providers
        run: |
          pwsh scripts/test-all-providers.ps1
      
      - name: Upload test report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-report-${{ matrix.os }}
          path: ${{ runner.temp }}/vx-test-*/test-report.json
```

### 本地测试脚本示例

```bash
#!/bin/bash
# local-test.sh - 本地测试所有功能

set -e

echo "Building vx..."
cargo build --release

echo "Testing check command..."
./scripts/test-check-command.ps1

echo "Testing all providers..."
./scripts/test-all-providers.ps1 -KeepCache

echo "✓ All tests completed!"
```

## 故障排除

### 问题：vx binary not found

**解决**:
```bash
# 先构建项目
cargo build

# 或构建 release 版本
cargo build --release
```

### 问题：测试失败率过高

**检查步骤**:
1. 确认是真正的失败还是预期的跳过
2. 查看 Skipped 数量是否合理
3. 检查具体失败的测试项

**示例**:
```powershell
# 运行详细模式
.\scripts\test-all-providers.ps1 -Verbose

# 只测试特定 provider
.\scripts\test-all-providers.ps1 -Filter "node"
```

### 问题：Windows 上无法运行 .ps1 脚本

**解决**:
```powershell
# 设置执行策略
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# 或直接运行
pwsh -ExecutionPolicy Bypass -File .\scripts\test-all-providers.ps1
```

## 相关链接

- [VX 开发指南](../development/GUIDE.md)
- [Provider 开发指南](../development/PROVIDER_GUIDE.md)
- [架构规范](../../docs/architecture/README.md)
