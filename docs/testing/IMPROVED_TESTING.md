# 改进的测试方案

## 问题

原有的测试脚本 `test-all-providers.ps1` 在 Windows 上测试所有 providers 时，会遇到以下问题：

1. **平台不支持**: 某些工具（如 `spack`、`systemctl`）只支持 Linux/macOS，在 Windows 上会失败
2. **无法自动安装**: 某些工具（如 `systemctl`）的 `auto_installable = false`，无法自动安装

这导致测试结果中包含大量预期的失败，难以区分真正的问题。

## 解决方案

### 1. 添加 `vx check` 命令

新增 `vx check <runtime>` 命令，用于检测 runtime 是否可用（不尝试安装）：

```bash
# 检查 runtime 是否可用
vx check <runtime>

# 静默模式（只返回退出码）
vx check <runtime> --quiet
```

退出码：
- `0`: 可用
- `1`: 不可用（不支持平台、未安装且不能自动安装等）

### 2. 更新测试脚本

测试脚本使用 `vx check` 跳过不可用的工具：

```powershell
# 检查 runtime 是否可用
function Test-RuntimeAvailable {
    param([string]$Runtime)
    
    try {
        $output = & vx check $Runtime --quiet 2>&1
        $exitCode = $LASTEXITCODE
        return $exitCode -eq 0
    } catch {
        return $false
    }
}

# 在测试循环中使用
foreach ($runtime in $runtimes) {
    Write-Info "  Testing: $runtime"
    
    # 检查 runtime 是否可用
    $available = Test-RuntimeAvailable -Runtime $runtime
    if (-not $available) {
        Write-Warning "  ⚠ Runtime '$runtime' is not available on this platform (skipped)"
        $TestResults.Skipped++
        continue
    }
    
    # 继续测试...
}
```

## 测试结果示例

### 改进前

```
=== Testing Provider: systemctl ===
  Runtimes: systemctl, journalctl, systemd-analyze, loginctl
  Testing: systemctl
  ✓ vx list
  ✗ vx systemctl --version (exit: 1)
    Error: Error: Cannot auto-install 'systemctl' (Control systemd services and units). Please install it manually.
  Testing: journalctl
  ✓ vx list
  ✗ vx journalctl --version (exit: 1)
    Error: Error: Cannot auto-install 'systemctl' (Control systemd services and units). Please install it manually.
  ...
```

所有测试都失败，难以区分真正的问题。

### 改进后

```
=== Testing Provider: systemctl ===
  Runtimes: systemctl, journalctl, systemd-analyze, loginctl
  Testing: systemctl
  ⚠ Runtime 'systemctl' is not available on this platform (skipped)
  Testing: journalctl
  ⚠ Runtime 'journalctl' is not available on this platform (skipped)
  Testing: systemd-analyze
  ⚠ Runtime 'systemd-analyze' is not available on this platform (skipped)
  Testing: loginctl
  ⚠ Runtime 'loginctl' is not available on this platform (skipped)

=== Test Summary ===
Total Tests: 120
Passed: 95
Failed: 5
Skipped: 20
Success Rate: 95.00%
```

清晰地区分了：
- **Passed**: 测试通过
- **Failed**: 真正的问题（需要修复）
- **Skipped**: 不支持当前平台（预期行为）

## 实现详情

### vx check 命令实现

文件: `crates/vx-cli/src/commands/check.rs`

```rust
pub async fn handle(
    ctx: &CommandContext,
    runtime_name: &str,
    check_installed: bool,
    check_system: bool,
    detailed: bool,
    quiet: bool,
) -> Result<()> {
    // 1. 检查平台支持
    let current_platform = vx_runtime::Platform::current();
    if !runtime.supports_platform(&current_platform) {
        if !quiet {
            eprintln!("❌ Runtime '{}' does not support the current platform", runtime_name);
        }
        std::process::exit(1);
    }
    
    // 2. 检查安装状态
    let installed = path_manager.list_store_versions(runtime_name)?;
    
    // 3. 检查系统可用性
    let system_available = runtime.detect_system_installation(ctx).await;
    
    // 4. 返回结果
    if installed || system_available {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
```

### 测试脚本更新

文件: `scripts/test-all-providers.ps1`

主要改动：

1. 添加 `Test-RuntimeAvailable` 函数
2. 在测试循环中先检查可用性
3. 跳过不可用的 runtime
4. 统计 Skipped 数量

## 优势

1. **更清晰的测试结果**: 区分真正的失败和预期的跳过
2. **跨平台测试**: 同一个测试脚本可以在 Windows/Linux/macOS 运行
3. **更快的测试速度**: 跳过不可用的工具，避免无用的安装尝试
4. **独立的检测工具**: `vx check` 可用于其他场景（CI/CD、脚本等）

## 使用场景

### CI/CD 流程

```yaml
# GitHub Actions 示例
- name: Check if tools are available
  run: |
    vx check node --quiet && echo "✓ Node.js available" || echo "✗ Node.js not available"
    vx check go --quiet && echo "✓ Go available" || echo "✗ Go not available"

- name: Install missing tools
  run: |
    vx check node --quiet || vx install node
    vx check go --quiet || vx install go
```

### 项目脚本

```bash
#!/bin/bash
# check_tools.sh

REQUIRED_TOOLS=(node go rust)

for tool in "${REQUIRED_TOOLS[@]}"; do
    if vx check "$tool" --quiet; then
        echo "✓ $tool is available"
    else
        echo "✗ $tool is not available"
        exit 1
    fi
done
```

### 交互式安装

```powershell
# Interactive tool checker
$tools = @("node", "go", "rust", "python")

foreach ($tool in $tools) {
    if (vx check $tool --quiet) {
        Write-Host "✓ $tool is available" -ForegroundColor Green
    } else {
        $install = Read-Host "✗ $tool is not available. Install? (y/n)"
        if ($install -eq "y") {
            vx install $tool
        }
    }
}
```
