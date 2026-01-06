# RFC 0012: Provider Manifest (provider.toml)

> **状态**: Implemented（Phase 1/2 完成）
> **作者**: vx team
> **创建日期**: 2026-01-06
> **目标版本**: v0.8.0

## 摘要

引入 `provider.toml` 配置文件作为每个 Provider 的声明式清单，将依赖约束、别名、hooks 等元数据从 Rust 代码中抽离到配置文件中。这种设计参考了 Spack 和 Rez 等成熟包管理器的方案，使 Provider 的定义更加灵活、可维护，并为未来复杂的依赖需求奠定基础。

## 主流方案调研

在设计本方案之前，我们调研了以下主流实现：

### 1. Spack (spack/spack)

**架构**: 基于 Python 的包定义文件 (`package.py`)

**核心设计**:
```python
class Yarn(Package):
    version("1.22.22", sha256="...")
    version("4.0.0", sha256="...")
    
    # 依赖定义 - 支持条件和版本范围
    depends_on("node@12:22", when="@1:")      # Yarn 1.x 需要 Node 12-22
    depends_on("node@16:", when="@2:3")       # Yarn 2-3.x 需要 Node 16+
    depends_on("node@18:", when="@4:")        # Yarn 4.x 需要 Node 18+
    
    # 变体定义
    variant("pnp", default=False, description="Enable Plug'n'Play")
    
    # 条件依赖
    depends_on("python@3.8:", when="+pnp", type="build")
```

**关键特性**:
- `depends_on(spec, when=condition, type=...)` - 条件依赖声明
- `variant(name, values, default)` - 可配置变体
- 版本范围语法: `@1.2:3.4` 表示 `>=1.2, <3.5`
- 依赖类型: `build`, `link`, `run`, `test`

**优点**:
- 表达力强，支持复杂的条件依赖
- 版本约束与包定义紧密结合
- 社区验证，支持数千个包

**缺点**:
- Python 代码，不适合静态分析
- 运行时解析开销

### 2. Rez (AcademySoftwareFoundation/rez)

**架构**: Python 变量定义文件 (`package.py`)

**核心设计**:
```python
name = "yarn"
version = "1.22.22"
description = "Fast, reliable, and secure dependency management"
authors = ["Yarn Contributors"]

# 运行时依赖 - 版本范围语法
requires = [
    "node-12+<23",        # Node.js 12-22
]

# 构建时依赖
build_requires = [
    "cmake-3.10+",
]

# 暴露的工具
tools = ["yarn", "yarnpkg"]

# 别名
aliases = {
    "yarnpkg": "yarn"
}

# 环境设置
def commands():
    env.PATH.append("{root}/bin")
```

**关键特性**:
- `requires` - 运行时依赖列表
- `build_requires` - 构建时依赖
- `tools` - 暴露的可执行文件
- `commands()` - 环境变量设置函数
- 版本语法: `package-1.2+<3` 表示 `>=1.2, <3`

**优点**:
- 简洁的声明式语法
- 清晰的依赖分类 (requires vs build_requires)
- 工具和别名显式声明
- **依赖解析器**: 会检查本地已安装的版本是否满足约束

**缺点**:
- 条件依赖需要用函数实现
- 版本约束不如 Spack 灵活

### 3. Cargo/npm (标准 semver)

**架构**: TOML/JSON 配置文件

**核心设计** (npm semver 语法):
```
>=1.2.3          # 大于等于
<2.0.0           # 小于
>=1.2.3, <2.0.0  # 范围 (AND)
^1.2.3           # 兼容版本 (>=1.2.3, <2.0.0)
~1.2.3           # 补丁版本 (>=1.2.3, <1.3.0)
1.2.*            # 通配符
*                # 任意版本
```

**优点**:
- 业界标准，广泛使用
- 工具生态成熟
- 易于理解

### 方案对比

| 特性 | Spack | Rez | npm/Cargo |
|------|-------|-----|-----------|
| 配置格式 | Python | Python | TOML/JSON |
| 条件依赖 | ✓ `when=` | ✓ 函数 | △ features |
| 版本范围 | `@1:2` | `1+<2` | `>=1, <2` |
| 静态分析 | ✗ | ✗ | ✓ |
| 依赖解析 | ✓ | ✓ | ✓ |
| 本地版本检测 | ✓ | ✓ | ✓ |

### 设计启示

基于以上调研，本 RFC 应采用：

1. **TOML 格式** - 静态配置易于解析和验证
2. **npm/Cargo semver 语法** - 业界标准，用户熟悉
3. **Spack 的条件语法** - `when` 条件实现版本相关的依赖约束
4. **Rez 的依赖解析模型** - 检查本地已安装版本，满足则复用
5. **复用 vx-resolver 的 VersionRequest** - 与现有系统统一

## 动机

### 当前状态分析

目前，Runtime 的依赖约束定义在两个地方：

1. **集中式 `ConstraintsRegistry`** (`vx-runtime/src/constraints.rs`)
   - 所有约束硬编码在一个文件中
   - 难以维护，随着 Provider 增多会变得臃肿
   - 与 Provider 代码分离，不直观

2. **Provider 代码中** (如 `YarnRuntime::dependencies()`)
   - 已废弃，现在只是注释
   - 无法表达版本相关的约束

```rust
// 当前: 集中式定义 (constraints.rs)
self.register("yarn", vec![
    ConstraintRule::new(VersionPattern::major(1))
        .with_constraint(
            DependencyConstraint::required("node")
                .min("12.0.0")
                .max("22.99.99")
        ),
    // ... 更多规则
]);
```

### 问题

1. **维护困难**: 添加新 Provider 需要修改多个文件
2. **缺乏内聚性**: 约束与 Provider 代码分离
3. **扩展性差**: 无法支持更复杂的依赖场景（如可选依赖、平台特定依赖）
4. **无法覆盖**: 用户无法自定义或覆盖内置约束
5. **无依赖解析**: 当前只是声明约束，没有智能解析本地已安装版本

### 需求分析

1. **声明式定义** - 约束应该是声明式的，而非命令式代码
2. **就近原则** - 约束应该定义在对应的 Provider 目录中
3. **版本感知** - 支持基于 Runtime 版本的条件约束
4. **可扩展** - 支持别名、hooks、平台特定配置等
5. **可覆盖** - 用户可以在项目或用户级别覆盖约束
6. **智能解析** - 检查本地已安装版本，满足约束则复用

## 设计方案

### 完整配置预览

每个 Provider 目录下包含一个 `provider.toml` 文件：

```
crates/vx-providers/yarn/
├── Cargo.toml
├── provider.toml      # 新增: Provider 清单
└── src/
    ├── lib.rs
    └── runtime.rs
```

**完整的 `provider.toml` 示例 (Yarn)**:

```toml
# Provider 元数据
[provider]
name = "yarn"
description = "Fast, reliable, and secure dependency management"
homepage = "https://yarnpkg.com"
repository = "https://github.com/yarnpkg/yarn"
ecosystem = "nodejs"

# 提供的 Runtime 列表
[[runtimes]]
name = "yarn"
description = "Yarn package manager"
executable = "yarn"

# 别名定义
aliases = ["yarnpkg"]

# 可执行文件配置
[runtimes.executable]
extensions = [".cmd", ".exe"]  # Windows 扩展名
dir_pattern = "yarn-v{version}/bin"  # 解压后的目录模式

# 版本获取配置
[runtimes.versions]
source = "github-releases"
owner = "yarnpkg"
repo = "yarn"
strip_v_prefix = true
lts_pattern = "1.22.*"  # LTS 版本模式

# 依赖约束 - 按版本范围定义
# 使用标准 semver 语法
[[runtimes.constraints]]
when = "^1"  # Yarn 1.x (>=1.0.0, <2.0.0)
requires = [
    { runtime = "node", version = ">=12, <23", reason = "Yarn 1.x requires Node.js 12-22 for native module compatibility" }
]

[[runtimes.constraints]]
when = ">=2, <4"  # Yarn 2.x - 3.x
requires = [
    { runtime = "node", version = ">=16", recommended = "20", reason = "Yarn 2.x-3.x requires Node.js 16+" }
]

[[runtimes.constraints]]
when = ">=4"  # Yarn 4.x+
requires = [
    { runtime = "node", version = ">=18", recommended = "22", reason = "Yarn 4.x requires Node.js 18+" }
]

# Hooks 配置
[runtimes.hooks]
pre_run = ["ensure_node_modules"]  # 内置 hook 名称

# 平台特定配置
[runtimes.platforms.windows]
executable_extensions = [".cmd", ".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

### 详细说明

#### 1. Provider 元数据 (`[provider]`)

```toml
[provider]
name = "yarn"                    # Provider 名称 (必填)
description = "..."              # 描述
homepage = "https://..."         # 官网
repository = "https://..."       # 源码仓库
ecosystem = "nodejs"             # 所属生态系统
```

**生态系统枚举**:
- `nodejs`, `python`, `rust`, `go`, `java`, `dotnet`, `system`

#### 2. Runtime 定义 (`[[runtimes]]`)

一个 Provider 可以提供多个 Runtime：

```toml
[[runtimes]]
name = "yarn"
description = "Yarn package manager"
executable = "yarn"
aliases = ["yarnpkg"]

[[runtimes]]
name = "yarnpkg"
description = "Yarn package manager (alias)"
executable = "yarn"
# 这是 yarn 的别名，共享同一个可执行文件
```

#### 3. 版本约束语法 (`[[runtimes.constraints]]`)

**使用标准 npm/Cargo semver 语法** (复用 `vx-resolver::VersionRequest`):

| 语法 | 含义 | 示例 |
|------|------|------|
| `^1.2.3` | 兼容版本 | `>=1.2.3, <2.0.0` |
| `~1.2.3` | 补丁版本 | `>=1.2.3, <1.3.0` |
| `>=1.2.3` | 大于等于 | |
| `<2.0.0` | 小于 | |
| `>=1, <3` | 范围 | |
| `1.2.*` | 通配符 | 匹配 1.2.x |
| `1.2` | 部分版本 | 匹配 1.2.x |
| `1` | 主版本 | 匹配 1.x.x |
| `*` | 任意版本 | |

**`when` 条件语法** - 用于匹配当前 Runtime 的版本:

```toml
[[runtimes.constraints]]
when = "^1"           # 当 yarn 版本是 1.x 时
requires = [...]

[[runtimes.constraints]]
when = ">=2, <4"      # 当 yarn 版本是 2.x 或 3.x 时
requires = [...]

[[runtimes.constraints]]
when = ">=4"          # 当 yarn 版本是 4.x+ 时
requires = [...]

[[runtimes.constraints]]
when = "*"            # 所有版本
requires = [...]
```

**依赖定义**:

```toml
[[runtimes.constraints]]
when = "^1"
requires = [
    { 
        runtime = "node",           # 依赖的 runtime 名称
        version = ">=12, <23",      # 版本约束 (semver)
        recommended = "20",         # 推荐版本 (可选)
        reason = "...",             # 原因说明 (可选)
        optional = false            # 是否可选 (默认 false)
    }
]

# 推荐依赖 (可选，不强制)
recommends = [
    { runtime = "git", version = ">=2.0", reason = "For better source control" }
]
```

#### 4. 依赖解析流程

**核心原则**: 如果本地已安装的版本满足约束，则直接使用，不需要下载新版本。

```
用户执行: vx yarn@1.22.22 install

1. 解析 yarn@1.22.22 的依赖约束
   - 读取 provider.toml
   - 匹配 when = "^1" 的规则
   - 获取: requires = [{ runtime = "node", version = ">=12, <23" }]

2. 检查本地已安装的 node 版本
   - 扫描 ~/.vx/runtimes/node/
   - 发现: 19.0.0, 20.10.0, 22.0.0

3. 版本匹配
   - 19.0.0: 满足 >=12, <23 ✓
   - 20.10.0: 满足 >=12, <23 ✓
   - 22.0.0: 满足 >=12, <23 ✓
   - 选择最新满足约束的版本: 22.0.0

4. 如果没有满足约束的版本
   - 检查 recommended 版本
   - 如果有 recommended = "20"，下载 node@20
   - 否则下载满足约束的最新版本

5. 设置环境并执行
   - PATH 中添加 node@22.0.0
   - 执行 yarn install
```

**解析器伪代码**:

```rust
pub struct DependencyResolver {
    registry: ProviderRegistry,
    installed_versions: InstalledVersionsCache,
}

impl DependencyResolver {
    /// 解析运行时依赖
    pub async fn resolve(
        &self,
        runtime: &str,
        version: &str,
    ) -> Result<Vec<ResolvedDependency>> {
        // 1. 获取约束
        let constraints = self.get_constraints(runtime, version)?;
        
        let mut resolved = Vec::new();
        
        for dep in constraints.requires {
            // 2. 检查本地已安装版本
            let installed = self.installed_versions.get(&dep.runtime);
            
            // 3. 查找满足约束的版本
            let version_req = VersionRequest::parse(&dep.version);
            
            if let Some(matching) = installed
                .iter()
                .filter(|v| version_req.satisfies(v))
                .max()
            {
                // 本地有满足约束的版本，直接使用
                resolved.push(ResolvedDependency {
                    runtime: dep.runtime,
                    version: matching.clone(),
                    source: DependencySource::Local,
                });
            } else if !dep.optional {
                // 需要下载
                let target_version = dep.recommended
                    .unwrap_or_else(|| self.find_latest_matching(&dep.runtime, &version_req));
                    
                resolved.push(ResolvedDependency {
                    runtime: dep.runtime,
                    version: target_version,
                    source: DependencySource::Download,
                });
            }
        }
        
        Ok(resolved)
    }
}
```

#### 5. Hooks 配置 (`[runtimes.hooks]`)

```toml
[runtimes.hooks]
# 内置 hooks
pre_run = ["ensure_node_modules"]
post_install = ["setup_global_bin"]

# 自定义 hooks (未来扩展)
# pre_run_script = "scripts/pre_run.sh"
```

**内置 Hooks**:
- `ensure_node_modules` - 确保 node_modules 已安装
- `setup_global_bin` - 设置全局 bin 目录
- `verify_checksum` - 验证下载文件校验和

#### 6. 平台特定配置 (`[runtimes.platforms.*]`)

```toml
[runtimes.platforms.windows]
executable_extensions = [".cmd", ".exe", ".bat"]
download_url_pattern = "https://.../{version}/yarn-{version}.msi"

[runtimes.platforms.macos]
executable_extensions = []
download_url_pattern = "https://.../{version}/yarn-v{version}.tar.gz"

[runtimes.platforms.linux]
executable_extensions = []
download_url_pattern = "https://.../{version}/yarn-v{version}.tar.gz"
```

### 更多 Provider 示例

#### Node.js Provider

```toml
[provider]
name = "node"
description = "JavaScript runtime built on Chrome's V8 engine"
ecosystem = "nodejs"

[[runtimes]]
name = "node"
description = "Node.js runtime"
executable = "node"
aliases = ["nodejs"]

[runtimes.executable]
extensions = [".exe"]
dir_pattern = "node-v{version}-{platform}-{arch}"

[runtimes.versions]
source = "nodejs-org"  # 专用版本源
lts_pattern = "lts/*"

# Node.js 没有外部依赖
# 但可以声明推荐的配套工具
[[runtimes.constraints]]
when = "*"
recommends = [
    { runtime = "npm", reason = "Default package manager" }
]

[[runtimes]]
name = "npm"
description = "Node Package Manager"
executable = "npm"
bundled_with = "node"  # npm 与 node 捆绑

[[runtimes]]
name = "npx"
description = "Node Package Execute"
executable = "npx"
bundled_with = "node"

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=12" }
]
```

#### pnpm Provider

```toml
[provider]
name = "pnpm"
description = "Fast, disk space efficient package manager"
ecosystem = "nodejs"

[[runtimes]]
name = "pnpm"
executable = "pnpm"
aliases = ["pnpx"]

[runtimes.versions]
source = "github-releases"
owner = "pnpm"
repo = "pnpm"

[[runtimes.constraints]]
when = "^7"
requires = [
    { runtime = "node", version = ">=14", recommended = "18" }
]

[[runtimes.constraints]]
when = "^8"
requires = [
    { runtime = "node", version = ">=16", recommended = "20" }
]

[[runtimes.constraints]]
when = ">=9"
requires = [
    { runtime = "node", version = ">=18", recommended = "22" }
]
```

### 架构变更

#### 与 vx-extension 复用

`vx-extension` 已经有类似的配置解析能力，我们应该复用底层设计：

```
共享组件:
├── vx-manifest/              # 新增: 通用清单解析库
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── version_spec.rs   # 复用 vx-resolver::VersionRequest
│       ├── provider.rs       # Provider 清单类型
│       └── extension.rs      # Extension 清单类型 (重构自 vx-extension)
│
├── vx-extension/
│   └── src/
│       └── config.rs         # 使用 vx-manifest::extension
│
└── vx-runtime/
    └── src/
        ├── constraints.rs    # 重构: 从清单加载约束
        └── manifest.rs       # 使用 vx-manifest::provider
```

**共享的版本约束解析**:

```rust
// vx-manifest/src/version_spec.rs
// 直接复用 vx-resolver 的 VersionRequest

pub use vx_resolver::{
    VersionRequest,
    VersionConstraint,
    RangeConstraint,
    RangeOp,
    Version,
};

/// 扩展 VersionRequest 以支持清单中的版本约束
impl VersionRequest {
    /// 检查一个版本是否满足此约束
    pub fn satisfies(&self, version: &str) -> bool {
        let v = match Version::parse(version) {
            Some(v) => v,
            None => return false,
        };
        
        match &self.constraint {
            VersionConstraint::Exact(target) => &v == target,
            VersionConstraint::Partial { major, minor } => {
                v.major == *major && v.minor == *minor
            }
            VersionConstraint::Major(major) => v.major == *major,
            VersionConstraint::Latest => true,
            VersionConstraint::Any => true,
            VersionConstraint::Range(constraints) => {
                constraints.iter().all(|c| c.satisfies(&v))
            }
            VersionConstraint::Wildcard { major, minor } => {
                v.major == *major && v.minor == *minor
            }
            VersionConstraint::Caret(target) => {
                RangeConstraint::new(RangeOp::Caret, target.clone()).satisfies(&v)
            }
            VersionConstraint::Tilde(target) => {
                RangeConstraint::new(RangeOp::Tilde, target.clone()).satisfies(&v)
            }
            _ => true,
        }
    }
}
```

#### 类型定义

```rust
// vx-manifest/src/provider.rs

use serde::{Deserialize, Serialize};
use vx_resolver::VersionRequest;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderManifest {
    pub provider: ProviderMeta,
    #[serde(default)]
    pub runtimes: Vec<RuntimeDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub ecosystem: Option<Ecosystem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub executable: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub bundled_with: Option<String>,
    #[serde(default)]
    pub constraints: Vec<ConstraintRule>,
    #[serde(default)]
    pub hooks: Option<HooksDef>,
    #[serde(default)]
    pub platforms: Option<PlatformsDef>,
    #[serde(default)]
    pub versions: Option<VersionSourceDef>,
    #[serde(default, rename = "executable")]
    pub executable_config: Option<ExecutableConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConstraintRule {
    /// 版本条件 (semver 语法)
    pub when: String,
    /// 平台条件 (可选)
    #[serde(default)]
    pub platform: Option<String>,
    /// 必需依赖
    #[serde(default)]
    pub requires: Vec<DependencyDef>,
    /// 推荐依赖
    #[serde(default)]
    pub recommends: Vec<DependencyDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DependencyDef {
    pub runtime: String,
    pub version: String,
    #[serde(default)]
    pub recommended: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub optional: bool,
}

impl ConstraintRule {
    /// 检查此规则是否适用于给定版本
    pub fn matches(&self, version: &str) -> bool {
        let req = VersionRequest::parse(&self.when);
        req.satisfies(version)
    }
}

impl DependencyDef {
    /// 转换为 VersionRequest
    pub fn version_request(&self) -> VersionRequest {
        VersionRequest::parse(&self.version)
    }
    
    /// 检查给定版本是否满足此依赖约束
    pub fn satisfies(&self, version: &str) -> bool {
        self.version_request().satisfies(version)
    }
}
```

#### 加载流程

```rust
// vx-runtime/src/manifest.rs

impl ConstraintsRegistry {
    /// 从 Provider 目录加载清单
    pub fn load_from_providers(providers_dir: &Path) -> Result<Self> {
        let mut registry = Self::new();
        
        for entry in fs::read_dir(providers_dir)? {
            let provider_dir = entry?.path();
            let manifest_path = provider_dir.join("provider.toml");
            
            if manifest_path.exists() {
                let manifest = ProviderManifest::load(&manifest_path)?;
                registry.register_from_manifest(&manifest)?;
            }
        }
        
        Ok(registry)
    }
    
    fn register_from_manifest(&mut self, manifest: &ProviderManifest) -> Result<()> {
        for runtime in &manifest.runtimes {
            self.register_runtime(&runtime.name, &runtime.constraints);
        }
        Ok(())
    }
}
```

### 用户覆盖

用户可以在 `~/.vx/providers/` 或项目 `.vx/providers/` 中创建覆盖文件：

```toml
# ~/.vx/providers/yarn.override.toml

# 覆盖特定约束
[[constraints]]
when = "^1"
requires = [
    # 公司内部使用 Node 14-20
    { runtime = "node", version = ">=14, <21" }
]

# 添加额外约束
[[constraints]]
when = "*"
requires = [
    { runtime = "git", version = ">=2.0", optional = true }
]
```

**加载优先级**:
1. 项目级 `.vx/providers/*.override.toml`
2. 用户级 `~/.vx/providers/*.override.toml`
3. 内置 `provider.toml`

## 向后兼容性

### 兼容策略

1. **渐进式迁移** - 保留现有 `ConstraintsRegistry` 作为后备
2. **自动检测** - 优先使用 `provider.toml`，不存在时回退到代码定义
3. **警告提示** - 对使用旧方式定义约束的 Provider 发出警告

### 代码迁移

**迁移前** (constraints.rs):
```rust
self.register("yarn", vec![
    ConstraintRule::new(VersionPattern::major(1))
        .with_constraint(
            DependencyConstraint::required("node")
                .min("12.0.0")
                .max("22.99.99")
        ),
]);
```

**迁移后** (provider.toml):
```toml
[[runtimes.constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=12, <23" }
]
```

## 实现计划

### Phase 1: 核心基础 (v0.8.0)

- [x] 创建 `vx-manifest` crate
- [x] 实现版本约束解析 (独立实现，避免循环依赖)
- [x] 实现 `ProviderManifest` 类型定义和解析
- [x] 实现 `ManifestLoader` 加载器
- [x] 集成到 `ConstraintsRegistry`
- [x] 迁移 yarn, npm, pnpm 的约束到 `provider.toml`

### Phase 2: 完整迁移 (v0.9.0)

- [x] 迁移所有 Provider 到 `provider.toml`
- [x] 删除 `constraints.rs` 中的硬编码约束
- [x] 实现用户覆盖机制
- [x] 重构 `vx-extension` 使用 `vx-manifest`

### Phase 3: 高级特性 (v0.10.0)

- [ ] 平台特定约束
- [ ] 自定义 hooks 脚本
- [ ] 约束冲突检测和报告
- [ ] 依赖图可视化

## 替代方案

### 方案 A: 继续使用 Rust 代码

保持现有的 `ConstraintsRegistry` 方式。

**优点**: 无需额外解析，类型安全
**缺点**: 维护困难，无法用户覆盖

### 方案 B: 使用 JSON Schema

使用 JSON 而非 TOML。

**优点**: 更广泛的工具支持
**缺点**: 可读性差，注释不友好

### 方案 C: 嵌入 Cargo.toml

将约束定义在 Provider 的 `Cargo.toml` 中。

**优点**: 不需要额外文件
**缺点**: 混淆 Rust 依赖和 Runtime 依赖

## 参考资料

### 主流项目源码

- [Spack Packaging Guide](https://spack.readthedocs.io/en/latest/packaging_guide.html) - depends_on 和 variant 语法
- [Rez Package Definition Guide](https://github.com/AcademySoftwareFoundation/rez/wiki/Package-Definition-Guide) - requires 和 tools 定义
- [npm semver](https://github.com/npm/node-semver) - 版本范围语法标准
- [Cargo Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html) - TOML 配置参考

### 内部复用

- `vx-resolver::VersionRequest` - 版本约束解析
- `vx-resolver::VersionConstraint` - 约束类型定义
- `vx-extension::config` - 扩展配置解析模式

### 依赖库

- [`toml`](https://crates.io/crates/toml) - TOML 解析
- [`serde`](https://crates.io/crates/serde) - 序列化/反序列化

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-01-06 | Draft | 初始草案 |
| 2026-01-06 | Draft v2 | 更新版本语法为标准 semver；添加依赖解析流程；复用 vx-resolver；移除 vx provider 命令 |
| 2026-01-06 | Phase 1 完成 | 所有 34 个 Provider 已迁移到 provider.toml，包含完整的 constraints 定义 |
| 2026-01-06 | Phase 2 完成 | 实现用户覆盖机制（`.override.toml` 文件支持）；重构 `vx-extension` 使用 `vx-manifest` 的 `VersionRequest` 进行版本约束检查 |
