# aarch64-pc-windows-msvc 依赖问题分析

## 问题背景

在交叉编译到 `aarch64-pc-windows-msvc` 目标时，vx 项目遇到构建失败，主要原因是多个依赖库使用了 `ring` 加密库，而 `ring` 的平台特定汇编代码无法在该目标上编译。

## 依赖分类

### 自有项目（loonghao 组织）

这些项目我们可以直接修改解决根本问题：

| 项目 | 仓库地址 | 问题 | 状态 |
|------|----------|------|------|
| msvc-kit | https://github.com/loonghao/msvc-kit | reqwest 0.13 默认使用 aws-lc-rs 需要 cmake/NASM | Issue #44 已创建 |
| turbo-cdn | https://github.com/loonghao/turbo-cdn | aarch64-windows 交叉编译支持 | Issue #126 已创建 (新) |
| turbo-cdn | https://github.com/loonghao/turbo-cdn | rustls 默认使用 ring/aws-lc-rs 需要额外构建工具 | Issue #102 已创建 |

### 第三方项目

这些项目需要 fork 并提交 PR：

| 项目 | 仓库地址 | 问题 | 建议 |
|------|----------|------|------|
| axoupdater | https://github.com/axodotdev/axoupdater | 依赖 axoasset → reqwest 0.12 → ring | Fork 并修改为支持 aws-lc-rs |

## 详细分析

### 1. turbo-cdn (自有)

**问题链：**
```
turbo-cdn
  └── reqwest 0.12/0.13
        └── rustls
              └── ring (平台特定汇编，aarch64-windows 不支持)
```

**已有 Issues：**
- #102: rustls feature requires cmake and NASM on Windows due to aws-lc-sys dependency
- #89: Make self_update dependency optional to avoid lzma-sys conflict

**解决方案：**
1. 升级到 reqwest 0.13 并配置使用 `aws-lc-rs`
2. 或提供 `ring` 和 `aws-lc-rs` 两种后端选项

### 2. axoupdater (第三方)

**问题链：**
```
axoupdater
  └── axoasset
        └── reqwest 0.12
              └── rustls (默认 ring)
```

**需要 Fork 并解决：**
1. Fork axoupdater 到 loonghao 组织
2. 修改 axoasset 依赖或直接修改 axoupdater 的 reqwest 配置
3. 提交 PR 到上游

## 当前临时解决方案

在 vx 项目中，我们通过以下方式临时解决：

1. **target-specific 依赖**：在 `vx-cli` 和 `vx-runtime` 中，将 `axoupdater` 和 `turbo-cdn` 设为 `aarch64-pc-windows-msvc` 排除
2. **aws-lc-rs 强制使用**：在根 `Cargo.toml` 中显式配置 rustls 使用 `aws-lc-rs`

```toml
# Cargo.toml
rustls = { version = "0.23", default-features = false, features = [
  "aws_lc_rs",
  "logging",
  "std",
  "tls12",
] }
```

## 行动计划

### 短期（已完成）
- [x] 在 vx 中禁用 aarch64-windows 的 axoupdater 和 turbo-cdn
- [x] 创建 PR #572 修复交叉编译

### 中期
- [ ] 为 turbo-cdn 添加 aws-lc-rs 支持（自有项目）
- [ ] Fork axoupdater 并添加 aws-lc-rs 支持
- [ ] 向 axoupdater 上游提交 PR

### 长期
- [ ] 推动 ring 项目支持 aarch64-windows
- [ ] 统一所有依赖使用 aws-lc-rs

## 相关链接

- vx PR #572: https://github.com/loonghao/vx/pull/572
- turbo-cdn Issue #102: https://github.com/loonghao/turbo-cdn/issues/102
- msvc-kit Issue #44: https://github.com/loonghao/msvc-kit/issues/44
