# RFC 0031: 统一结构化输出 — `--json` 全局支持与 TOON 格式展望

> **状态**: Draft
> **作者**: VX Team
> **创建日期**: 2026-02-11
> **目标版本**: v0.5.0
> **关联**: RFC-0009 (统一控制台输出系统), RFC-0015 (系统工具发现)

## 摘要

本 RFC 提议为 vx CLI 的所有命令统一添加 `--json` 全局输出选项，并为未来的 TOON (Token-Oriented Object Notation) 格式支持做好架构准备。目标是让 vx 的输出既对人类友好（默认彩色文本），又能被脚本/CI 和 AI Agent 高效消费。

## 动机

### 现状问题

1. **输出格式不一致**：仅 6 个命令支持 `--json`（`info`, `metrics`, `test`, `analyze`, `global list`, `global info`），其余 10+ 命令无结构化输出
2. **存在两个 `OutputFormat` 枚举**：`cli.rs` 中的 `Table/Json/Yaml` 和 `global/args.rs` 中的 `Table/Json/Plain`，互不相关
3. **`search` 命令的 `--format` 参数是死代码**：接受参数但完全忽略
4. **`vx-console` 的 JSON 管道未接通**：`OutputMode::Json` 和 `JsonOutput` 结构体已定义但未被 `Shell` 使用，`ConsoleBuilder` 接受 `output_mode` 但 `build()` 时丢弃
5. **AI Agent 无法可靠解析输出**：通过 Skills 告知 AI 用 `vx list`，但纯文本 + emoji 输出难以程序化解析

### AI 场景的特殊需求

vx 的 AI 集成策略是 **Skills-first**（参见 `vx ai setup`），AI Agent 通过终端直接执行 vx 命令。这意味着：

- AI **直接执行** `vx list --json` 比通过 MCP 中间层调用更高效（零额外 token 开销）
- 结构化输出让 AI 能可靠解析结果，而不是猜测文本格式
- TOON 格式可进一步将 token 消耗降低 ~40%，对大数据量输出（版本列表、搜索结果）尤为显著

### 为什么不用 MCP

| 对比项 | MCP Server | CLI `--json` + Skills |
|--------|-----------|----------------------|
| Token 消耗 | 每次调用：schema 描述 + JSON-RPC 序列化 + 结果解析 | 一次性 Skills 加载，后续零开销 |
| 适合场景 | 非 CLI 的 API/数据库/浏览器 | CLI 工具 — 命令本身就是接口 |
| 部署要求 | 需要运行 MCP Server 进程 | 无需额外进程 |
| 覆盖面 | 需要每个 AI Agent 配置 | Skills 覆盖 40+ AI Agent |

vx 是 CLI-native 的工具，`--json` + Skills 是最自然的 AI 集成方式。

---

## 设计

### 分层输出架构

```
┌─────────────────────────────────────────────────────┐
│                  CLI 命令层                           │
│         各命令返回 impl CommandOutput                 │
├─────────────────────────────────────────────────────┤
│              输出渲染层 (OutputRenderer)              │
│   ┌──────────┬──────────┬──────────┬──────────┐     │
│   │   Text   │   JSON   │   TOON   │    CI    │     │
│   │  (默认)   │  (脚本)   │  (AI)   │  (CI/CD) │     │
│   └──────────┴──────────┴──────────┴──────────┘     │
├─────────────────────────────────────────────────────┤
│              vx-console (Shell)                      │
│        stdout (数据) / stderr (进度/日志)             │
└─────────────────────────────────────────────────────┘
```

### 核心原则

1. **数据写 stdout，日志写 stderr** — JSON/TOON 输出到 stdout，进度条/提示信息到 stderr
2. **全局 flag，命令无感** — 命令只需返回结构化数据，渲染格式由全局参数决定
3. **JSON 模式静默进度** — `--json` 时自动抑制进度条和 emoji 装饰
4. **向后兼容** — 默认 text 模式输出不变

### 全局参数设计

```rust
/// 全局 CLI 参数
#[derive(Parser)]
pub struct Cli {
    /// 输出格式
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// JSON 输出快捷方式 (等同于 --format json)
    #[arg(long, global = true)]
    pub json: bool,

    // ... 其他现有参数
}

/// 统一输出格式枚举（替换现有的两个不同枚举）
#[derive(Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    /// 人类可读的彩色文本输出（默认）
    #[default]
    Text,
    /// JSON 结构化输出（用于脚本/CI/AI 解析）
    Json,
    /// TOON 格式输出（用于 LLM prompt，节省 token）
    Toon,
}
```

### CommandOutput trait

```rust
use serde::Serialize;

/// 所有命令的结构化输出 trait
///
/// 命令实现此 trait 后，输出格式由全局参数自动决定。
/// 命令只需关注"返回什么数据"，不需关注"怎么展示"。
pub trait CommandOutput: Serialize {
    /// 人类可读的文本渲染
    fn render_text(&self, shell: &mut Shell) -> Result<()>;
}

/// 输出渲染器
pub struct OutputRenderer {
    format: OutputFormat,
}

impl OutputRenderer {
    pub fn render<T: CommandOutput>(&self, output: &T, shell: &mut Shell) -> Result<()> {
        match self.format {
            OutputFormat::Text => output.render_text(shell),
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(output)?;
                println!("{json}");
                Ok(())
            }
            OutputFormat::Toon => {
                // Phase 2: TOON 格式支持
                let toon = toon::to_string(output)?;
                println!("{toon}");
                Ok(())
            }
        }
    }
}
```

### 各命令输出结构体

#### `vx list`

```rust
#[derive(Serialize)]
pub struct ListOutput {
    pub runtimes: Vec<RuntimeEntry>,
}

#[derive(Serialize)]
pub struct RuntimeEntry {
    pub name: String,
    pub version: String,
    pub active: bool,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecosystem: Option<String>,
}

impl CommandOutput for ListOutput {
    fn render_text(&self, shell: &mut Shell) -> Result<()> {
        shell.header("Installed Runtimes")?;
        for rt in &self.runtimes {
            let status = if rt.active { "✓ active" } else { "" };
            shell.item(&format!("{:<12} {:<12} {}", rt.name, rt.version, status))?;
        }
        Ok(())
    }
}
```

**Text 输出**:
```
📦 Installed Runtimes

  node         20.0.0       ✓ active
  go           1.22.0       ✓ active
  uv           0.5.14       ✓ active
```

**JSON 输出** (`--json`):
```json
{
  "runtimes": [
    { "name": "node", "version": "20.0.0", "active": true, "path": "~/.vx/store/node/20.0.0" },
    { "name": "go", "version": "1.22.0", "active": true, "path": "~/.vx/store/go/1.22.0" },
    { "name": "uv", "version": "0.5.14", "active": true, "path": "~/.vx/store/uv/0.5.14" }
  ]
}
```

**TOON 输出** (`--format toon`):
```
runtimes[3]{name,version,active,path}:
  node,20.0.0,true,~/.vx/store/node/20.0.0
  go,1.22.0,true,~/.vx/store/go/1.22.0
  uv,0.5.14,true,~/.vx/store/uv/0.5.14
```

> Token 对比：JSON ~120 tokens → TOON ~50 tokens（**节省 58%**）

#### `vx versions <runtime>`

```rust
#[derive(Serialize)]
pub struct VersionsOutput {
    pub runtime: String,
    pub versions: Vec<VersionEntry>,
}

#[derive(Serialize)]
pub struct VersionEntry {
    pub version: String,
    pub installed: bool,
    pub lts: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lts_name: Option<String>,
    pub date: String,
}
```

TOON 在版本列表场景（通常 50-200 条）下优势更明显：

```
# 50 个版本的 JSON: ~2000 tokens
# 50 个版本的 TOON:  ~800 tokens (节省 60%)
versions[50]{version,installed,lts,date}:
  22.0.0,false,false,2026-02-01
  20.18.0,true,true,2025-12-15
  20.17.0,false,true,2025-11-20
  ...
```

#### `vx which <runtime>`

```rust
#[derive(Serialize)]
pub struct WhichOutput {
    pub runtime: String,
    pub version: String,
    pub path: String,
    pub source: String, // "vx", "system", "project"
}
```

#### `vx check`

```rust
#[derive(Serialize)]
pub struct CheckOutput {
    pub project_file: Option<String>,
    pub requirements: Vec<RequirementStatus>,
    pub all_satisfied: bool,
}

#[derive(Serialize)]
pub struct RequirementStatus {
    pub runtime: String,
    pub required: String,
    pub installed: Option<String>,
    pub satisfied: bool,
}
```

#### `vx install`

```rust
#[derive(Serialize)]
pub struct InstallOutput {
    pub runtime: String,
    pub version: String,
    pub path: String,
    pub already_installed: bool,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependencies_installed: Vec<DependencyInstalled>,
}

#[derive(Serialize)]
pub struct DependencyInstalled {
    pub runtime: String,
    pub version: String,
}
```

#### `vx search`

```rust
#[derive(Serialize)]
pub struct SearchOutput {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub name: String,
    pub description: String,
    pub provider: String,
    pub installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
}
```

#### `vx analyze`（已有，规范化）

```rust
#[derive(Serialize)]
pub struct AnalyzeOutput {
    pub languages: Vec<LanguageInfo>,
    pub dependencies: Vec<DependencyInfo>,
    pub scripts: Vec<ScriptInfo>,
    pub required_tools: Vec<RequiredToolInfo>,
}
```

---

## vx-console 集成

### 需要修改的现有代码

#### 1. Shell 添加 OutputMode 感知

```rust
// crates/vx-console/src/shell.rs
pub struct Shell {
    output: ShellOut,
    verbosity: Verbosity,
    theme: Theme,
    needs_clear: bool,
    output_mode: OutputMode, // 新增
    progress_manager: Option<ProgressManager>,
}

impl Shell {
    /// 在 JSON 模式下，info/warn/success 等写入 stderr（不污染 stdout 的 JSON）
    pub fn info(&mut self, message: &str) {
        if self.output_mode == OutputMode::Json {
            // JSON 模式下状态信息写 stderr
            eprintln!("{}", JsonOutput::info(message).to_json());
            return;
        }
        // 原有 text 逻辑...
    }
}
```

#### 2. ConsoleBuilder 正确传递 output_mode

```rust
// crates/vx-console/src/lib.rs
impl ConsoleBuilder {
    pub fn build(self) -> Console {
        let output_mode = self.output_mode.unwrap_or_default();
        let shell = ShellBuilder::new()
            .output_mode(output_mode) // 传递下去
            .build();
        Console { shell, /* ... */ }
    }
}
```

#### 3. 进度条在 JSON 模式下静默

```rust
impl Shell {
    pub fn create_progress(&self, msg: &str) -> Option<ProgressSpinner> {
        if self.output_mode == OutputMode::Json || self.verbosity == Verbosity::Quiet {
            return None; // JSON 模式不显示进度条
        }
        // ...
    }
}
```

### 清理工作

- **删除** `cli.rs` 中的 `OutputFormat`（Table/Json/Yaml）枚举
- **删除** `global/args.rs` 中的 `OutputFormat`（Table/Json/Plain）枚举
- **统一使用** 全局 `OutputFormat`（Text/Json/Toon）
- **修复** `search` 命令的死代码

---

## TOON 格式支持

### 什么是 TOON

[TOON](https://github.com/toon-format/toon)（Token-Oriented Object Notation）是专为 LLM prompt 设计的数据格式，核心特点：

- **JSON 的无损编码层**：数据模型完全兼容 JSON，可双向无损转换
- **对统一数组优化**：将结构相同的对象数组折叠为 CSV 表格形式，消除重复键名
- **显式模式声明**：`[N]{fields}` 语法帮助 LLM 理解数据结构
- **节省 ~40% token**：在对象数组场景下效果更佳

### 为什么适合 vx

vx 的大部分输出都是**统一对象数组**（TOON 的最佳场景）：

| 命令 | 输出类型 | TOON 节省 |
|------|----------|----------|
| `vx list` | Runtime 数组 | ~55% |
| `vx versions node` | 版本数组（通常 50-200 条） | ~60% |
| `vx search node` | 搜索结果数组 | ~50% |
| `vx check` | 需求状态数组 | ~45% |
| `vx analyze` | 依赖/脚本数组 | ~50% |
| `vx which node` | 单一对象 | ~10%（效果不明显） |

### 实现策略

#### Phase 1（本 RFC）：不实现 TOON

- 在 `OutputFormat` 枚举中预留 `Toon` variant
- 选择 `--format toon` 时报错：`TOON format is not yet supported. Use --json instead.`
- 为所有命令实现 `CommandOutput` trait（`Serialize` + `render_text()`）

#### Phase 2（未来 RFC）：接入 TOON SDK

TOON 目前仅有 TypeScript SDK，Rust SDK 尚不存在。两种方案：

**方案 A：等待 Rust SDK**
- TOON 项目可能会发布 `toon-rs` crate
- 直接依赖即可

**方案 B：自实现 TOON 序列化**
- TOON 规范简洁，核心逻辑不复杂
- 通过 `serde` 的 `Serializer` trait 实现 `ToonSerializer`
- 检测统一数组 → 表格化输出，其余 → 缩进格式

```rust
// 未来实现
pub struct ToonSerializer;

impl serde::Serializer for ToonSerializer {
    // 检测 Vec<T> 中的 T 是否结构统一
    // 统一 → 表格格式 (name[N]{fields}: ...)
    // 不统一 → 缩进格式
}
```

#### Phase 3：Skills 集成

在 vx 的 Skills 文档中指导 AI 使用 TOON：

```markdown
## 输出解析

当需要解析 vx 命令输出时：
- 优先使用 `--format toon`（如果可用，节省 token）
- 回退到 `--json`（通用兼容）

示例：
```bash
vx list --format toon    # AI 友好，省 token
vx list --json           # 通用结构化输出
```

---

## 完整命令覆盖清单

### 已有 JSON 支持（需迁移到统一架构）

| 命令 | 当前实现 | 迁移工作 |
|------|---------|---------|
| `vx info` | `--json` flag + `Capabilities` struct | 实现 `CommandOutput`，接入全局 `--json` |
| `vx metrics` | `--json` flag + `MetricsSummary` struct | 同上 |
| `vx test` | `--json` flag + `CITestSummary` struct | 同上 |
| `vx analyze` | `--json` flag + `AnalysisResult` struct | 同上 |
| `vx global list` | `--format` flag + 独立枚举 | 删除独立枚举，接入全局 |
| `vx global info` | `--json` flag | 接入全局 |

### 需要新增 JSON 支持

| 命令 | 输出结构体 | 优先级 |
|------|-----------|--------|
| `vx list` | `ListOutput { runtimes: Vec<RuntimeEntry> }` | P0 |
| `vx versions` | `VersionsOutput { runtime, versions: Vec<VersionEntry> }` | P0 |
| `vx which` | `WhichOutput { runtime, version, path, source }` | P0 |
| `vx check` | `CheckOutput { requirements: Vec<RequirementStatus> }` | P0 |
| `vx install` | `InstallOutput { runtime, version, path, duration_ms }` | P1 |
| `vx search` | `SearchOutput { query, results: Vec<SearchResult> }` | P1 |
| `vx sync` | `SyncOutput { installed: Vec<...>, skipped: Vec<...> }` | P1 |
| `vx lock` | `LockOutput { lockfile, entries: Vec<...> }` | P2 |
| `vx cache` | `CacheOutput { size, entries: Vec<...> }` | P2 |
| `vx env` | `EnvOutput { variables: HashMap<String, String> }` | P2 |
| `vx version` | `VersionOutput { version, git_hash, build_date }` | P2 |
| `vx dev info` | `DevInfoOutput { ... }` | P2 |

### 不需要 JSON 输出的命令

| 命令 | 原因 |
|------|------|
| `vx <runtime> [args]` | 透传执行，输出由目标 runtime 控制 |
| `vx ai setup` | 交互式命令 |
| `vx config edit` | 打开编辑器 |
| `vx completion` | Shell 补全脚本 |

---

## 实施计划

### Phase 1: 基础架构（1-2 周）

1. **定义 `CommandOutput` trait** 和 `OutputRenderer`
2. **统一 `OutputFormat` 枚举**，删除冗余定义
3. **添加全局 `--json` / `--format` 参数** 到 `Cli` struct
4. **接通 vx-console 的 JSON 管道**：Shell 添加 `output_mode`，进度条 JSON 模式静默
5. **预留 `Toon` variant**（选择时报友好错误）

### Phase 2: 命令迁移 — P0（1 周）

6. 为 `list`, `versions`, `which`, `check` 实现 `CommandOutput`
7. 迁移已有 JSON 命令（`info`, `metrics`, `test`, `analyze`）到统一架构
8. 修复 `search` 的死代码 `--format` 参数

### Phase 3: 命令迁移 — P1/P2（1-2 周）

9. 为 `install`, `search`, `sync` 实现 `CommandOutput`
10. 为 `lock`, `cache`, `env`, `version`, `dev info` 实现 `CommandOutput`
11. 清理 `global/args.rs` 中的独立 `OutputFormat`

### Phase 4: TOON 支持（未来）

12. 实现 `ToonSerializer`（或等待社区 `toon-rs` crate）
13. 接通 `OutputFormat::Toon` 渲染路径
14. 更新 Skills 文档，指导 AI 使用 `--format toon`

### Phase 5: Skills 集成

15. 在 SKILL.md 中增加结构化输出使用指导
16. 为常见 AI 场景提供示例（解析安装结果、检查依赖状态等）

---

## stdout / stderr 约定

```
Text 模式:
  stdout: 所有输出（彩色文本 + emoji + 进度条）
  stderr: 错误信息

JSON 模式:
  stdout: 纯 JSON 数据（一个完整的 JSON 对象）
  stderr: 进度信息（如果有）、警告、错误（也是 JSON Lines 格式）

TOON 模式（未来）:
  stdout: 纯 TOON 数据
  stderr: 同 JSON 模式
```

这遵循 Unix 管道哲学：`vx list --json | jq '.runtimes[] | select(.active)'`

---

## 环境变量支持

除了 `--json` / `--format` 命令行参数，支持环境变量配置：

```bash
# 全局设置 JSON 输出（适合 CI/脚本）
export VX_OUTPUT=json

# 全局设置 TOON 输出（适合 AI Agent 环境）
export VX_OUTPUT=toon

# 优先级: --format > --json > VX_OUTPUT > default(text)
```

已有的 `VX_OUTPUT_JSON=1` 环境变量保持向后兼容，等同于 `VX_OUTPUT=json`。

---

## 测试策略

### 单元测试

```rust
// crates/vx-cli/tests/output_tests.rs

#[test]
fn test_list_output_json() {
    let output = ListOutput {
        runtimes: vec![
            RuntimeEntry { name: "node".into(), version: "20.0.0".into(), active: true, .. },
        ],
    };
    let json: serde_json::Value = serde_json::from_str(
        &serde_json::to_string(&output).unwrap()
    ).unwrap();
    assert_eq!(json["runtimes"][0]["name"], "node");
}

#[rstest]
#[case("list", &["--json"], "runtimes")]
#[case("check", &["--json"], "all_satisfied")]
#[case("which", &["node", "--json"], "path")]
fn test_json_output_has_expected_field(
    #[case] cmd: &str,
    #[case] args: &[&str],
    #[case] field: &str,
) {
    let env = E2ETestEnv::new();
    let result = env.run_ok(&[cmd, ..args]);
    let json: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
    assert!(json.get(field).is_some(), "Missing field: {field}");
}
```

### 快照测试

```markdown
<!-- tests/cmd/json-output.md -->
# JSON Output

​```console
$ vx list --json
{
  "runtimes": []
}
​```

​```console
$ vx version --json
{
  "version": "...",
  ...
}
​```
```

### 契约测试

确保 JSON schema 不会意外变更：

```rust
#[test]
fn test_list_output_schema_stability() {
    let output = ListOutput::sample();
    let json = serde_json::to_value(&output).unwrap();

    // 必须有的字段
    assert!(json["runtimes"].is_array());
    for rt in json["runtimes"].as_array().unwrap() {
        assert!(rt["name"].is_string());
        assert!(rt["version"].is_string());
        assert!(rt["active"].is_boolean());
    }
}
```

---

## 对现有功能的影响

### 向后兼容

- 默认输出（text）完全不变
- 已有的 `--json` flag 在各命令上继续工作（但内部重定向到全局机制）
- `VX_OUTPUT_JSON=1` 环境变量继续工作

### Breaking Changes

- `global/args.rs` 的 `OutputFormat`（Table/Json/Plain）将被删除，迁移到全局 `--format`
- `cli.rs` 的 `OutputFormat`（Table/Json/Yaml）将被删除
- `search` 命令的 `--format` 参数语义变更（从死代码变为实际生效）

---

## 开放问题

1. **TOON Rust SDK**：目前仅有 TypeScript SDK。是否自实现 `ToonSerializer`，还是等待社区方案？
2. **JSON Lines vs 单 JSON**：对于流式输出（如 `vx install` 的进度），stderr 是否采用 JSON Lines 格式？
3. **`--json` 的退出码**：JSON 模式下命令失败时，是否仍通过退出码表示错误，同时在 JSON 中包含 error 字段？
4. **TOON 的 serde 集成**：TOON 规范中的表格化检测需要两遍扫描（先检测结构一致性，再序列化），这与 serde 的单遍 Serializer 模型有冲突，可能需要先序列化为 `serde_json::Value` 再转 TOON。

---

## 参考

- [TOON Specification](https://github.com/toon-format/toon) — Token-Oriented Object Notation
- [jq](https://stedolan.github.io/jq/) — JSON 命令行处理器
- [ripgrep `--json`](https://github.com/BurntSushi/ripgrep/blob/master/crates/printer/src/json.rs) — JSON Lines 输出参考
- [Cargo Shell](https://github.com/rust-lang/cargo/blob/master/src/cargo/core/shell.rs) — Rust CLI 输出架构参考
- RFC-0009: 统一控制台输出系统 — vx-console 现有设计
- RFC-0015: 系统工具发现 — MCP 工具定义（vx_run 等）
