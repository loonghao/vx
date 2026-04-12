# 系统架构

> 本文档是 vx 架构的**权威参考**。所有架构决策应记录在此或 [`docs/rfcs/`](../rfcs/)。

## 系统架构图

```
                        用户命令
                            │
                            ▼
                  ┌──────────────────┐
                  │     vx-cli       │  CLI 解析、命令路由
                  └────────┬─────────┘
                           │
                  ┌────────▼─────────┐
                  │   vx-resolver    │  解析运行时、检查依赖、
                  │   + Executor     │  自动安装、转发命令
                  └────────┬─────────┘
                           │
            ┌──────────────┼──────────────┐
            │              │              │
   ┌────────▼───────┐ ┌───▼──────┐ ┌────▼─────────┐
   │  vx-starlark   │ │vx-runtime│ │ vx-installer  │
   │ (DSL 引擎)     │ │(注册表)  │ │ (下载与安装)  │
   └────────┬───────┘ └──────────┘ └──────────────┘
            │
   ┌────────▼───────┐
   │ provider.star   │  129 个 Provider 定义
   │ 文件            │  (Starlark DSL)
   └────────────────┘
```

## Crate 依赖层级

### 第 0 层：基础（无内部依赖）

| Crate | 职责 |
|-------|------|
| `vx-core` | 核心 trait：`Runtime`、`Provider`、`PackageManager` |
| `vx-paths` | 跨平台路径管理（`~/.vx/` 目录结构） |
| `vx-cache` | 缓存层（HTTP 响应、版本列表） |
| `vx-versions` | 语义化版本解析与比较 |
| `vx-manifest` | Provider 清单解析（provider.star 元数据） |
| `vx-args` | 参数解析工具 |

### 第 1 层：基础设施（依赖第 0 层）

| Crate | 职责 | 关键依赖 |
|-------|------|---------|
| `vx-runtime-core` | Runtime trait 扩展 | vx-core |
| `vx-runtime-archive` | 归档解压（tar、zip、xz） | vx-core |
| `vx-runtime-http` | HTTP 客户端封装 | vx-core、vx-cache |
| `vx-config` | 分层配置（内置 → 用户 → 项目 → 环境变量） | vx-paths |
| `vx-env` | 环境变量管理 | vx-paths |
| `vx-console` | 统一输出、进度条、结构化日志 | — |
| `vx-metrics` | OpenTelemetry 追踪与指标 | — |

### 第 2 层：服务（依赖第 0-1 层）

| Crate | 职责 | 关键依赖 |
|-------|------|---------|
| `vx-runtime` | 运行时管理、`ManifestDrivenRuntime`、`ProviderRegistry` | vx-core、vx-runtime-*、vx-paths |
| `vx-starlark` | Starlark DSL 引擎，加载 `provider.star` | vx-runtime、vx-paths |
| `vx-installer` | 下载、校验、解压 | vx-runtime-archive、vx-runtime-http |
| `vx-version-fetcher` | 从 GitHub/npm/PyPI 获取可用版本 | vx-cache、vx-runtime-http |
| `vx-system-pm` | 系统包管理器集成（apt、brew、winget） | vx-core |
| `vx-ecosystem-pm` | 生态包管理器（npm、pip、cargo） | vx-core |
| `vx-shim` | Shim 二进制生成 | vx-paths |

### 第 3 层：编排（依赖第 0-2 层）

| Crate | 职责 | 关键依赖 |
|-------|------|---------|
| `vx-resolver` | 依赖解析、拓扑排序、命令执行 | vx-runtime、vx-installer |
| `vx-setup` | `vx setup` 命令——从 vx.toml 安装所有工具 | vx-resolver、vx-config |
| `vx-migration` | vx 版本间的迁移 | vx-paths、vx-config |
| `vx-extension` | 扩展系统 | vx-runtime、vx-args |
| `vx-project-analyzer` | 检测项目类型（React、Python、Rust 等） | vx-config |

### 第 4 层：应用（依赖所有层）

| Crate | 职责 |
|-------|------|
| `vx-cli` | CLI 入口、命令路由、用户交互 |

### Provider 层（独立，与第 2-3 层并列）

| 目录 | 职责 |
|------|------|
| `crates/vx-providers/*` | 129 个 Provider 定义，使用 `provider.star` Starlark DSL |
| `vx-bridge` | 通用命令桥接框架 |

**依赖规则**：每层只能依赖其**下方**的层，严禁向上依赖。

## 数据流：`vx node --version`

```
1. CLI 解析
   vx-cli 接收 ["node", "--version"]

2. 运行时查找
   vx-resolver → ProviderRegistry.find("node")
   → 找到：NodeProvider（通过 provider.star）

3. 依赖检查
   vx-resolver 检查 node 无未满足的依赖
   （npm/npx 与 node 捆绑，而非反向依赖）

4. 版本解析
   vx-starlark 调用 provider.star 中的 fetch_versions(ctx)
   → 返回可用版本列表
   vx-config 解析使用哪个版本（vx.toml、.vxrc、默认值）

5. 安装检查
   检查 ~/.vx/store/node/<version>/ 是否存在
   如不存在：download_url(ctx, version) → vx-installer → 解压

6. 环境配置
   vx-starlark 调用 environment(ctx, version)
   → 返回 [env_prepend("PATH", ".../bin")]

7. 命令转发
   执行：/path/to/node --version
   将退出码转发给调用方
```

## 存储布局

```
~/.vx/
├── store/           # 已安装的工具版本（内容寻址）
│   ├── node/
│   │   ├── 20.0.0/
│   │   └── 22.0.0/
│   └── go/
│       └── 1.23.0/
├── cache/           # 下载缓存、版本列表
│   ├── downloads/
│   └── versions/
├── bin/             # 全局 shims
├── config/          # 用户配置
└── metrics/         # 遥测数据（JSON 文件）
```

## Starlark Provider 系统

vx 使用**两阶段执行模型**（灵感来自 Buck2）：

1. **分析阶段（Starlark）**：`provider.star` 作为纯计算运行，返回描述符 dict，无 I/O。
2. **执行阶段（Rust）**：Rust 运行时解释描述符，执行实际的下载、安装和进程执行。

### Provider 模板

| 模板 | 适用场景 |
|------|---------|
| `github_rust_provider` | GitHub Releases 发布 Rust 二进制（最常见） |
| `github_go_provider` | GitHub Releases 发布 Go 二进制（goreleaser 风格） |
| `github_binary_provider` | 单一二进制下载（无归档） |
| `system_provider` | 仅系统包管理器安装 |

最简 Provider 示例：

```python
# crates/vx-providers/mytool/provider.star
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")

name        = "mytool"
description = "My awesome tool"
ecosystem   = "custom"

runtimes    = [runtime_def("mytool", aliases = ["mt"])]
permissions = github_permissions()

_p = github_rust_provider("owner", "repo",
    asset = "mytool-{vversion}-{triple}.{ext}")

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

## 关键设计决策

| 决策 | 理由 | RFC |
|------|------|-----|
| Starlark DSL 用于 Provider | 零编译、声明式、类型安全 | [RFC-0036](../rfcs/0036-starlark-provider-support.md) |
| provider.star 替代 TOML | 单一真相来源，更具表达力 | [RFC-0038](../rfcs/0038-provider-star-replaces-toml.md) |
| 清单驱动注册 | 新 Provider 无需 Rust 代码 | [RFC-0013](../rfcs/0013-manifest-driven-registration.md) |
| cargo-nextest 测试 | 并行测试速度提升 3 倍 | — |
| sccache CI 缓存 | 减少编译时间 40-60% | — |
| 纯 Rust TLS（rustls） | 无 OpenSSL 依赖，便于交叉编译 | — |

## 跨平台支持

| 平台 | 构建目标 | 备注 |
|------|---------|------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | 主要平台 |
| Linux x86_64（静态）| `x86_64-unknown-linux-musl` | Alpine/Docker |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | Raspberry Pi、ARM 服务器 |
| Linux ARM64（静态）| `aarch64-unknown-linux-musl` | Alpine ARM |
| macOS x86_64 | `x86_64-apple-darwin` | Intel Mac |
| macOS ARM64 | `aarch64-apple-darwin` | Apple Silicon |
| Windows x86_64 | `x86_64-pc-windows-msvc` | 主要 Windows 平台 |

## 并发与错误处理

- 使用 `tokio` 并行安装工具
- 异步版本获取（非阻塞 HTTP）
- 线程安全的 Provider 注册表（`Arc<RwLock<T>>`）
- 库代码使用 `thiserror` 定义具体错误类型
- 应用代码使用 `anyhow::Result` 简化错误传播
- 日志始终使用 `tracing` 宏，不使用 `println!`

## 退出码

| 代码 | 含义 |
|------|------|
| 0 | 成功 |
| 1 | 通用错误 |
| 2 | 工具未找到 |
| 3 | 安装失败 |
| 4 | 版本未找到 |
| 5 | 网络错误 |
| 6 | 权限错误 |
| 7 | 配置错误 |
