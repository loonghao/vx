# Provider 测试指南

## 快速开始

### 测试单个 Runtime

```bash
# 基本测试
vx test node

# 只检查平台支持（最快）
vx test node --platform-only

# JSON 输出（CI 友好）
vx test node --json
```

### 测试所有 Providers

```bash
# 完整测试
vx test --all

# 只测试平台支持
vx test --all --platform-only

# CI 模式（静默 + JSON）
vx test --all --quiet --json > results.json
```

## 开发新 Provider

### Step 1: 创建 Provider 目录

```bash
mkdir -p crates/vx-providers/my-tool
cd crates/vx-providers/my-tool
```

### Step 2: 创建 provider.toml

```toml
name = "my-tool"
description = "My awesome development tool"
version = "0.1.0"

[[runtimes]]
name = "mytool"
description = "Main tool executable"

# 平台支持
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

# 下载 URLs
[runtimes.download_urls]
windows-x86_64 = "https://example.com/releases/v{version}/mytool-windows-x64.zip"
linux-x86_64 = "https://example.com/releases/v{version}/mytool-linux-x64.tar.gz"
macos-x86_64 = "https://example.com/releases/v{version}/mytool-darwin-x64.tar.gz"
macos-aarch64 = "https://example.com/releases/v{version}/mytool-darwin-arm64.tar.gz"

# 可执行文件路径
[runtimes.bin_paths]
windows = "mytool.exe"
linux = "bin/mytool"
macos = "bin/mytool"
```

### Step 3: 测试 Provider

```bash
# 测试配置是否正确
vx test --local . --verbose

# 检查平台支持
vx test --local . --platform-only

# 详细输出（包括错误信息）
vx test --local . --detailed

# JSON 输出（用于自动化）
vx test --local . --json
```

### Step 4: 验证输出

**成功输出示例：**
```
🧪 Testing local provider: ./crates/vx-providers/my-tool
📋 Validating provider.toml...
✓ Provider: my-tool (My awesome development tool)
✓ Runtimes: 1

--- Testing Runtime: mytool ---
  ✓ mytool - passed

=== Test Summary ===
Total:   1
Passed:  1
Failed:  0
Skipped: 0
```

**失败输出示例：**
```
🧪 Testing local provider: ./crates/vx-providers/my-tool
📋 Validating provider.toml...
✓ Provider: my-tool (My awesome development tool)
✓ Runtimes: 1

--- Testing Runtime: mytool ---
  ⚠ mytool - platform not supported

=== Test Summary ===
Total:   1
Passed:  0
Failed:  0
Skipped: 1
```

## CI/CD 集成

### GitHub Actions

```yaml
name: Test Provider

on:
  push:
    paths:
      - 'crates/vx-providers/**'
  pull_request:
    paths:
      - 'crates/vx-providers/**'

jobs:
  test-providers:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Build vx
        run: cargo build --release
      
      - name: Test all providers
        run: ./target/release/vx test --all --json > test-results.json
      
      - name: Check results
        run: |
          # Parse JSON and check for failures
          if ! jq -e '.failed == 0' test-results.json; then
            echo "Some providers failed tests"
            exit 1
          fi
      
      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: test-results-${{ matrix.os }}
          path: test-results.json
```

### GitLab CI

```yaml
test:providers:
  stage: test
  parallel:
    matrix:
      - OS: ubuntu
      - OS: macos
      - OS: windows
  script:
    - cargo build --release
    - ./target/release/vx test --all --json > results.json
    - |
      # Check for failures
      if [ $(jq '.failed' results.json) -gt 0 ]; then
        echo "Provider tests failed"
        exit 1
      fi
  artifacts:
    paths:
      - results.json
    reports:
      junit: results.json
```

## 测试第三方扩展

### 测试 GitHub 上的 Provider

```bash
# 直接测试
vx test --extension https://github.com/user/vx-provider-foo

# 带详细输出
vx test --extension https://github.com/user/vx-provider-foo --verbose

# JSON 输出
vx test --extension https://github.com/user/vx-provider-foo --json
```

### 本地测试克隆的仓库

```bash
# 克隆仓库
git clone https://github.com/user/vx-provider-foo
cd vx-provider-foo

# 测试
vx test --local .
```

## 常见问题

### Q: 如何只测试特定平台？

A: 使用 `--platform-only` 标志：

```bash
vx test mytool --platform-only
```

这会快速检查当前平台是否支持，无需安装工具。

### Q: 如何在 CI 中使用测试结果？

A: 使用 `--json` 和 `--quiet` 标志：

```bash
vx test --all --quiet --json > results.json
```

然后解析 JSON：

```bash
# 检查是否有失败
if [ $(jq '.failed' results.json) -gt 0 ]; then
  echo "Tests failed"
  exit 1
fi
```

### Q: 如何测试 Provider 的下载 URL 是否正确？

A: 使用 `--install` 标志（未来功能）：

```bash
vx test --local . --install
```

这会尝试下载并安装工具，验证 URL 格式是否正确。

### Q: 如何跳过不支持的平台？

A: 测试框架会自动跳过不支持的平台，并在输出中标记为 "skipped"。

```bash
vx test --all
# ⚠ spack - platform not supported (skipped)
```

### Q: 如何测试所有 Provider 但不安装？

A: 使用 `--platform-only` 标志：

```bash
vx test --all --platform-only
```

这只检查平台支持，不会尝试安装任何工具。

## 最佳实践

1. **开发时频繁测试**
   ```bash
   # 监控文件变化并自动测试
   watchexec -e toml "vx test --local . --quiet"
   ```

2. **提交前完整测试**
   ```bash
   vx test --local . --detailed
   ```

3. **CI 中使用 JSON 输出**
   ```bash
   vx test --all --quiet --json
   ```

4. **本地测试多个平台（Docker）**
   ```bash
   # Linux
   docker run --rm -v $(pwd):/workspace rust:latest \
     bash -c "cd /workspace && cargo build && ./target/debug/vx test --local ."
   ```

5. **使用 Pre-commit Hook**
   ```bash
   # .git/hooks/pre-commit
   #!/bin/sh
   cargo build --release
   ./target/release/vx test --all --quiet || exit 1
   ```

## 相关文档

- [VX Test Framework 设计](../testing/VX_TEST_FRAMEWORK.md)
- [Provider 开发指南](../../docs/Provider开发指南.md)
- [E2E 测试指南](../testing/e2e-testing.md)
