# RFC 0033: Package Alias Provider

> **状态**: Draft
> **作者**: VX Team
> **创建日期**: 2026-02-15
> **目标版本**: v0.14.0
> **关联**: RFC-0027 (Implicit Package Execution), RFC-0032 (Build Time Optimization)

## 摘要

引入 **Package Alias** 机制，允许 provider 在 `provider.toml` 中声明自己是某个生态系统包的别名。当用户执行 `vx vite` 时，系统自动将其路由到 `npm:vite` 的包执行路径，统一安装和运行行为，同时保留第一层命名空间的用户体验。

## 动机

### 当前问题

1. **两条执行路径不一致**：`vx vite` 走 Provider 安装路径（需要在 PATH/store 中找 npm），`vx npm:vite` 走包执行路径（自动安装 node 再安装包）。前者失败，后者成功。

2. **维护成本高**：vite、release-please、rez、pre-commit、meson 这 5 个工具各有独立的 Rust crate（runtime.rs、provider.rs、config.rs、Cargo.toml），但它们的逻辑几乎完全相同——都是 `PackageRuntime` 的薄包装。

3. **Node/Python 版本绑定缺失**：`vx vite` 走 Provider 路径时，不感知项目 `vx.toml` 中的 node 版本配置，也不会自动安装依赖运行时。

4. **编译时间浪费**：5 个同构 crate 增加了 ~5 个编译单元，与 RFC 0032 的优化目标相悖。

#### 设计目标

- `vx vite@5.0` 完全等价于 `vx npm:vite@5.0`
- `vx rez@3.0` 完全等价于 `vx uv:rez@3.0`（Python 生态统一使用 uv，更健壮）
- `vx uninstall rez@3.0` 完全等价于 `vx uninstall uv:rez@3.0`
- 支持 `choco:xxx::cmd` 格式，统一安装到 vx 管控的 `choco_tools` 目录
- 保留第一层命名空间（用户可以直接 `vx vite` 而不用记 `npm:` 前缀）
- 零 Rust 代码：纯 `provider.toml` 声明式配置
- 自动继承项目的运行时版本（node/python）

## 设计方案

### 1. provider.toml 新增 `[provider.package_alias]` 字段

```toml
[provider]
name = "vite"
description = "Next generation frontend build tool"
homepage = "https://vitejs.dev"
ecosystem = "nodejs"

# 新增：声明此 provider 是一个包别名
[provider.package_alias]
ecosystem = "npm"        # 目标生态系统（对应 npm:/pip:/cargo:/uv:/choco: 前缀）
package = "vite"         # 包名
# executable = "vite"    # 可选，默认同 runtimes[0].executable
```

#### Python 生态：统一使用 `uv` 替代 `pip`

Python 包的 `package_alias.ecosystem` 应使用 `"uv"` 而非 `"pip"`：

- `uv` 安装更快、更健壮（隔离的 tool install）
- `uv` 自动管理 Python 虚拟环境
- `pip` 安装器需要手动创建 venv，且缺乏 tool isolation
- 对于独立工具的全局安装场景，`uvx` 是最佳选择

```toml
# ✅ 推荐：使用 uv
[provider.package_alias]
ecosystem = "uv"
package = "rez"

# ❌ 不推荐：pip 缺乏隔离
[provider.package_alias]
ecosystem = "pip"
package = "rez"
```

### 2. Manifest Schema 变更

`ProviderMeta` 结构体新增 `package_alias` 字段：

```rust
/// Package alias configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageAlias {
    /// Target ecosystem (e.g., "npm", "pip", "cargo")
    pub ecosystem: String,
    /// Package name in that ecosystem
    pub package: String,
    /// Executable name override (defaults to package name)
    pub executable: Option<String>,
}

pub struct ProviderMeta {
    pub name: String,
    pub description: Option<String>,
    // ... existing fields ...
    /// Package alias: if set, this provider routes to a package execution
    #[serde(default)]
    pub package_alias: Option<PackageAlias>,
}
```

### 3. CLI 路由变更

在 `execute_tool()` 中，当检测到已知 runtime 时，增加 package_alias 检查：

```
用户输入: vx vite@5.0 --port 3000
         │
         ▼
Step 1: is_package_request("vite@5.0")? → false (无 : 前缀)
         │
         ▼
Step 2: registry.get_runtime("vite") → 找到
         │
         ▼
Step 3: 检查 runtime 对应的 manifest 是否有 package_alias?
        │
        ├── 有 package_alias { ecosystem: "npm", package: "vite" }
        │   → 合成 PackageRequest { ecosystem: "npm", package: "vite", version: "5.0" }
        │   → 走 execute_package_request() 路径
        │   → 自动安装 node（尊重 vx.toml）→ npm install vite → 执行
        │
        └── 无 → 正常 provider 路由
```

### 4. 版本映射规则

#### 执行 `(vx <tool>)`

| 用户输入 | 等价转换 | 说明 |
|----------|----------|------|
| `vx vite` | `vx npm:vite` | 使用 latest |
| `vx vite@5.0` | `vx npm:vite@5.0` | 指定包版本 |
| `vx rez@3.0` | `vx uv:rez@3.0` | uv 生态（Python 工具推荐使用 uv） |
| `vx pre-commit` | `vx uv:pre-commit` | uv 生态 |
| `vx meson` | `vx uv:meson` | uv 生态 |
| `vx release-please` | `vx npm:release-please` | npm 生态 |

#### 卸载 `(vx uninstall <tool>)`

| 用户输入 | 等价转换 | 说明 |
|----------|----------|------|
| `vx uninstall vite` | `vx global uninstall npm:vite` | 卸载 npm 全局包 |
| `vx uninstall rez@3.0` | `vx global uninstall uv:rez` | 卸载 uv 安装的包 |
| `vx uninstall pre-commit` | `vx global uninstall uv:pre-commit` | 卸载 uv 安装的包 |

### 5. Uninstall 路由

当前 `vx uninstall` 只处理运行时版本的卸载（通过 `runtime.uninstall()`）。对于 package_alias provider，需要路由到 `vx global uninstall` 的包卸载路径：

```
用户输入: vx uninstall rez@3.0
         │
         ▼
Step 1: parse "rez@3.0" → tool_name="rez", version="3.0"
         │
         ▼
Step 2: registry.get_runtime("rez") → 找到
         │
         ▼
Step 3: 检查 manifest 是否有 package_alias?
        │
        ├── 有 package_alias { ecosystem: "uv", package: "rez" }
        │   → 合成 spec "uv:rez"
        │   → 调用 global uninstall handler
        │   → 删除隔离目录 + shims + 注册表项
        │
        └── 无 → 走原有 runtime.uninstall() 路径
```

### 6. Choco 生态支持

Chocolatey 包也可以通过 package_alias 路由到 `choco:package::executable` 格式：

```toml
# 示例：通过 choco 安装 7zip
[provider]
name = "7zip"
description = "7-Zip file archiver"
ecosystem = "system"

[provider.package_alias]
ecosystem = "choco"
package = "7zip"
executable = "7z"       # choco:7zip::7z
```

Choco 安装的包统一放在 vx 管控的 `choco_tools` 目录（`~/.vx/choco_tools`），方便统一管理和卸载：

| 用户输入 | 等价转换 | 说明 |
|----------|----------|------|
| `vx 7z` | `vx choco:7zip::7z` | choco 安装 + 指定 executable |
| `vx uninstall 7z` | `vx uninstall choco:7zip` | choco 卸载 |

> **注意**：choco 生态的 installer 需要后续实现（Phase 4），本 RFC 先定义 schema 和路由机制。

### 7. 受影响的 Provider 列表

| Provider | 当前状态 | 目标状态 | 映射 |
|----------|---------|---------|------|
| vite | Rust crate + provider.toml | **provider.toml only** | `npm:vite` |
| release-please | Rust crate + provider.toml | **provider.toml only** | `npm:release-please` |
| rez | Rust crate + provider.toml | **provider.toml only** | `uv:rez` |
| pre-commit | Rust crate + provider.toml | **provider.toml only** | `uv:pre-commit` |
| meson | Rust crate + provider.toml | **provider.toml only** | `uv:meson` |

### 6. provider.toml 完整示例

#### npm 包别名（vite）

```toml
[provider]
name = "vite"
description = "Next generation frontend build tool"
homepage = "https://vitejs.dev"
repository = "https://github.com/vitejs/vite"
ecosystem = "nodejs"

[provider.package_alias]
ecosystem = "npm"
package = "vite"

# runtimes 仍需保留，用于 vx list/info 展示和约束检查
[[runtimes]]
name = "vite"
description = "Vite frontend build tool"
executable = "vite"

[runtimes.versions]
source = "npm"
package = "vite"

[runtimes.platforms.windows]
executable_extensions = [".cmd", ".exe"]

[runtimes.platforms.unix]
executable_extensions = []

# 约束保留：用于 vx info 展示、兼容性检查
[[runtimes.constraints]]
when = ">=5"
requires = [
    { runtime = "node", version = ">=18", recommended = "20", reason = "Vite 5.x requires Node.js 18+" }
]
```

#### uv 包别名（rez）

```toml
[provider]
name = "rez"
description = "Cross-platform package manager for deterministic environments"
homepage = "https://rez.readthedocs.io"
ecosystem = "python"

[provider.package_alias]
ecosystem = "uv"
package = "rez"

[[runtimes]]
name = "rez"
description = "Rez package manager"
executable = "rez"

[runtimes.versions]
source = "github-releases"
owner = "AcademySoftwareFoundation"
repo = "rez"

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "python", version = ">=3.8", recommended = "3.11", reason = "Rez requires Python 3.8+" }
]

# bundled 子命令仍然保留
[[runtimes]]
name = "rez-env"
description = "Rez environment resolver"
executable = "rez-env"
bundled_with = "rez"
auto_installable = false
```

## 实施计划

### Phase 1: Schema & 执行路由 ✅

- [x] `vx-manifest`: `ProviderMeta` 新增 `package_alias: Option<PackageAlias>` 字段
- [x] `vx-runtime`: `ManifestRegistry` 传递 `package_alias` 信息到 `ProviderRegistry`
- [x] `vx-cli`: `execute_tool()` 增加 package_alias 检查，自动路由到 `execute_package_request()`
- [x] 更新 5 个 provider.toml 添加 `[provider.package_alias]` 配置

### Phase 2: 移除 Rust crate ✅

- [x] 删除 `vx-providers/vite/src/`（保留 provider.toml）
- [x] 删除 `vx-providers/release-please/src/`
- [x] 删除 `vx-providers/rez/src/`（保留 bundled runtimes 的 manifest 声明）
- [x] 删除 `vx-providers/pre-commit/src/`
- [x] 删除 `vx-providers/meson/src/`
- [x] 从 `vx-cli/Cargo.toml` 移除 5 个 crate 依赖
- [x] 从 `registry.rs` 的宏调用中移除 5 个 provider
- [x] 从 workspace `Cargo.toml` 移除 5 个 member

### Phase 3: Python 生态 pip→uv 迁移

- [x] 更新 rez/provider.toml: `ecosystem = "uv"` 替代 `"pip"`
- [x] 更新 pre-commit/provider.toml: `ecosystem = "uv"` 替代 `"pip"`
- [x] 更新 meson/provider.toml: `ecosystem = "uv"` 替代 `"pip"`

### Phase 4: Uninstall 路由

- [x] `vx uninstall <tool>` 增加 package_alias 检查，路由到 `global uninstall` 路径
- [ ] 添加 E2E 测试验证 uninstall 路由

### Phase 5: Choco 生态支持（待实现）

- [ ] `vx-ecosystem-pm` 添加 `ChocoInstaller` 实现
- [ ] 定义 `~/.vx/choco_tools` 作为 choco 工具的统一安装目录
- [ ] `choco install <pkg> --install-directory=<choco_tools>` 集成
- [ ] `choco uninstall <pkg>` 卸载支持
- [ ] choco 包的 provider.toml 示例

### Phase 6: 测试

- [ ] E2E 测试：`vx vite --help` 等价于 `vx npm:vite --help`
- [ ] E2E 测试：`vx rez --version` 等价于 `vx uv:rez --version`
- [ ] E2E 测试：`vx uninstall vite` 正确路由到 `global uninstall npm:vite`
- [ ] 回归测试：`vx npm:vite` 仍然正常工作

## 向后兼容性

- **完全兼容**：`vx npm:vite` 语法不受影响
- **行为变更**：`vx vite` 从 Provider 安装路径变为包执行路径（修复了当前的 bug）
- **vx.toml**：`[tools] vite = "5.0"` 继续工作（版本传递给包请求）

## 替代方案

### 方案 A：修复 PackageRuntime 的依赖安装

在 `PackageRuntime::install_package()` 中自动安装 `required_runtime()`。

**否决原因**：治标不治本，仍需维护 5 个 Rust crate，且两条路径的语义持续分裂。

### 方案 B：移除这些 Provider，只用 `npm:` 前缀

强制用户使用 `vx npm:vite`。

**否决原因**：损失第一层命名空间的用户体验，增加记忆负担。

## 参考资料

- RFC 0027: Implicit Package Execution
- RFC 0032: Build Time Optimization (Phase 3 - merge thin providers)
