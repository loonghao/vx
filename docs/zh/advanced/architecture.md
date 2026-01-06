# 架构

vx 的内部架构概述。

## 高层架构

```
┌─────────────────────────────────────────────────────────────┐
│                        vx CLI                               │
├─────────────────────────────────────────────────────────────┤
│ 命令  │ 配置  │ UI  │ Shell 集成                            │
├─────────────────────────────────────────────────────────────┤
│                     vx-resolver                             │
│ 版本解析 │ 依赖图 │ 执行器                                   │
├─────────────────────────────────────────────────────────────┤
│                     vx-runtime                              │
│ Provider 注册表 │ Manifest 注册表 │ 运行时上下文             │
├─────────────────────────────────────────────────────────────┤
│                     vx-providers                            │
│ Node │ Go │ Rust │ UV │ Deno │ ... (可插拔)                 │
├─────────────────────────────────────────────────────────────┤
│                      vx-core                                │
│ 类型 │ Traits │ 工具 │ 平台抽象                             │
├─────────────────────────────────────────────────────────────┤
│                      vx-paths                               │
│ 路径管理 │ 存储 │ 环境 │ 缓存                               │
└─────────────────────────────────────────────────────────────┘
```

## Crate 结构

### vx-core

所有 crate 共享的核心类型和 traits。

```
vx-core/
├── src/
│   ├── lib.rs
│   ├── types.rs      # 通用类型
│   ├── traits.rs     # 核心 traits
│   ├── error.rs      # 错误类型
│   └── platform.rs   # 平台检测
```

### vx-paths

路径管理和目录结构。

```
vx-paths/
├── src/
│   ├── lib.rs
│   ├── manager.rs    # PathManager
│   ├── store.rs      # 版本存储
│   ├── envs.rs       # 环境
│   └── cache.rs      # 缓存管理
```

### vx-runtime

运行时管理和 Provider 注册。

```
vx-runtime/
├── src/
│   ├── lib.rs
│   ├── registry.rs          # ProviderRegistry
│   ├── manifest_registry.rs # ManifestRegistry（清单驱动）
│   ├── context.rs           # RuntimeContext
│   ├── provider.rs          # Provider trait
│   └── runtime.rs           # 运行时信息
```

#### ManifestRegistry

`ManifestRegistry` 提供清单驱动的 Provider 注册，支持延迟加载和元数据查询：

```rust
// 使用工厂函数创建注册表
let mut registry = ManifestRegistry::new();
registry.register_factory("node", || create_node_provider());
registry.register_factory("go", || create_go_provider());

// 从工厂构建 ProviderRegistry
let provider_registry = registry.build_registry_from_factories()?;

// 不加载 provider 即可查询元数据
if let Some(metadata) = registry.get_runtime_metadata("npm") {
    println!("Provider: {}", metadata.provider_name);
    println!("生态系统: {:?}", metadata.ecosystem);
}
```

优势：
- **延迟加载**：Provider 仅在需要时创建
- **元数据访问**：无需加载 provider 即可查询运行时信息
- **可扩展性**：通过清单文件添加新 provider

### vx-resolver

版本解析和执行，带可观测性支持。

```
vx-resolver/
├── src/
│   ├── lib.rs
│   ├── resolver.rs        # 版本解析器
│   ├── executor.rs        # 命令执行器（带追踪 span）
│   ├── resolution_cache.rs # 带结构化日志的缓存
│   ├── deps.rs            # 依赖解析
│   └── spec.rs            # 运行时规格
```

#### 可观测性

执行器包含结构化追踪，用于调试和监控：

```rust
// 带结构化字段的执行 span
info_span!("vx_execute",
    runtime = %runtime_name,
    version = version.unwrap_or("latest"),
    args_count = args.len()
)

// 带结构化字段的缓存日志
debug!(
    runtime = %runtime,
    cache_hit = true,
    "Resolution cache hit"
);
```

### vx-cli

命令行界面。

```
vx-cli/
├── src/
│   ├── lib.rs
│   ├── cli.rs        # Clap 定义
│   ├── commands/     # 命令实现
│   ├── config.rs     # 配置（重新导出 vx-config）
│   ├── registry.rs   # Provider 注册
│   └── ui.rs         # 用户界面
```

### vx-config

配置管理，带安全特性。

```
vx-config/
├── src/
│   ├── lib.rs
│   ├── parser.rs      # TOML 解析
│   ├── inheritance.rs # 预设继承，带 SHA256 验证
│   └── types.rs       # 配置类型
```

#### 安全特性

远程预设验证：

```rust
// 带 SHA256 验证的 PresetSource
impl PresetSource {
    pub fn warn_if_unverified(&self);
    pub fn verify_content(&self, content: &str) -> Result<()>;
    pub fn has_hash_verification(&self) -> bool;
}
```

### vx-extension

扩展系统，带信任模型。

```
vx-extension/
├── src/
│   ├── lib.rs
│   ├── discovery.rs  # 扩展发现，带警告
│   └── loader.rs     # 扩展加载
```

#### 扩展信任模型

```rust
impl Extension {
    /// 获取扩展来源信息
    pub fn source_info(&self) -> String;
    
    /// 检查扩展是否来自可能不可信的来源
    pub fn is_potentially_untrusted(&self) -> bool;
    
    /// 为不可信扩展显示警告
    pub fn warn_if_untrusted(&self);
}
```

### vx-providers

工具 Provider（每个工具一个 crate）。

```
vx-providers/
├── node/
├── go/
├── rust/
├── uv/
├── deno/
└── ... (34+ providers)
```

每个 provider 包含 `provider.toml` 清单：

```toml
[provider]
name = "node"
description = "Node.js 运行时"

[[runtimes]]
name = "node"
executable = "node"
ecosystem = "nodejs"

[[runtimes]]
name = "npm"
executable = "npm"
ecosystem = "nodejs"
```

## 数据流

### 命令执行

```
用户输入
    │
    ▼
┌─────────┐
│  CLI    │ 解析参数
└────┬────┘
     │
     ▼
┌─────────┐
│Resolver │ 解析版本，检查依赖
└────┬────┘
     │
     ▼
┌─────────┐
│Provider │ 如需要则安装
└────┬────┘
     │
     ▼
┌─────────┐
│Executor │ 使用正确的 PATH 运行命令
└────┬────┘
     │
     ▼
  输出
```

### 版本解析

```
版本规格（如 "node@20"）
    │
    ▼
┌──────────────┐
│解析规格      │ 提取工具名和版本约束
└──────┬───────┘
       │
       ▼
┌──────────────┐
│检查存储      │ 版本是否已安装？
└──────┬───────┘
       │
       ▼（如未安装）
┌──────────────┐
│获取列表      │ 从 provider 获取可用版本
└──────┬───────┘
       │
       ▼
┌──────────────┐
│匹配版本      │ 找到最佳匹配版本
└──────┬───────┘
       │
       ▼
┌──────────────┐
│安装          │ 下载并安装
└──────────────┘
```

## 目录结构

```
~/.local/share/vx/
├── store/              # 已安装的工具版本
│   ├── node/
│   │   ├── 18.19.0/
│   │   └── 20.10.0/
│   ├── go/
│   │   └── 1.21.5/
│   └── uv/
│       └── 0.1.24/
├── envs/               # 命名环境
│   ├── default/        # 默认环境（符号链接）
│   └── my-project/     # 项目环境
├── cache/              # 下载的归档、版本列表
└── tmp/                # 临时文件
```

## 关键抽象

### Provider Trait

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn info(&self) -> ProviderInfo;
    async fn list_versions(&self) -> Result<Vec<String>>;
    async fn install(&self, version: &str) -> Result<()>;
    fn get_runtime(&self, version: &str) -> Result<RuntimeInfo>;
}
```

### RuntimeSpec

```rust
pub struct RuntimeSpec {
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub dependencies: Vec<RuntimeDependency>,
    pub executable: Option<String>,
    pub command_prefix: Vec<String>,
    pub ecosystem: Ecosystem,
}
```

### PathManager

```rust
impl PathManager {
    pub fn version_store_dir(&self, tool: &str, version: &str) -> PathBuf;
    pub fn env_dir(&self, name: &str) -> PathBuf;
    pub fn cache_dir(&self) -> PathBuf;
    pub fn list_store_versions(&self, tool: &str) -> Result<Vec<String>>;
}
```

### ConfigView

配置的扁平化视图，用于简单的键值操作：

```rust
pub struct ConfigView {
    pub tools: HashMap<String, String>,
    pub settings: HashMap<String, String>,
    pub env: HashMap<String, String>,
    pub scripts: HashMap<String, String>,
}

impl From<VxConfig> for ConfigView {
    fn from(config: VxConfig) -> Self { ... }
}
```

## 并发

- 使用 `tokio` 并行安装工具
- 异步版本获取
- 线程安全的 provider 注册表

## 错误处理

- 使用 `anyhow` 传播错误
- 上下文相关的错误消息
- 用户友好的错误显示

## 安全

详见 [安全](/zh/advanced/security)：
- 远程预设 SHA256 验证
- 扩展信任模型
- 结构化日志用于审计
