# RFC 0016: 环境管理系统 v2

## 概述

本 RFC 描述 vx 环境管理系统的三个核心改进：

1. **环境变量组装机制** - 参考 Rez 的设计，支持灵活的环境变量操作策略
2. **锁文件管理优化** - 明确 `vx sync` 和 `vx dev` 的锁文件处理流程
3. **上下文感知** - 基于项目配置的环境隔离和优先级管理

## 动机

### 当前问题

1. **环境变量组装不够灵活**
   - PATH 排序逻辑分散在多处
   - 不支持显式的 `append`、`prepend`、`replace` 策略
   - 环境变量冲突缺乏明确的解决机制

2. **锁文件流程不清晰**
   - `vx sync` 命令直接安装，不会自动更新锁文件
   - `vx dev` 不检查锁文件是否需要更新
   - 版本解析结果不持久化

3. **上下文优先级不明确**
   - 全局命令和项目命令的版本选择逻辑混淆
   - 缺乏显式的环境切换机制
   - 不同目录间的工具版本可能冲突

## 设计方案

### 1. 环境变量组装机制 (参考 Rez)

#### 1.1 环境变量操作类型

```rust
/// 环境变量操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvOperation {
    /// 设置变量（覆盖）
    Set(String),
    /// 追加到现有值末尾
    Append(String),
    /// 追加到现有值开头
    Prepend(String),
    /// 从现有值中移除
    Remove(String),
    /// 使用默认值（仅当未设置时）
    Default(String),
}

/// 环境变量配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub operation: EnvOperation,
    /// 分隔符（用于 PATH 类变量）
    #[serde(default = "default_separator")]
    pub separator: String,
}

fn default_separator() -> String {
    if cfg!(windows) { ";".to_string() } else { ":".to_string() }
}
```

#### 1.2 vx.toml 配置格式

```toml
# vx.toml

[tools]
python = "3.11"
node = "22"

# 简单环境变量（等同于 set）
[env]
NODE_ENV = "development"
DEBUG = "true"

# 高级环境变量操作
[env.advanced]
# PATH 前置（优先级最高）
path_prepend = [
    "${VX_STORE}/custom-tools/bin",
]
# PATH 追加（优先级最低）
path_append = [
    "/opt/legacy/bin",
]
# 自定义变量操作
[env.advanced.vars]
PYTHONPATH = { operation = "prepend", value = "${PROJECT_ROOT}/src" }
LD_LIBRARY_PATH = { operation = "append", value = "${VX_STORE}/libs" }
MY_CONFIG = { operation = "default", value = "/etc/default.conf" }
```

#### 1.3 环境变量组装器

```rust
/// 环境变量组装器
pub struct EnvAssembler {
    /// 基础环境（继承或隔离）
    base_env: HashMap<String, String>,
    /// 操作队列（按优先级排序）
    operations: Vec<(i32, EnvVar)>,
}

impl EnvAssembler {
    pub fn new() -> Self { /* ... */ }
    
    /// 从系统环境继承
    pub fn inherit_system(mut self) -> Self { /* ... */ }
    
    /// 使用隔离环境
    pub fn isolated(mut self, passenv: Vec<String>) -> Self { /* ... */ }
    
    /// 添加环境变量操作
    pub fn add_operation(mut self, priority: i32, var: EnvVar) -> Self {
        self.operations.push((priority, var));
        self
    }
    
    /// 添加工具的环境变量
    pub fn add_tool_env(mut self, tool: &ToolSpec, priority: i32) -> Self {
        // 工具 bin 目录 -> prepend to PATH
        self.add_operation(priority, EnvVar {
            name: "PATH".to_string(),
            operation: EnvOperation::Prepend(tool.bin_dir.to_string_lossy().to_string()),
            separator: default_separator(),
        })
    }
    
    /// 构建最终环境
    pub fn build(self) -> Result<HashMap<String, String>> {
        let mut env = self.base_env;
        
        // 按优先级排序操作
        let mut ops = self.operations;
        ops.sort_by_key(|(p, _)| -p); // 高优先级先执行
        
        for (_, var) in ops {
            match var.operation {
                EnvOperation::Set(value) => {
                    env.insert(var.name, value);
                }
                EnvOperation::Prepend(value) => {
                    let current = env.get(&var.name).cloned().unwrap_or_default();
                    if current.is_empty() {
                        env.insert(var.name, value);
                    } else {
                        env.insert(var.name, format!("{}{}{}", value, var.separator, current));
                    }
                }
                EnvOperation::Append(value) => {
                    let current = env.get(&var.name).cloned().unwrap_or_default();
                    if current.is_empty() {
                        env.insert(var.name, value);
                    } else {
                        env.insert(var.name, format!("{}{}{}", current, var.separator, value));
                    }
                }
                EnvOperation::Remove(pattern) => {
                    if let Some(current) = env.get(&var.name).cloned() {
                        let parts: Vec<&str> = current.split(&var.separator)
                            .filter(|p| !p.contains(&pattern))
                            .collect();
                        env.insert(var.name, parts.join(&var.separator));
                    }
                }
                EnvOperation::Default(value) => {
                    env.entry(var.name).or_insert(value);
                }
            }
        }
        
        Ok(env)
    }
}
```

#### 1.4 PATH 优先级保证

```rust
/// PATH 优先级（数值越大优先级越高）
pub mod PathPriority {
    /// 项目特定工具（最高优先级）
    pub const PROJECT_TOOLS: i32 = 1000;
    /// vx 管理的工具
    pub const VX_TOOLS: i32 = 900;
    /// vx shims
    pub const VX_SHIMS: i32 = 800;
    /// 用户自定义 prepend
    pub const USER_PREPEND: i32 = 700;
    /// 系统 PATH（继承）
    pub const SYSTEM: i32 = 500;
    /// 用户自定义 append
    pub const USER_APPEND: i32 = 300;
    /// 兼容性路径（最低优先级）
    pub const LEGACY: i32 = 100;
}

/// 确保 PATH 排序一致性
pub fn build_path_with_priority(
    tools: &[ToolSpec],
    config: &EnvConfig,
) -> String {
    let mut assembler = EnvAssembler::new().inherit_system();
    
    // 1. 添加项目工具（按配置顺序）
    for (i, tool) in tools.iter().enumerate() {
        let priority = PathPriority::PROJECT_TOOLS - i as i32;
        assembler = assembler.add_tool_env(tool, priority);
    }
    
    // 2. 添加 vx shims
    assembler = assembler.add_operation(PathPriority::VX_SHIMS, EnvVar {
        name: "PATH".to_string(),
        operation: EnvOperation::Prepend(vx_paths::shims_dir().to_string_lossy().to_string()),
        separator: default_separator(),
    });
    
    // 3. 用户自定义
    for path in &config.path_prepend {
        assembler = assembler.add_operation(PathPriority::USER_PREPEND, EnvVar {
            name: "PATH".to_string(),
            operation: EnvOperation::Prepend(path.clone()),
            separator: default_separator(),
        });
    }
    
    for path in &config.path_append {
        assembler = assembler.add_operation(PathPriority::USER_APPEND, EnvVar {
            name: "PATH".to_string(),
            operation: EnvOperation::Append(path.clone()),
            separator: default_separator(),
        });
    }
    
    assembler.build().unwrap().get("PATH").cloned().unwrap_or_default()
}
```

### 2. 锁文件管理优化

#### 2.1 改进后的流程

```
┌─────────────────────────────────────────────────────────────┐
│                      vx sync 流程                            │
├─────────────────────────────────────────────────────────────┤
│  1. 读取 vx.toml                                             │
│  2. 检查 vx.lock 是否存在且一致                               │
│     ├─ 不存在 → 自动运行 vx lock 生成                         │
│     └─ 存在但不一致 → 提示用户运行 vx lock --update           │
│  3. 使用 vx.lock 中的精确版本                                 │
│  4. 下载和安装工具                                            │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                      vx dev 流程                             │
├─────────────────────────────────────────────────────────────┤
│  1. 检查 vx.lock 状态                                        │
│     ├─ 不存在 → 自动运行 vx lock                             │
│     └─ 存在但不一致 → 提示是否更新                            │
│  2. 检查工具是否已安装                                        │
│     └─ 未安装 → 自动运行 vx sync                             │
│  3. 基于 vx.lock 构建环境                                    │
│  4. 启动开发环境                                              │
└─────────────────────────────────────────────────────────────┘
```

#### 2.2 sync 命令改进

```rust
/// vx sync 命令改进
pub async fn handle_sync(
    registry: &ProviderRegistry,
    options: SyncOptions,
) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;
    let project_root = config_path.parent().unwrap_or(&current_dir);
    let lock_path = project_root.join(LOCK_FILE_NAME);
    
    // Step 1: 检查锁文件状态
    let lockfile = match check_lockfile_status(&lock_path, &config)? {
        LockStatus::UpToDate(lf) => {
            if options.verbose {
                UI::info(&format!("Using versions from {}", LOCK_FILE_NAME));
            }
            lf
        }
        LockStatus::NeedsUpdate(inconsistencies) => {
            if options.auto_lock {
                UI::info("Lock file out of date, updating...");
                handle_lock_internal(registry, &config, &lock_path, options.verbose).await?
            } else {
                UI::warn("Lock file inconsistencies detected:");
                for inc in &inconsistencies {
                    UI::detail(&format!("  - {}", inc));
                }
                UI::hint("Run 'vx lock' to update the lock file, or use 'vx sync --auto-lock'");
                return Ok(());
            }
        }
        LockStatus::NotFound => {
            UI::info("No lock file found, generating...");
            handle_lock_internal(registry, &config, &lock_path, options.verbose).await?
        }
    };
    
    // Step 2: 使用锁文件版本安装
    let effective_tools = resolve_from_lockfile(&lockfile);
    install_tools(registry, &effective_tools, options).await
}

/// 锁文件状态
enum LockStatus {
    UpToDate(LockFile),
    NeedsUpdate(Vec<LockFileInconsistency>),
    NotFound,
}

fn check_lockfile_status(lock_path: &Path, config: &VxConfig) -> Result<LockStatus> {
    if !lock_path.exists() {
        return Ok(LockStatus::NotFound);
    }
    
    let lockfile = LockFile::load(lock_path)?;
    let config_tools: HashMap<String, String> = config.tools
        .iter()
        .map(|(k, v)| (k.clone(), get_version_string(v)))
        .collect();
    
    let inconsistencies = lockfile.check_consistency(&config_tools);
    
    if inconsistencies.is_empty() {
        Ok(LockStatus::UpToDate(lockfile))
    } else {
        Ok(LockStatus::NeedsUpdate(inconsistencies))
    }
}
```

#### 2.3 dev 命令改进

```rust
/// vx dev 命令改进
pub async fn handle_dev(
    registry: &ProviderRegistry,
    options: DevOptions,
) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config_full(&config_path)?;
    let project_root = config_path.parent().unwrap_or(&current_dir);
    let lock_path = project_root.join(LOCK_FILE_NAME);
    
    // Step 1: 确保锁文件存在且一致
    let lockfile = ensure_lockfile_ready(registry, &config, &lock_path, options.verbose).await?;
    
    // Step 2: 确保工具已安装
    if !options.skip_sync {
        ensure_tools_installed(registry, &lockfile, options.verbose).await?;
    }
    
    // Step 3: 构建环境并启动
    let env_vars = build_dev_environment(&config, &lockfile)?;
    
    if options.export {
        // 导出模式
        output_env_export(&env_vars, options.format)?;
    } else {
        // 交互式 shell 模式
        spawn_dev_shell(&env_vars, &config)?;
    }
    
    Ok(())
}

async fn ensure_lockfile_ready(
    registry: &ProviderRegistry,
    config: &VxConfig,
    lock_path: &Path,
    verbose: bool,
) -> Result<LockFile> {
    match check_lockfile_status(lock_path, config)? {
        LockStatus::UpToDate(lf) => Ok(lf),
        LockStatus::NeedsUpdate(inconsistencies) => {
            UI::warn("Lock file out of date:");
            for inc in &inconsistencies {
                UI::detail(&format!("  - {}", inc));
            }
            
            if UI::confirm("Update lock file and continue?")? {
                handle_lock_internal(registry, config, lock_path, verbose).await
            } else {
                Err(anyhow::anyhow!("Lock file update required"))
            }
        }
        LockStatus::NotFound => {
            UI::info("Generating lock file...");
            handle_lock_internal(registry, config, lock_path, verbose).await
        }
    }
}
```

### 3. 上下文感知

#### 3.1 环境上下文检测

```rust
/// 环境上下文
#[derive(Debug, Clone)]
pub enum EnvContext {
    /// 全局环境（无项目配置）
    Global,
    /// 项目环境
    Project {
        root: PathBuf,
        config: VxConfig,
        lockfile: Option<LockFile>,
    },
}

impl EnvContext {
    /// 从当前目录检测上下文
    pub fn detect() -> Result<Self> {
        let current_dir = env::current_dir()?;
        
        match find_vx_config(&current_dir) {
            Ok(config_path) => {
                let project_root = config_path.parent()
                    .ok_or_else(|| anyhow::anyhow!("Invalid config path"))?
                    .to_path_buf();
                let config = parse_vx_config(&config_path)?;
                let lock_path = project_root.join(LOCK_FILE_NAME);
                let lockfile = if lock_path.exists() {
                    LockFile::load(&lock_path).ok()
                } else {
                    None
                };
                
                Ok(EnvContext::Project {
                    root: project_root,
                    config,
                    lockfile,
                })
            }
            Err(_) => Ok(EnvContext::Global),
        }
    }
    
    /// 获取工具版本
    pub fn get_tool_version(&self, tool: &str) -> Option<String> {
        match self {
            EnvContext::Global => None, // 使用最新或默认版本
            EnvContext::Project { lockfile, config, .. } => {
                // 优先使用锁文件版本
                if let Some(lf) = lockfile {
                    if let Some(locked) = lf.get_tool(tool) {
                        return Some(locked.version.clone());
                    }
                }
                // 其次使用配置版本
                config.tools.get(tool).map(|v| get_version_string(v))
            }
        }
    }
}
```

#### 3.2 执行器上下文感知

```rust
impl Executor {
    /// 执行命令（上下文感知）
    pub async fn execute_with_context(
        &self,
        command: &str,
        args: &[String],
    ) -> Result<ExitStatus> {
        let context = EnvContext::detect()?;
        
        // 获取工具版本
        let version = match &context {
            EnvContext::Project { .. } => {
                // 项目环境：使用项目配置的版本
                context.get_tool_version(command)
            }
            EnvContext::Global => {
                // 全局环境：使用最新安装的版本或安装最新版
                None
            }
        };
        
        // 显示上下文信息（如果启用）
        if self.config.show_context {
            match &context {
                EnvContext::Project { root, .. } => {
                    tracing::debug!(
                        "Using project context: {} (version: {})",
                        root.display(),
                        version.as_deref().unwrap_or("latest")
                    );
                }
                EnvContext::Global => {
                    tracing::debug!("Using global context");
                }
            }
        }
        
        // 执行命令
        self.execute_internal(command, args, version.as_deref()).await
    }
}
```

#### 3.3 上下文切换命令

```bash
# 显示当前上下文
vx context
# Output:
# Project: /path/to/project
# Config: vx.toml
# Lock: vx.lock (up-to-date)
# Tools:
#   python = 3.11.13 (from lock)
#   node = 22.12.0 (from lock)

# 临时使用全局上下文
vx --global python --version
# 或
VX_CONTEXT=global vx python --version

# 临时使用特定版本（覆盖上下文）
vx python@3.14 --version
```

## 实现计划

### Phase 1: 环境变量组装改进 (v0.8.1)

1. [ ] 实现 `EnvOperation` 枚举和 `EnvVar` 结构
2. [ ] 实现 `EnvAssembler` 组装器
3. [ ] 更新 `vx.toml` 解析支持高级环境变量配置
4. [ ] 更新 `ToolEnvironment` 使用新的组装器
5. [ ] 添加单元测试验证 PATH 排序一致性

### Phase 2: 锁文件管理优化 (v0.8.2)

1. [ ] 改进 `vx sync` 命令，添加自动锁文件生成
2. [ ] 添加 `--auto-lock` 选项
3. [ ] 改进 `vx dev` 命令，添加锁文件检查
4. [ ] 实现 `ensure_lockfile_ready` 函数
5. [ ] 添加 `vx lock --check` 的 CI 集成支持

### Phase 3: 上下文感知 (v0.8.3)

1. [ ] 实现 `EnvContext` 类型
2. [ ] 更新 `Executor` 支持上下文感知
3. [ ] 实现 `vx context` 命令
4. [ ] 添加 `--global` 和 `VX_CONTEXT` 支持
5. [ ] 更新文档

## 向后兼容性

1. **环境变量配置**：现有的简单 `[env]` 配置继续支持
2. **锁文件**：现有的 `vx.lock` 格式完全兼容
3. **命令行**：所有现有命令保持不变

## 参考

- [Rez Environment Management](https://github.com/nerdvegas/rez)
- [Python venv activate scripts](https://docs.python.org/3/library/venv.html)
- [Conda Environment](https://docs.conda.io/projects/conda/en/latest/user-guide/concepts/environments.html)
- [nix-shell](https://nixos.org/manual/nix/stable/command-ref/nix-shell.html)
