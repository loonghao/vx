# axoupdater aarch64-pc-windows-msvc 支持问题

## 问题摘要

`axoupdater` 无法在 `aarch64-pc-windows-msvc` 目标上编译，因为其依赖链中的 `ring` 加密库不支持该平台。

## 详细描述

### 问题链
```
axoupdater 0.9.x
  └── axoasset
        └── reqwest 0.12
              └── rustls (默认使用 ring 后端)
                    └── ring (平台特定汇编)
```

### 错误信息
```
error: failed to run custom build command for `ring v0.17.x`

--- stderr
error: couldn't generate assembly for windows-aarch64
```

### 根本原因
`ring` 使用平台特定的汇编代码（如 `sha256-armv8-win64.S`），在交叉编译到 `aarch64-pc-windows-msvc` 时无法编译。

## 建议解决方案

### 方案 1：升级 reqwest 到 0.13 并使用 aws-lc-rs

```toml
# Cargo.toml
[dependencies]
reqwest = { version = "0.13", features = [
    "json",
    "stream",
    "rustls",  # 0.13 默认使用 aws-lc-rs
], default-features = false }
```

`reqwest 0.13` 的 `rustls` feature 默认使用 `aws-lc-rs` 而不是 `ring`，解决了交叉编译问题。

### 方案 2：提供 feature 选择加密后端

```toml
[features]
default = ["rustls-ring"]
rustls-ring = ["reqwest/rustls-tls"]  # 使用 ring
rustls-aws-lc = ["reqwest/rustls"]     # 使用 aws-lc-rs (推荐用于交叉编译)
native-tls = ["reqwest/native-tls"]    # 使用平台原生 TLS
```

### 方案 3：支持更多目标平台预编译

当前 axoupdater 只提供常见平台的预编译二进制。参考 Issue #313，建议扩展支持更多目标。

## 对 vx 项目的影响

vx 使用 `axoupdater` 实现 self-update 功能。当前我们通过以下方式临时解决：

1. 在 `aarch64-pc-windows-msvc` 上禁用 self-update 功能
2. 使用 target-specific 依赖配置

```toml
# vx-cli/Cargo.toml
[target.'cfg(not(all(target_arch = "aarch64", target_os = "windows")))'.dependencies]
axoupdater = { workspace = true, optional = true }
```

## 请求

1. 考虑升级 `axoasset` 或 `axoupdater` 使用 `reqwest 0.13` 以支持 `aws-lc-rs`
2. 或提供 feature flag 让用户选择加密后端
3. 或提供 `aarch64-pc-windows-msvc` 的预编译二进制

## 相关链接

- axoupdater 仓库: https://github.com/axodotdev/axoupdater
- 相关 Issue #313: Support additional prebuilts for uncommon target triples
- vx PR #572: https://github.com/loonghao/vx/pull/572
