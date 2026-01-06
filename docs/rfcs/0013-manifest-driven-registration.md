# RFC 0013: Manifest-Driven Provider Registration

> **状态**: Implementing (Phase 1 完成)
> **作者**: vx team
> **创建日期**: 2026-01-06
> **目标版本**: v0.9.0
> **依赖**: RFC 0012 (Provider Manifest)

## 摘要

基于 RFC 0012 引入的 `provider.toml` 清单系统，本 RFC 提出通过清单驱动的方式自动注册 Provider，消除 CLI 中的硬编码注册代码，实现真正的"零代码"Provider 发现和注册。

## Provider vs Extension：核心区别

在深入设计之前，需要明确 **Provider** 和 **Extension** 的本质区别：

### 概念对比

| 维度 | Provider | Extension |
|------|----------|-----------|
| **定位** | 运行时管理器 | 功能扩展脚本 |
| **实现语言** | Rust (编译型) | Python/Node.js/等 (脚本型) |
| **执行方式** | 直接调用，原生性能 | 通过解释器执行 |
| **核心职责** | 下载、安装、版本管理运行时 | 提供 CLI 命令、hooks、工具 |
| **配置文件** | `provider.toml` | `vx-extension.toml` |
| **存放位置** | `crates/vx-providers/` (内置) | `~/.vx/extensions/` (用户) |
| **调用方式** | `vx node`, `vx yarn install` | `vx x my-ext`, `vx ext run my-ext` |

### 架构层次

```
┌─────────────────────────────────────────────────────────────────────┐
│                           vx CLI                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────┐          ┌─────────────────────┐           │
│  │     Extensions      │          │     Providers       │           │
│  │  (vx-extension)     │          │   (vx-runtime)      │           │
│  ├─────────────────────┤          ├─────────────────────┤           │
│  │ • 脚本执行器        │          │ • 运行时管理        │           │
│  │ • 命令扩展          │ 依赖于   │ • 版本下载/安装     │           │
│  │ • Hooks 系统        │ ───────> │ • 依赖约束解析      │           │
│  │ • 远程安装          │          │ • 环境配置          │           │
│  └─────────────────────┘          └─────────────────────┘           │
│           │                                 │                        │
│           ▼                                 ▼                        │
│  ┌─────────────────────┐          ┌─────────────────────┐           │
│  │ vx-extension.toml   │          │   provider.toml     │           │
│  │ • runtime: python   │          │ • ecosystem: nodejs │           │
│  │ • entrypoint: main  │          │ • runtimes: [...]   │           │
│  │ • commands: {...}   │          │ • constraints: [...] │           │
│  └─────────────────────┘          └─────────────────────┘           │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

### 关键区别详解

#### 1. Extension 依赖 Provider

Extension 需要 Provider 提供的运行时来执行：

```toml
# vx-extension.toml
[extension]
name = "my-linter"
type = "command"

[runtime]
requires = "python >= 3.10"  # 依赖 Python Provider
dependencies = ["ruff", "black"]

[entrypoint]
main = "main.py"
```

当执行 `vx x my-linter` 时：
1. Extension 系统读取 `requires = "python >= 3.10"`
2. 调用 **Python Provider** 获取/安装 Python 3.10+
3. 使用该 Python 执行 `main.py`

#### 2. Provider 是原生 Rust 代码

Provider 直接编译进 vx 二进制（或作为动态库），提供：

```rust
// vx-provider-node/src/lib.rs
pub struct NodeProvider;

impl Provider for NodeProvider {
    fn name(&self) -> &str { "node" }
    
    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(NodeRuntime::new()),
            Arc::new(NpmRuntime::new()),
            Arc::new(NpxRuntime::new()),
        ]
    }
}

impl Runtime for NodeRuntime {
    async fn install(&self, version: &str) -> Result<PathBuf>;
    async fn list_versions(&self) -> Result<Vec<String>>;
    fn executable_name(&self) -> &str { "node" }
}
```

#### 3. 性能差异

| 操作 | Provider | Extension |
|------|----------|-----------|
| 启动时间 | ~0ms (已编译) | ~50-200ms (解释器启动) |
| 执行开销 | 直接系统调用 | 进程间通信 |
| 内存占用 | 共享 vx 进程 | 独立解释器进程 |
| 适用场景 | 高频操作 (版本检测、路径解析) | 低频操作 (工具命令) |

### 为什么不合并？

1. **性能要求不同**：Provider 需要毫秒级响应（每次 `vx node` 都要检测版本），Extension 可以接受秒级延迟

2. **开发门槛不同**：
   - Provider：需要 Rust 知识，编译发布
   - Extension：任何脚本语言，直接放入目录即可

3. **职责边界清晰**：
   - Provider：管理工具本身（node、python、go）
   - Extension：基于工具构建功能（linter、formatter、deploy）

4. **生命周期不同**：
   - Provider：随 vx 版本发布，相对稳定
   - Extension：用户随时安装/更新，快速迭代

## 动机

### 当前状态分析

目前，Provider 注册在 `vx-cli/src/registry.rs` 中硬编码：

```rust
pub fn create_registry() -> ProviderRegistry {
    let registry = ProviderRegistry::new();

    // 33 个 Provider 的硬编码注册
    registry.register(vx_provider_node::create_provider());
    registry.register(vx_provider_go::create_provider());
    registry.register(vx_provider_rust::create_provider());
    // ... 30 more lines
    
    registry
}
```

**问题**：

1. **样板代码过多** - 每个新 Provider 需要添加 3 处代码：
   - `Cargo.toml` 依赖
   - `registry.rs` 中的 `use` 语句
   - `registry.rs` 中的 `register()` 调用

2. **编译时耦合** - CLI crate 必须依赖所有 Provider crate，导致：
   - 编译时间长
   - 二进制体积大
   - 无法按需加载

3. **无法动态扩展** - 用户无法添加自定义 Provider

4. **信息冗余** - Provider 元数据（name, description, aliases）在代码和 `provider.toml` 中重复定义

### 目标

1. **自动发现** - 扫描 `provider.toml` 自动注册 Provider
2. **延迟加载** - 只在需要时加载 Provider 代码
3. **减少样板** - 新增 Provider 只需创建 `provider.toml`
4. **支持扩展** - 用户可通过 `~/.vx/providers/` 添加自定义 Provider

## 性能分析

### 当前性能基线

```
vx --version 启动时间分析 (33 个 Provider):

┌────────────────────────────────────────────────────────────────┐
│ 阶段                    │ 耗时      │ 占比    │ 说明          │
├────────────────────────────────────────────────────────────────┤
│ 进程启动                │ ~5ms      │ 25%     │ OS 加载二进制 │
│ Provider 初始化         │ ~8ms      │ 40%     │ 33 个 Provider│
│   - 创建实例            │   ~3ms    │         │               │
│   - 注册到 Registry     │   ~5ms    │         │               │
│ CLI 解析                │ ~4ms      │ 20%     │ clap 解析     │
│ 其他                    │ ~3ms      │ 15%     │               │
├────────────────────────────────────────────────────────────────┤
│ 总计                    │ ~20ms     │ 100%    │               │
└────────────────────────────────────────────────────────────────┘
```

### Phase 1 优化：编译时清单嵌入

```
优化后启动时间:

┌────────────────────────────────────────────────────────────────┐
│ 阶段                    │ 耗时      │ 变化    │ 说明          │
├────────────────────────────────────────────────────────────────┤
│ 进程启动                │ ~5ms      │ 不变    │               │
│ Provider 初始化         │ ~6ms      │ -25%    │               │
│   - 清单解析 (缓存)     │   ~1ms    │ 新增    │ 编译时嵌入    │
│   - 创建实例            │   ~3ms    │ 不变    │               │
│   - 注册 (元数据复用)   │   ~2ms    │ -60%    │ 从清单读取    │
│ CLI 解析                │ ~4ms      │ 不变    │               │
│ 其他                    │ ~3ms      │ 不变    │               │
├────────────────────────────────────────────────────────────────┤
│ 总计                    │ ~18ms     │ -10%    │               │
└────────────────────────────────────────────────────────────────┘

主要收益: 减少重复元数据构建，代码更简洁
```

### Phase 2 优化：延迟加载

```
延迟加载后启动时间 (典型场景: vx node --version):

┌────────────────────────────────────────────────────────────────┐
│ 阶段                    │ 耗时      │ 变化    │ 说明          │
├────────────────────────────────────────────────────────────────┤
│ 进程启动                │ ~5ms      │ 不变    │               │
│ 清单索引加载            │ ~2ms      │ 新增    │ 只读元数据    │
│ 目标 Provider 加载      │ ~1ms      │ 新增    │ 动态库加载    │
│ CLI 解析                │ ~4ms      │ 不变    │               │
│ 其他                    │ ~3ms      │ 不变    │               │
├────────────────────────────────────────────────────────────────┤
│ 总计                    │ ~15ms     │ -25%    │               │
└────────────────────────────────────────────────────────────────┘

关键优化: 不再初始化所有 33 个 Provider，只加载需要的
```

### 内存占用对比

```
┌─────────────────────────────────────────────────────────────────┐
│ 场景                      │ 当前      │ Phase 2   │ 节省      │
├─────────────────────────────────────────────────────────────────┤
│ vx --help                 │ ~8MB      │ ~4MB      │ 50%       │
│ vx node --version         │ ~8MB      │ ~5MB      │ 37%       │
│ vx install node go rust   │ ~8MB      │ ~6MB      │ 25%       │
│ 使用全部 33 个 Provider   │ ~8MB      │ ~8MB      │ 0%        │
└─────────────────────────────────────────────────────────────────┘

说明: 大多数用户只使用 3-5 个 Provider，延迟加载显著减少内存占用
```

### 编译时间对比

```
┌─────────────────────────────────────────────────────────────────┐
│ 构建类型                  │ 当前      │ Phase 2   │ 变化      │
├─────────────────────────────────────────────────────────────────┤
│ 完整构建 (release)        │ ~180s     │ ~120s     │ -33%      │
│ 增量构建 (单 Provider)    │ ~45s      │ ~15s      │ -67%      │
│ vx-cli 单独构建           │ ~60s      │ ~20s      │ -67%      │
└─────────────────────────────────────────────────────────────────┘

说明: 解耦后，修改单个 Provider 不需要重新链接所有依赖
```

### 二进制体积对比

```
┌─────────────────────────────────────────────────────────────────┐
│ 组件                      │ 当前      │ Phase 2   │ 说明      │
├─────────────────────────────────────────────────────────────────┤
│ vx 主程序                 │ ~15MB     │ ~8MB      │ 核心功能  │
│ Provider 插件 (总计)      │ -         │ ~12MB     │ 按需加载  │
│   - node.so               │ -         │ ~1.2MB    │           │
│   - go.so                 │ -         │ ~0.8MB    │           │
│   - python.so             │ -         │ ~1.0MB    │           │
│   - ...                   │ -         │ ...       │           │
├─────────────────────────────────────────────────────────────────┤
│ 完整安装                  │ ~15MB     │ ~20MB     │ +33%      │
│ 最小安装 (核心+3插件)     │ ~15MB     │ ~11MB     │ -27%      │
└─────────────────────────────────────────────────────────────────┘

说明: 支持按需安装插件，用户可选择只安装需要的 Provider
```

## 设计方案

### 架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│                         vx-cli                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  ProviderRegistry                        │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │    │
│  │  │ BuiltinSlot │  │ BuiltinSlot │  │  UserSlot   │      │    │
│  │  │   (node)    │  │   (yarn)    │  │  (custom)   │      │    │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘      │    │
│  └─────────┼────────────────┼────────────────┼─────────────┘    │
└────────────┼────────────────┼────────────────┼──────────────────┘
             │                │                │
             ▼                ▼                ▼
┌────────────────────┐ ┌────────────────┐ ┌────────────────┐
│  provider.toml     │ │  provider.toml │ │  provider.toml │
│  (builtin/node)    │ │  (builtin/yarn)│ │  (~/.vx/...)   │
└────────────────────┘ └────────────────┘ └────────────────┘
```

### Phase 1: 静态编译 + 清单元数据 (v0.9.0)

保持现有的静态编译模式，但从 `provider.toml` 读取元数据：

#### 1.1 清单增强

扩展 `provider.toml` 支持 CLI 注册所需的信息：

```toml
[provider]
name = "yarn"
description = "Fast, reliable, and secure dependency management"
ecosystem = "nodejs"

# CLI 注册配置
[provider.cli]
# 是否默认启用（可用于禁用某些 Provider）
enabled = true
# 加载优先级（数字越小优先级越高）
priority = 100
# 平台限制
platforms = ["windows", "macos", "linux"]  # 默认全平台

[[runtimes]]
name = "yarn"
executable = "yarn"
aliases = ["yarnpkg"]
# ...
```

#### 1.2 编译时清单收集

使用 `build.rs` 在编译时收集所有 `provider.toml`：

```rust
// vx-cli/build.rs
use std::fs;
use std::path::Path;

fn main() {
    let providers_dir = Path::new("../vx-providers");
    let mut manifests = Vec::new();
    
    for entry in fs::read_dir(providers_dir).unwrap() {
        let provider_dir = entry.unwrap().path();
        let manifest_path = provider_dir.join("provider.toml");
        
        if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path).unwrap();
            let name = provider_dir.file_name().unwrap().to_str().unwrap();
            manifests.push((name.to_string(), content));
        }
    }
    
    // 生成 manifests.rs
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("manifests.rs");
    
    let code = generate_manifest_code(&manifests);
    fs::write(dest_path, code).unwrap();
    
    println!("cargo:rerun-if-changed=../vx-providers");
}

fn generate_manifest_code(manifests: &[(String, String)]) -> String {
    let mut code = String::from("pub static PROVIDER_MANIFESTS: &[(&str, &str)] = &[\n");
    for (name, content) in manifests {
        code.push_str(&format!("    (\"{}\", r#\"{}\"#),\n", name, content));
    }
    code.push_str("];\n");
    code
}
```

#### 1.3 简化注册代码

```rust
// vx-cli/src/registry.rs

use vx_manifest::{ManifestLoader, ProviderManifest};

// 编译时嵌入的清单
include!(concat!(env!("OUT_DIR"), "/manifests.rs"));

/// Provider 工厂函数类型
type ProviderFactory = fn() -> Box<dyn Provider>;

/// 内置 Provider 注册表（编译时确定）
static BUILTIN_PROVIDERS: &[(&str, ProviderFactory)] = &[
    ("node", || Box::new(vx_provider_node::create_provider())),
    ("go", || Box::new(vx_provider_go::create_provider())),
    ("rust", || Box::new(vx_provider_rust::create_provider())),
    // ... 其他内置 Provider
];

pub fn create_registry() -> ProviderRegistry {
    let registry = ProviderRegistry::new();
    
    // 1. 加载清单元数据
    let manifests = load_builtin_manifests();
    
    // 2. 按优先级排序
    let mut providers: Vec<_> = manifests.iter()
        .filter(|m| m.provider.cli.as_ref().map_or(true, |c| c.enabled))
        .filter(|m| is_platform_supported(m))
        .collect();
    providers.sort_by_key(|m| m.provider.cli.as_ref().map_or(100, |c| c.priority));
    
    // 3. 注册 Provider
    for manifest in providers {
        if let Some(factory) = get_provider_factory(&manifest.provider.name) {
            registry.register_with_manifest(factory(), manifest);
        }
    }
    
    registry
}

fn load_builtin_manifests() -> Vec<ProviderManifest> {
    PROVIDER_MANIFESTS
        .iter()
        .filter_map(|(name, content)| {
            ProviderManifest::from_str(content).ok()
        })
        .collect()
}

fn get_provider_factory(name: &str) -> Option<ProviderFactory> {
    BUILTIN_PROVIDERS.iter()
        .find(|(n, _)| *n == name)
        .map(|(_, f)| *f)
}
```

#### 1.4 使用宏进一步简化

```rust
// vx-cli/src/registry.rs

macro_rules! register_providers {
    ($($name:ident),* $(,)?) => {
        fn create_builtin_provider(name: &str) -> Option<Box<dyn Provider>> {
            match name {
                $(
                    stringify!($name) => Some(Box::new(
                        paste::paste! { [<vx_provider_ $name>]::create_provider() }
                    )),
                )*
                _ => None,
            }
        }
    };
}

register_providers!(
    node, go, rust, uv, bun, pnpm, yarn, vscode, just, vite,
    rez, deno, zig, java, terraform, kubectl, helm, rcedit,
    git, choco, docker, awscli, azcli, gcloud, ninja, cmake,
    protoc, task, pre_commit, ollama, spack, release_please,
    python, msvc,
);
```

### Phase 2: 延迟加载 (v0.10.0)

使用动态库实现真正的延迟加载：

#### 2.1 Provider 插件接口

```rust
// vx-core/src/plugin.rs

/// Provider 插件接口
pub trait ProviderPlugin: Send + Sync {
    /// 获取 Provider 实例
    fn create_provider(&self) -> Box<dyn Provider>;
    
    /// 获取插件版本
    fn version(&self) -> &str;
}

/// 插件入口点宏
#[macro_export]
macro_rules! declare_provider_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn _vx_create_plugin() -> *mut dyn ProviderPlugin {
            let plugin = Box::new(<$plugin_type>::new());
            Box::into_raw(plugin)
        }
        
        #[no_mangle]
        pub extern "C" fn _vx_plugin_version() -> *const std::os::raw::c_char {
            concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const _
        }
    };
}
```

#### 2.2 插件加载器

```rust
// vx-runtime/src/plugin_loader.rs

use libloading::{Library, Symbol};

pub struct PluginLoader {
    loaded: RwLock<HashMap<String, LoadedPlugin>>,
    search_paths: Vec<PathBuf>,
}

struct LoadedPlugin {
    library: Library,
    plugin: Box<dyn ProviderPlugin>,
}

impl PluginLoader {
    /// 延迟加载 Provider
    pub fn load(&self, name: &str) -> Result<&dyn Provider> {
        // 检查是否已加载
        if let Some(loaded) = self.loaded.read().unwrap().get(name) {
            return Ok(loaded.plugin.create_provider().as_ref());
        }
        
        // 查找插件文件
        let plugin_path = self.find_plugin_path(name)?;
        
        // 加载动态库
        unsafe {
            let library = Library::new(&plugin_path)?;
            
            let create_fn: Symbol<extern "C" fn() -> *mut dyn ProviderPlugin> =
                library.get(b"_vx_create_plugin")?;
            
            let plugin = Box::from_raw(create_fn());
            
            self.loaded.write().unwrap().insert(name.to_string(), LoadedPlugin {
                library,
                plugin,
            });
        }
        
        Ok(self.loaded.read().unwrap().get(name).unwrap().plugin.create_provider().as_ref())
    }
    
    fn find_plugin_path(&self, name: &str) -> Result<PathBuf> {
        let lib_name = format!("libvx_provider_{}.{}", name, std::env::consts::DLL_EXTENSION);
        
        for dir in &self.search_paths {
            let path = dir.join(&lib_name);
            if path.exists() {
                return Ok(path);
            }
        }
        
        Err(anyhow!("Plugin not found: {}", name))
    }
}
```

#### 2.3 混合注册模式

```rust
// vx-cli/src/registry.rs

pub fn create_registry() -> ProviderRegistry {
    let registry = ProviderRegistry::new();
    let loader = PluginLoader::new();
    
    // 1. 发现所有清单（内置 + 用户）
    let manifests = discover_all_manifests();
    
    // 2. 注册清单元数据（不加载实际代码）
    for manifest in &manifests {
        registry.register_manifest(manifest);
    }
    
    // 3. 设置延迟加载器
    registry.set_plugin_loader(loader);
    
    registry
}

impl ProviderRegistry {
    /// 获取 Runtime（触发延迟加载）
    pub fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        // 1. 查找 Runtime 对应的 Provider
        let provider_name = self.runtime_map.read().unwrap().get(name)?.clone();
        
        // 2. 检查 Provider 是否已加载
        if let Some(entry) = self.providers.read().unwrap()
            .iter()
            .find(|e| e.meta.name == provider_name)
        {
            if let Some(provider) = &entry.provider {
                return provider.get_runtime(name);
            }
        }
        
        // 3. 延迟加载 Provider
        if let Some(loader) = &self.plugin_loader {
            if let Ok(provider) = loader.load(&provider_name) {
                // 更新注册表
                self.set_provider(&provider_name, provider);
                return provider.get_runtime(name);
            }
        }
        
        None
    }
}
```

### Phase 3: 用户自定义 Provider (v0.11.0)

支持用户通过 `~/.vx/providers/` 添加自定义 Provider：

#### 3.1 目录结构

```
~/.vx/providers/
├── custom-tool/
│   ├── provider.toml       # 清单定义
│   └── plugin.so           # 可选：自定义插件
├── my-node/
│   └── provider.toml       # 覆盖内置 node 配置
└── index.toml              # 可选：全局配置
```

#### 3.2 简单 Provider（无代码）

对于只需要下载和执行的工具，可以完全通过 `provider.toml` 定义：

```toml
# ~/.vx/providers/ripgrep/provider.toml

[provider]
name = "ripgrep"
description = "A line-oriented search tool"
ecosystem = "system"

[[runtimes]]
name = "rg"
executable = "rg"
aliases = ["ripgrep"]

[runtimes.versions]
source = "github-releases"
owner = "BurntSushi"
repo = "ripgrep"
strip_v_prefix = true
asset_pattern = "ripgrep-{version}-{arch}-{platform}.tar.gz"

[runtimes.install]
type = "archive"
bin_dir = "ripgrep-{version}-{arch}-{platform}"
```

#### 3.3 Provider 发现流程

```rust
fn discover_all_manifests() -> Vec<ProviderManifest> {
    let mut manifests = Vec::new();
    
    // 1. 内置清单（编译时嵌入）
    manifests.extend(load_builtin_manifests());
    
    // 2. 系统级清单
    if let Some(system_dir) = system_providers_dir() {
        manifests.extend(load_manifests_from_dir(&system_dir));
    }
    
    // 3. 用户级清单（可覆盖内置）
    if let Some(user_dir) = user_providers_dir() {
        for manifest in load_manifests_from_dir(&user_dir) {
            // 覆盖同名 Provider
            if let Some(idx) = manifests.iter().position(|m| m.provider.name == manifest.provider.name) {
                manifests[idx] = manifest;
            } else {
                manifests.push(manifest);
            }
        }
    }
    
    manifests
}
```

## 优势分析

### 代码量对比

| 指标 | 当前 | Phase 1 | Phase 2 |
|------|------|---------|---------|
| registry.rs 行数 | 115 | ~50 | ~30 |
| 新增 Provider 需改动文件数 | 3 | 1 | 1 |
| 编译时依赖数 | 33 | 33 | 按需 |
| 支持用户自定义 | ✗ | ✗ | ✓ |

### 开发体验

**当前流程（新增 Provider）**：
1. 创建 Provider crate
2. 编辑 `vx-cli/Cargo.toml` 添加依赖
3. 编辑 `vx-cli/src/registry.rs` 添加 use 和 register
4. 创建 `provider.toml`

**Phase 1 后**：
1. 创建 Provider crate
2. 创建 `provider.toml`
3. 在 `register_providers!` 宏中添加名称

**Phase 2 后**：
1. 创建 Provider crate
2. 创建 `provider.toml`
3. 编译为插件（自动发现）

## 与 Extension 系统的协同

### 统一发现机制

Provider 和 Extension 可以共享部分发现逻辑：

```rust
// vx-discovery/src/lib.rs

pub trait Discoverable {
    type Config;
    fn config_filename() -> &'static str;
    fn parse_config(content: &str) -> Result<Self::Config>;
}

impl Discoverable for ProviderManifest {
    type Config = ProviderManifest;
    fn config_filename() -> &'static str { "provider.toml" }
    fn parse_config(content: &str) -> Result<Self::Config> {
        toml::from_str(content)
    }
}

impl Discoverable for ExtensionConfig {
    type Config = ExtensionConfig;
    fn config_filename() -> &'static str { "vx-extension.toml" }
    fn parse_config(content: &str) -> Result<Self::Config> {
        toml::from_str(content)
    }
}
```

### 搜索路径统一

```
~/.vx/
├── providers/           # 用户 Provider
│   └── custom-tool/
│       └── provider.toml
├── extensions/          # 用户 Extension
│   └── my-linter/
│       └── vx-extension.toml
├── extensions-dev/      # 开发中的 Extension
└── plugins/             # Provider 动态库
    └── libvx_provider_custom.so
```

### 互操作场景

Extension 可以声明对特定 Provider 的依赖：

```toml
# vx-extension.toml
[extension]
name = "node-tools"

[runtime]
requires = "node >= 18"

# 新增: Provider 级别依赖
[provider_requires]
node = ">= 0.8.0"  # 需要 vx 的 node provider >= 0.8.0
```

## 替代方案

### 方案 A: 保持现状

继续使用硬编码注册。

**优点**: 无需改动
**缺点**: 维护成本高，无法扩展

### 方案 B: 代码生成

使用 `build.rs` 生成完整的注册代码。

**优点**: 编译时完成，类型安全
**缺点**: 仍需编译时依赖所有 Provider

### 方案 C: 合并 Provider 和 Extension

将 Provider 也用脚本实现。

**优点**: 统一架构
**缺点**: 性能损失严重，不适合高频操作

## 向后兼容性

1. **API 兼容** - `ProviderRegistry::register()` 继续工作
2. **行为兼容** - 所有内置 Provider 默认启用
3. **渐进迁移** - 可逐步迁移 Provider

## 实现计划

### v0.9.0 (Phase 1)
- [x] 添加 `vx-cli/build.rs` 清单收集
- [x] 扩展 `ProviderManifest` CLI 配置
- [x] 重构 `registry.rs` 使用清单
- [x] 添加 `register_providers!` 宏
- [ ] 更新文档

### v0.10.0 (Phase 2)
- [ ] 实现 `ProviderPlugin` trait
- [ ] 实现 `PluginLoader`
- [ ] 支持延迟加载
- [ ] 性能测试和优化

### v0.11.0 (Phase 3)
- [ ] 用户 Provider 目录支持
- [ ] 无代码 Provider 支持
- [ ] Provider 市场/索引

## 参考资料

- [RFC 0012: Provider Manifest](./0012-provider-manifest.md)
- [Rust Plugin System Patterns](https://adventures.michaelfbryan.com/posts/plugins-in-rust/)
- [libloading crate](https://crates.io/crates/libloading)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-01-06 | Draft | 初始草案 |
| 2026-01-06 | Draft v2 | 添加 Provider vs Extension 对比；添加性能分析；添加与 Extension 协同设计 |
