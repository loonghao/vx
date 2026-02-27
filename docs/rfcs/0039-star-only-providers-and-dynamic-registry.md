# RFC 0039: Star-Only Providers & Dynamic Provider Registry

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2026-02-27
> **目标版本**: v0.17.0
> **依赖 RFC**: [RFC 0036](./0036-starlark-provider-support.md), [RFC 0038](./0038-provider-star-replaces-toml.md)

## 摘要

本 RFC 提出两个相互关联的改进：

1. **Star-Only Provider**：将所有 provider 的 Rust crate 简化为"空壳"（仅包含 `lib.rs` + `Cargo.toml`），消除冗余的 Rust 代码，让 `provider.star` 成为唯一的实现载体，并为每个 provider 补充完整的 `starlark_logic_tests.rs` 单元测试。

2. **动态 Provider 注册表**：实现类似 `go mod` 的动态 provider 加载机制，支持从独立 Git 仓库或 URL 拉取 `provider.star`，彻底分离核心与非核心 provider，大幅减少编译时间。

---

## 动机

### 当前问题

#### 问题 1：Rust crate 是冗余的薄包装

当前每个 provider 都有一个 Rust crate，但其内容几乎完全相同：

```rust
// 所有 provider 的 src/lib.rs 都是这个模式
pub const PROVIDER_STAR: &str = include_str!("../provider.star");
pub fn star_metadata() -> &'static vx_starlark::StarMetadata { ... }
```

这 70+ 个 crate 的 Rust 代码**完全可以由 `vx-starlark` 自动生成**，不需要手写。

#### 问题 2：测试覆盖率不均匀

当前只有 `go`、`node`、`python` 三个 provider 有 `starlark_logic_tests.rs`，其余 66 个 provider 缺少对 `provider.star` 逻辑的直接测试。这意味着：
- URL 构建逻辑的 bug 无法被单元测试捕获
- 平台检测逻辑无法被验证
- install_layout 的正确性无法被保证

#### 问题 3：编译时间随 provider 数量线性增长

`vx-cli` 依赖 70+ provider crate，每次修改任何 provider 都会触发重新编译。

#### 问题 4：社区贡献门槛高

贡献一个新 provider 需要了解 Rust，在 monorepo 中创建 crate，等待 PR 合并。
理想情况下，贡献者只需要写一个 `provider.star` 文件。

---

## 设计方案

### Phase 1：为所有 Provider 补充 starlark_logic_tests.rs（立即实施）

**目标**：为所有 66 个缺少 `starlark_logic_tests.rs` 的 provider 补充完整的 Starlark 逻辑测试。

**标准测试套件**（每个 provider 必须覆盖）：

```rust
// 1. 元数据测试
test_provider_name()
test_provider_ecosystem()
test_provider_has_homepage()
test_provider_has_repository()

// 2. Runtime 定义测试
test_runtimes_list_not_empty()
test_runtime_names()
test_runtime_aliases()          // 如有别名
test_runtime_bundled_with()     // 如有 bundled runtime

// 3. 下载 URL 测试（每个支持的平台）
test_download_url_linux_x64()
test_download_url_linux_arm64()
test_download_url_macos_x64()
test_download_url_macos_arm64()
test_download_url_windows_x64()
test_download_url_unknown_platform_returns_none()

// 4. 安装布局测试
test_install_layout_type()
test_install_layout_strip_prefix()
test_install_layout_executable_paths()

// 5. Lint 检查
test_provider_star_lint_clean()
```

**测试文件结构**：

```rust
//! Pure Starlark logic tests for {name} provider.star
//!
//! These tests use `starlark::assert::Assert` to test the Starlark logic
//! directly — no network calls, no Rust runtime, just the script itself.

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_{name}::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_{name}::PROVIDER_STAR)
}
```

### Phase 2：Star-Only Provider 迁移（中期目标）

**目标**：将所有 provider 的 Rust crate 简化为"空壳"，消除冗余代码。

**空壳 `lib.rs` 模板**：

```rust
//! {name} provider for vx
//!
//! This is a star-only provider. All logic is in `provider.star`.
//! The Rust crate exists only for workspace membership and test hosting.

pub const PROVIDER_STAR: &str = include_str!("../provider.star");

pub fn star_metadata() -> &'static vx_starlark::StarMetadata {
    use std::sync::OnceLock;
    static META: OnceLock<vx_starlark::StarMetadata> = OnceLock::new();
    META.get_or_init(|| vx_starlark::StarMetadata::parse(PROVIDER_STAR))
}

pub fn create_provider() -> std::sync::Arc<dyn vx_runtime::Provider> {
    vx_runtime::StarProvider::new(PROVIDER_STAR)
}
```

**注册机制变更**：

```rust
// vx-cli/src/registry.rs（新）
// 不再需要 70+ 个 use 语句和 create_provider() 调用
// 改为通过 ALL_PROVIDER_STARS 自动注册

pub fn register_all_providers(registry: &mut ProviderRegistry) {
    for (name, content) in crate::ALL_PROVIDER_STARS {
        registry.register_star_provider(name, content);
    }
    registry.load_user_providers();
}
```

### Phase 3：动态 Provider 注册表（长期目标）

**架构概览**：

```
vx 二进制（核心 providers 内置）
    │
    ├── ~/.vx/providers/          ← 用户本地 providers（已支持）
    │   └── my-tool/provider.star
    │
    ├── ~/.vx/registry/           ← 动态注册表缓存（新增）
    │   ├── registry.lock         ← 锁文件（类似 go.sum）
    │   └── providers/
    │       ├── awscli@1.0.0/provider.star
    │       └── terraform@1.2.0/provider.star
    │
    └── vx-providers.toml         ← 项目级 provider 配置（新增）
```

**`vx provider` 子命令**：

```
vx provider add awscli              # 从注册表安装最新版
vx provider add awscli@1.0.0        # 指定版本
vx provider add github:vx-dev/vx-providers-cloud/terraform
vx provider remove awscli
vx provider list                    # 列出已安装的 provider
vx provider update                  # 更新所有 provider
vx provider search kubectl          # 搜索注册表
vx provider info awscli             # 查看 provider 详情
```

---

## 实施计划

### Phase 1：补充 starlark_logic_tests.rs（本 RFC 主要内容）

**分类**（按测试复杂度）：

| 类别 | Provider | 测试重点 |
|------|----------|----------|
| 核心运行时 | node, go, python, rust, uv, bun, deno | URL 构建、平台检测、环境变量 |
| 核心工具 | git, cmake, ninja, protoc | URL 构建、平台检测 |
| 云工具 | awscli, azcli, gcloud | Linux 直接下载、其他平台 system_install |
| 容器工具 | docker, kubectl, helm | URL 构建、平台检测 |
| CLI 工具 | bat, fd, fzf, jq, yq, ripgrep | github_rust_provider 模板 |
| 系统工具 | bash, make, curl | system_install 逻辑 |

**每个 provider 的 Cargo.toml 需要添加 dev-dependencies**：

```toml
[dev-dependencies]
rstest = { workspace = true }
starlark = { version = "0.13" }
vx-starlark = { workspace = true, features = ["test-mocks"] }
```

### Phase 2：Star-Only Provider 迁移

- 移除所有 provider 的 `src/provider.rs`、`src/runtime.rs`、`src/config.rs`
- 简化 `src/lib.rs` 为标准空壳
- 更新 `vx-cli/src/registry.rs` 使用自动注册

### Phase 3：动态 Provider 注册表

- 实现 `vx provider add/remove/list/update` 命令
- 建立 `providers.vx.dev` 注册表服务
- 实现版本锁定（`~/.vx/registry/registry.lock`）
- 支持从 Git 仓库直接加载 `provider.star`

---

## 关于是否发布 Rust Crate

**结论：不需要发布核心 crate 到 crates.io**

理由：
1. `provider.star` 已经是完整的扩展接口，开发者不需要了解 Rust 就能创建 provider
2. 社区贡献者只需要写 `provider.star` 文件，放到独立仓库即可
3. 如果未来确实需要，可以发布 `vx-runtime` 和 `vx-starlark`

**开发者调试 provider 的方式**：

```bash
# 方式 1：直接放到 ~/.vx/providers/（已支持）
mkdir -p ~/.vx/providers/my-tool
cp my-provider.star ~/.vx/providers/my-tool/provider.star
vx my-tool --version

# 方式 2（Phase 3 后）：从本地路径加载
# vx-providers.toml
[[providers]]
name = "my-tool"
source = "local"
path = "./my-provider.star"
```

---

## 影响分析

### 编译时间优化

| 阶段 | 编译时间（估算） | 说明 |
|------|-----------------|------|
| 当前 | ~4 分钟 | 70+ provider crate 全量编译 |
| Phase 2 后 | ~2.5 分钟 | 空壳 crate 编译极快 |
| Phase 3 后 | ~1.5 分钟 | 非核心 provider 不再编译进二进制 |

### 向后兼容性

- Phase 1 完全向后兼容：用户无感知
- Phase 2 完全向后兼容：API 不变
- Phase 3 需要用户显式安装非核心 provider（可提供迁移工具）

---

## 参考

- [RFC 0036: Starlark Provider Support](./0036-starlark-provider-support.md)
- [RFC 0038: Provider Star Replaces TOML](./0038-provider-star-replaces-toml.md)
- [go mod documentation](https://go.dev/ref/mod)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-02-27 | Draft | 初始草案 |
