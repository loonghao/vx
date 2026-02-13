# Fork axoupdater 行动计划

## 背景

`axoupdater` 是 vx 用于实现 self-update 功能的库，由 axodotdev 组织维护。由于该库依赖 `ring` 加密库，无法在 `aarch64-pc-windows-msvc` 目标上编译，我们需要 fork 并解决此问题。

## 上游仓库信息

- **原始仓库**: https://github.com/axodotdev/axoupdater
- **Stars**: 26
- **Open Issues**: 11
- **主要维护者**: axodotdev

## 问题分析

### 依赖链
```
axoupdater
  └── axoasset (axodotdev 内部库)
        └── reqwest 0.12
              └── rustls (默认 ring)
```

### 已有相关 Issues
- #313: Support additional prebuilts for "uncommon target triples"
- #85: pure rust self-install logic

## Fork 计划

### Step 1: Fork 仓库
```bash
# Fork 到 loonghao 组织
gh repo fork axodotdev/axoupdater --org loonghao
```

### Step 2: 克隆并创建分支
```bash
git clone https://github.com/loonghao/axoupdater.git
cd axoupdater
git checkout -b fix/aws-lc-rs-support
```

### Step 3: 修改依赖

#### 方案 A: 修改 axoasset 依赖
axoasset 是 axodotdev 的内部库，需要单独 fork 或修改：

```toml
# Cargo.toml
[dependencies]
axoasset = { git = "https://github.com/loonghao/axoasset", branch = "fix/aws-lc-rs" }
```

#### 方案 B: 替换 HTTP 客户端
直接修改 axoupdater 使用不同的 HTTP 客户端配置：

```toml
# Cargo.toml
[dependencies]
# 移除 axoasset 对 reqwest 的依赖传递
reqwest = { version = "0.13", features = [
    "json",
    "stream",
    "rustls",  # 0.13 默认 aws-lc-rs
], default-features = false }
```

### Step 4: 添加 Feature Flag

```toml
[features]
default = ["rustls"]
rustls = ["reqwest/rustls"]           # aws-lc-rs (支持交叉编译)
rustls-ring = ["reqwest/rustls-tls"]  # ring (原有行为)
native-tls = ["reqwest/native-tls"]   # 平台原生 TLS
```

### Step 5: 测试

```bash
# 测试常规构建
cargo build

# 测试交叉编译
cargo build --target aarch64-pc-windows-msvc --features native-tls
```

### Step 6: 提交 PR 到上游

1. 在 axodotdev/axoupdater 创建 Issue 描述问题
2. 提交 PR 链接到该 Issue
3. 等待上游反馈

## vx 项目集成

修改 vx-cli 的 Cargo.toml：

```toml
[patch.crates-io]
axoupdater = { git = "https://github.com/loonghao/axoupdater", branch = "fix/aws-lc-rs-support" }

# 或使用 feature
[target.'cfg(all(target_arch = "aarch64", target_os = "windows"))'.dependencies]
axoupdater = { git = "https://github.com/loonghao/axoupdater", branch = "fix/aws-lc-rs-support", features = ["native-tls"] }
```

## 时间线

| 阶段 | 任务 | 预计时间 |
|------|------|----------|
| 1 | Fork 并创建分支 | 1 小时 |
| 2 | 分析并修改依赖 | 2-4 小时 |
| 3 | 本地测试 | 2 小时 |
| 4 | CI 配置和测试 | 4 小时 |
| 5 | 提交上游 PR | 1 小时 |
| 6 | vx 集成测试 | 2 小时 |

## 风险和缓解

| 风险 | 缓解措施 |
|------|----------|
| 上游不接受 PR | 使用 fork 版本作为长期方案 |
| axoasset 也需要修改 | 同时 fork axoasset |
| API 兼容性问题 | 保持接口不变，只修改内部实现 |

## 相关链接

- axoupdater 仓库: https://github.com/axodotdev/axoupdater
- Issue #313: https://github.com/axodotdev/axoupdater/issues/313
- vx PR #572: https://github.com/loonghao/vx/pull/572
- turbo-cdn Issue #126: https://github.com/loonghao/turbo-cdn/issues/126
