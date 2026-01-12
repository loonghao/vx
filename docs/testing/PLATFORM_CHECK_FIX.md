# Platform Check 修复

## 问题

原始的 `vx check` 命令在检查平台支持时逻辑过于严格，导致大量应该支持 Windows 的工具被错误地标记为"不支持当前平台"。

### 根本原因

在 `crates/vx-cli/src/commands/check.rs` 中，平台检查使用了简单的相等比较：

```rust
// ❌ 错误的实现
let platform_supported = supported_platforms.iter().any(|p| {
    p.os == current_platform.os && p.arch == current_platform.arch
});
```

这个检查**忽略了 `Platform::matches()` 方法的智能匹配逻辑**。

## 解决方案

### 1. 使用 Runtime trait 的内置方法

Runtime trait 已经提供了 `is_platform_supported()` 方法，它使用 `Platform::matches()` 做更智能的匹配：

```rust
// ✓ 正确的实现
if !runtime.is_platform_supported(&current_platform) {
    // 平台不支持
}
```

### 2. 更新测试脚本逻辑

测试脚本不应该把"不可用"等同于"不支持平台"。

**改进前**：
```powershell
function Test-RuntimeAvailable {
    $exitCode = & vx check $Runtime --quiet
    return $exitCode -eq 0  # 退出码 0 = 可用
}
```

问题：
- 退出码 0 = runtime 已安装或在 PATH（available）
- 退出码 1 = runtime 不可用（**但可能支持平台，只是未安装**）

**改进后**：
```powershell
function Test-RuntimePlatformSupported {
    $output = & vx check $Runtime 2>&1 | Out-String
    
    # 检查错误消息中是否包含平台不支持的提示
    if ($output -match "does not support the current platform") {
        return $false  # 不支持平台
    }
    
    return $true  # 支持平台（即使未安装）
}
```

## 测试结果

### 改进前

```
=== Testing Provider: deno ===
  ⚠ Runtime 'deno' is not available on this platform (skipped)

=== Testing Provider: docker ===
  ⚠ Runtime 'docker' is not available on this platform (skipped)

=== Testing Provider: ffmpeg ===
  ⚠ Runtime 'ffmpeg' is not available on this platform (skipped)
```

**问题**：这些工具都支持 Windows，但被错误地跳过了。

### 改进后

```
=== Testing Provider: deno ===
  Testing: deno
  ✓ vx list
  ✗ vx deno --version (exit: 1)
    Error: deno not installed, installing...

=== Testing Provider: spack ===
  Testing: spack
  ⚠ Runtime 'spack' does not support the current platform (skipped)
```

**结果**：
- ✅ `deno` 被正确测试（即使未安装）
- ✅ `spack` 被正确跳过（真的不支持 Windows）

## 验证

运行以下命令验证修复：

```bash
# 测试支持 Windows 的工具
vx check deno   # 应该显示"可以安装"
vx check go     # 应该显示"可以安装"

# 测试不支持 Windows 的工具
vx check spack  # 应该显示"does not support the current platform"
```

运行测试脚本：

```bash
pwsh scripts/test-all-providers.ps1
```

## 相关文件

- `crates/vx-cli/src/commands/check.rs` - 修复平台检查逻辑
- `scripts/test-all-providers.ps1` - 更新测试脚本
- `crates/vx-runtime/src/runtime.rs` - Runtime trait 定义
- `crates/vx-runtime/src/platform.rs` - Platform 实现

## 总结

核心改进：

1. **使用正确的 API**：使用 `runtime.is_platform_supported()` 而不是手动比较
2. **区分概念**：
   - "平台支持" ≠ "已安装"
   - "可以安装" ≠ "已经可用"
3. **测试策略**：
   - 只跳过真正不支持平台的工具
   - 未安装但支持平台的工具应该正常测试（会触发自动安装）
