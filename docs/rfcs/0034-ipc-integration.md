# RFC 0034: IPC Integration with ipckit

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2026-02-16
> **目标版本**: v0.10.0 (Phase 2), v0.12.0 (Phase 3)

## 摘要

引入 [ipckit](https://github.com/loonghao/ipckit) crate 作为 vx 的进程间通信基础设施，替代当前基于环境变量的跨进程配置传递方案，并为未来的 daemon 模式、IDE 集成和跨语言扩展通信提供统一的 IPC 层。

## 背景

### Phase 1 已完成：RuntimeContext 方案

在 Phase 1 中，我们通过扩展 `RuntimeContext.install_options` 解决了 Executor 路径（进程内）的配置传递问题。具体改动：

1. `RuntimeContext` 新增 `install_options: HashMap<String, String>` 字段
2. `MsvcRuntime::parse_components()` 改为优先从 `ctx.install_options` 读取，env var 作为 fallback
3. `ProjectToolsConfig` 从 vx.toml 提取每个工具的详细配置（components, exclude_patterns, install_env）
4. `InstallationManager` 在调用 `runtime.install()` 前注入 install_options 到 cloned context

**Phase 1 解决了进程内路径的问题，但 `vx sync` 的跨进程路径仍依赖环境变量。**

### 当前跨进程通信的问题

```
vx sync
  └── spawn "vx install msvc@14.42"  ← 通过 env var 传递 VX_MSVC_COMPONENTS
      └── MsvcRuntime::install()     ← 通过 std::env::var() 读取
```

问题：
- **隐式依赖**：函数签名不体现对环境变量的依赖
- **不可测试**：环境变量是全局状态，难以 mock
- **单向通信**：只能传入配置，无法获取进度/状态反馈
- **无结构化**：只能传递字符串，不支持复杂数据结构

## 主流方案调研

在设计本方案之前，我们调研了以下主流实现：

### 1. interprocess crate (kotauskas/interprocess)

**架构**: 提供跨平台的 IPC 原语（Unix Domain Socket / Named Pipe）

**核心设计**:
```rust
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
let listener = LocalSocketListener::bind("example.sock")?;
for conn in listener.incoming() {
    // Handle client connections
}
```

**关键特性**:
- 跨平台统一 API（Unix socket / Windows named pipe）
- 支持 tokio async
- 只提供底层传输，不含消息协议

**依赖库**: 仅 OS 原生 API

### 2. ipckit crate (loonghao/ipckit)

**架构**: 全栈 IPC 框架，提供从传输层到应用层的完整抽象

**核心设计**:
```rust
use ipckit::{IpcChannel, Message};

// Server side
let (tx, rx) = IpcChannel::new("vx-install")?;
tx.send(Message::json(&InstallConfig { components: vec!["spectre"] }))?;

// Client side
let msg = rx.recv()?;
let config: InstallConfig = msg.parse_json()?;
```

**关键特性**:
- Channel 抽象（类似 Rust std channel 但跨进程）
- 共享内存支持（大数据传输）
- Socket 服务器模式（daemon）
- 事件流（实时推送）
- CLI Bridge（CLI ↔ daemon 通信）
- 任务管理器（分布式任务调度）
- 优雅关闭（进程生命周期管理）
- Python bindings（跨语言扩展）
- 指标采集（跨进程性能监控）

**依赖库**: 跨平台实现，支持 Windows/macOS/Linux

### 3. Axonweave Daemon (gRPC-based)

**架构**: 基于 gRPC 的微服务 IPC 代理

**关键特性**:
- 服务发现
- 语言无关的 RPC
- 适合大规模微服务

**评估**: 过于重量级，不适合 CLI 工具管理器场景

### 方案对比

| 特性 | interprocess | ipckit | gRPC (tonic) |
|------|-------------|--------|--------------|
| 跨平台 | ✓ | ✓ | ✓ |
| Channel 抽象 | ✗ | ✓ | ✗ (需要 proto) |
| 共享内存 | ✗ | ✓ | ✗ |
| Socket Server | 部分 | ✓ | ✓ |
| 事件流 | ✗ | ✓ | ✓ (streaming) |
| Python Bindings | ✗ | ✓ | ✓ (grpcio) |
| CLI Bridge | ✗ | ✓ | ✗ |
| 优雅关闭 | ✗ | ✓ | ✗ |
| 序列化开销 | 低 | 低 | 中 (protobuf) |
| 新增依赖 | 1 crate | 1 crate | 3+ crates |
| 学习成本 | 低 | 中 | 高 |

### 设计启示

基于以上调研，本 RFC 应采用：

1. **使用 ipckit** — 提供了我们需要的全部 IPC 原语，且与 vx 同一作者维护
2. **Channel 模式为主** — `vx sync` → `vx install` 的通信用 channel 最自然
3. **Socket Server 模式为远期** — daemon 场景用 socket server
4. **渐进式迁移** — 保持 env var 作为 fallback，逐步替换

## 动机

### 当前状态分析

vx 中存在以下跨进程通信场景：

| 场景 | 当前方案 | 问题 |
|------|---------|------|
| `vx sync` → `vx install` | env vars | 单向、隐式、不可测试 |
| `vx bridge` → `dotnet msbuild` | 命令转发 | 无进度回传 |
| Executor 路径 → runtime.install() | **Phase 1 已解决** (RuntimeContext) | ✓ |

### 需求分析

1. **结构化配置传递** — 替代 `VX_MSVC_COMPONENTS` 等 env vars
2. **双向通信** — 安装进度回传（downloading 45%, extracting...）
3. **未来 daemon** — 加速日常命令（缓存 manifest 解析结果）
4. **IDE 集成** — VSCode 插件 ↔ vx daemon 实时通信
5. **跨语言扩展** — Rust ↔ Python 扩展结构化通信

## 设计方案

### 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                        vx-ipc crate                          │
│  (封装 ipckit，提供 vx 特定的 IPC 协议和消息类型)            │
├──────────────┬──────────────┬───────────────┬───────────────┤
│ InstallChannel│ ProgressStream│ DaemonClient  │ ExtensionBridge│
│ (Phase 2)     │ (Phase 2)     │ (Phase 3)     │ (Phase 3)     │
└──────┬───────┴──────┬───────┴───────┬───────┴───────┬───────┘
       │              │               │               │
       ▼              ▼               ▼               ▼
┌──────────────────────────────────────────────────────────────┐
│                    ipckit (底层 IPC 框架)                      │
│  channel | pipe | shm | local_socket | event_stream | ...     │
└──────────────────────────────────────────────────────────────┘
```

### Crate 结构

```
crates/vx-ipc/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── messages.rs          # 消息类型定义
│   ├── install_channel.rs   # Phase 2: 安装配置 + 进度通信
│   ├── progress.rs          # Phase 2: 进度事件流
│   └── daemon/              # Phase 3: daemon 模式
│       ├── mod.rs
│       ├── server.rs        # daemon 服务端
│       ├── client.rs        # daemon 客户端 (CLI/IDE)
│       └── protocol.rs      # daemon 协议定义
```

### Phase 2: 替代 env vars 的 IPC Channel

#### 消息类型定义

```rust
// crates/vx-ipc/src/messages.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration message sent from parent to child process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// Tool name (e.g., "msvc")
    pub tool_name: String,
    /// Version to install
    pub version: String,
    /// Tool-specific options (replaces env vars like VX_MSVC_COMPONENTS)
    pub options: HashMap<String, String>,
    /// Download URL cache from lock file
    pub download_url: Option<String>,
}

/// Progress event sent from child to parent process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallProgress {
    /// Starting installation
    Started { tool: String, version: String },
    /// Downloading with progress
    Downloading { tool: String, percent: f32, bytes_downloaded: u64, total_bytes: u64 },
    /// Extracting archive
    Extracting { tool: String, percent: f32 },
    /// Running post-install hooks
    PostInstall { tool: String, message: String },
    /// Installation completed
    Completed { tool: String, version: String, install_path: String },
    /// Installation failed
    Failed { tool: String, error: String },
}
```

#### Install Channel

```rust
// crates/vx-ipc/src/install_channel.rs
use ipckit::IpcChannel;
use crate::messages::{InstallConfig, InstallProgress};

/// Channel name format: vx-install-{pid}-{tool}
fn channel_name(pid: u32, tool: &str) -> String {
    format!("vx-install-{}-{}", pid, tool)
}

/// Parent side: send config, receive progress
pub struct InstallChannelParent {
    config_tx: ipckit::Sender<InstallConfig>,
    progress_rx: ipckit::Receiver<InstallProgress>,
}

impl InstallChannelParent {
    /// Create a new channel and return the channel name for the child
    pub fn new(tool: &str) -> anyhow::Result<(Self, String)> {
        let name = channel_name(std::process::id(), tool);
        let (config_tx, progress_rx) = IpcChannel::pair(&name)?;
        Ok((Self { config_tx, progress_rx }, name))
    }

    /// Send install configuration to child
    pub fn send_config(&self, config: InstallConfig) -> anyhow::Result<()> {
        self.config_tx.send(&config)
    }

    /// Receive progress updates (non-blocking)
    pub fn try_recv_progress(&self) -> Option<InstallProgress> {
        self.progress_rx.try_recv().ok()
    }
}

/// Child side: receive config, send progress
pub struct InstallChannelChild {
    config_rx: ipckit::Receiver<InstallConfig>,
    progress_tx: ipckit::Sender<InstallProgress>,
}

impl InstallChannelChild {
    /// Connect to an existing channel by name
    pub fn connect(channel_name: &str) -> anyhow::Result<Self> {
        let (config_rx, progress_tx) = IpcChannel::connect(channel_name)?;
        Ok(Self { config_rx, progress_tx })
    }

    /// Receive install configuration from parent
    pub fn recv_config(&self) -> anyhow::Result<InstallConfig> {
        self.config_rx.recv()
    }

    /// Send progress update to parent
    pub fn send_progress(&self, progress: InstallProgress) -> anyhow::Result<()> {
        self.progress_tx.send(&progress)
    }
}
```

#### 改造 `vx sync` 的安装流程

**Before (当前)**:
```rust
// sync.rs - install_tool()
async fn install_tool(name: &str, version: &str, env_vars: Option<&InstallEnvVars>) -> (bool, Option<String>) {
    let mut cmd = Command::new(exe);
    cmd.args(["install", &format!("{}@{}", name, version)]);
    // 通过 env var 传递配置 (隐式!)
    if let Some(vars) = env_vars {
        for (key, value) in vars {
            cmd.env(key, value);
        }
    }
    cmd.output() // 阻塞等待, 无进度反馈
}
```

**After (Phase 2)**:
```rust
// sync.rs - install_tool() with IPC
async fn install_tool(name: &str, version: &str, config: InstallConfig) -> (bool, Option<String>) {
    // Create IPC channel
    let (parent, channel_name) = InstallChannelParent::new(name)?;

    // Spawn child with channel name (not env vars!)
    let mut cmd = Command::new(exe);
    cmd.args(["install", &format!("{}@{}", name, version)]);
    cmd.arg("--ipc-channel").arg(&channel_name);

    let child = cmd.spawn()?;

    // Send structured config through IPC
    parent.send_config(config)?;

    // Receive real-time progress
    while let Some(progress) = parent.try_recv_progress() {
        match progress {
            InstallProgress::Downloading { percent, .. } => {
                update_progress_bar(name, percent);
            }
            InstallProgress::Completed { .. } => break,
            InstallProgress::Failed { error, .. } => return (false, Some(error)),
            _ => {}
        }
    }

    let status = child.wait()?;
    (status.success(), None)
}
```

**Install command 端 (子进程)**:
```rust
// vx install msvc@14.42 --ipc-channel vx-install-1234-msvc
if let Some(channel_name) = args.ipc_channel {
    let child = InstallChannelChild::connect(&channel_name)?;
    let config = child.recv_config()?;

    // Use config.options instead of env vars
    ctx.set_install_options(config.options);

    // Install with progress reporting
    let result = runtime.install_with_progress(version, &ctx, |progress| {
        child.send_progress(progress).ok();
    }).await?;

    child.send_progress(InstallProgress::Completed { .. })?;
}
```

### Phase 3: Daemon 模式

#### Daemon 协议

```rust
// crates/vx-ipc/src/daemon/protocol.rs

/// Request from CLI/IDE to daemon
#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonRequest {
    /// Resolve a runtime (check version, find executable)
    Resolve { runtime: String, version: Option<String> },
    /// Get runtime environment variables
    GetEnvironment { runtime: String, version: String },
    /// List installed versions
    ListVersions { runtime: String },
    /// Subscribe to events
    Subscribe { event_types: Vec<EventType> },
    /// Health check
    Ping,
    /// Shutdown daemon
    Shutdown,
}

/// Response from daemon
#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonResponse {
    Resolved { executable: PathBuf, version: String, env: HashMap<String, String> },
    Environment { env: HashMap<String, String> },
    Versions { versions: Vec<String> },
    Subscribed { subscription_id: String },
    Pong { uptime_secs: u64, cached_runtimes: usize },
    Error { message: String },
}

/// Events pushed to subscribers
#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonEvent {
    RuntimeInstalled { runtime: String, version: String },
    RuntimeRemoved { runtime: String, version: String },
    ConfigChanged { path: PathBuf },
}
```

#### Daemon 使用场景

```
# Without daemon (current):
$ time vx node --version    # ~100ms (parse manifest, check version, build PATH)
v20.18.0

# With daemon:
$ vx daemon start           # Background process, caches everything
$ time vx node --version    # ~5ms (IPC query to daemon)
v20.18.0
```

#### IDE 集成

```
┌──────────────┐     local socket     ┌──────────────┐
│  VSCode Ext  │ ◄──────────────────► │  vx daemon   │
│  (TypeScript)│     DaemonRequest/   │  (Rust)      │
│              │     DaemonResponse   │              │
├──────────────┤                      ├──────────────┤
│ - Runtime    │                      │ - Manifest   │
│   selector   │                      │   cache      │
│ - Version    │                      │ - Version    │
│   picker     │                      │   cache      │
│ - Install    │                      │ - PATH       │
│   progress   │                      │   builder    │
└──────────────┘                      └──────────────┘
```

## 向后兼容性

### 兼容策略

1. **Env vars 作为 fallback**: `parse_components()` 已实现优先从 `ctx.install_options` 读取，env var 作为 fallback
2. **--ipc-channel 为可选**: `vx install` 在没有 `--ipc-channel` 参数时，继续从 env var 或 `ctx` 读取配置
3. **渐进迁移**: Phase 2 只改造 `vx sync` 路径，其他路径不变
4. **Feature flag**: `vx-ipc` crate 可通过 cargo feature 条件编译

### 迁移路径

```
Phase 1 (已完成): RuntimeContext.install_options (进程内)
    ↓
Phase 2: ipckit channel (vx sync 跨进程)
    ↓ (env vars 仍可用, 但标记为 deprecated)
Phase 3: ipckit daemon (全面替代)
    ↓ (移除 env var fallback)
```

## 实现计划

### Phase 2: IPC Channel 替代 env vars (v0.10.0)

- [ ] 创建 `crates/vx-ipc/` crate
- [ ] 定义 `InstallConfig` 和 `InstallProgress` 消息类型
- [ ] 实现 `InstallChannelParent` / `InstallChannelChild`
- [ ] 改造 `vx sync` 的 `install_tool()` 使用 IPC channel
- [ ] 改造 `vx install` 支持 `--ipc-channel` 参数
- [ ] 实现安装进度回传（下载 %、解压 %）
- [ ] `vx sync` 显示并行安装的实时进度
- [ ] 添加单元测试和集成测试
- [ ] 标记 env var 传递方式为 deprecated

### Phase 3: Daemon 模式 (v0.12.0)

- [ ] 实现 `vx daemon start/stop/status` 命令
- [ ] 实现 daemon 服务端（socket server）
- [ ] 实现 CLI 端 daemon 客户端
- [ ] 实现 Runtime 解析缓存（加速 `vx <tool>` 命令）
- [ ] 实现事件订阅/推送
- [ ] 实现 VSCode 扩展端 daemon 客户端
- [ ] 实现跨语言扩展通信（Python bindings）
- [ ] 添加 E2E 测试

## 替代方案

### 方案 A: 继续使用环境变量

**优点**: 零改动
**缺点**: 隐式依赖、不可测试、单向通信、无进度回传

### 方案 B: 使用 gRPC (tonic + prost)

**优点**: 成熟的 RPC 框架，强类型
**缺点**: 需要 .proto 文件、编译时代码生成、3+ 新依赖、对 CLI 工具过重

### 方案 C: 使用 interprocess crate

**优点**: 轻量，只提供传输层
**缺点**: 需要自己实现序列化、消息协议、进度流等高层抽象

### 方案 D: 使用 ipckit（本 RFC 选择）

**优点**: 全栈 IPC 方案、Channel 抽象、事件流、daemon 支持、Python bindings、同一作者维护
**缺点**: 相对较新的 crate

**结论**: ipckit 提供了最适合 vx 场景的抽象层级，且与项目同一作者维护，长期可控。

## 参考资料

### 主流项目源码
- [ipckit](https://github.com/loonghao/ipckit) — 本 RFC 的核心依赖
- [interprocess](https://github.com/kotauskas/interprocess) — 跨平台 IPC 原语参考

### 依赖库
- `ipckit` — 全栈 IPC 框架
- `serde` / `serde_json` — 消息序列化（已在 vx 中使用）

### 相关 RFC
- Phase 1 实现：`RuntimeContext.install_options`（代码已合并）

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-02-16 | Draft | 初始草案，Phase 1 已完成，描述 Phase 2/3 计划 |
