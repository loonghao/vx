# RFC 0035: AI 集成优化 — 打造 AI-First 的 CLI 工具框架

> **状态**: Draft
> **作者**: VX Team
> **创建日期**: 2026-02-19
> **目标版本**: v0.15.0
> **关联**: RFC-0031 (统一结构化输出), RFC-0034 (IPC 集成), RFC-0027 (隐式包执行), RFC-0016 (Unix CLI Tool Providers)

## 摘要

本 RFC 提出 vx 作为 **AI-First** 的 CLI 工具框架的全面优化方案。目标是让 AI Agent（如 Claude Code、Cursor、Copilot 等）能够高效、可靠地使用 vx 管理开发环境和执行命令，同时降低 token 消耗和提升命令执行速度。

## 动机

### 当前状态分析

vx 已具备良好的 AI 集成基础：

1. **Skills 系统** (`vx ai setup`)：支持 14+ AI Agent
2. **结构化输出** (`--json`)：部分命令支持
3. **隐式包执行** (`vx npm:package`)：简化命令语法
4. **自动安装**：零配置工具管理

### 存在的问题

| 问题 | 影响 | 相关 RFC |
|------|------|----------|
| `--json` 未覆盖所有命令 | AI 无法可靠解析输出 | RFC-0031 (Phase 2 待完成) |
| 无 TOON 格式支持 | Token 消耗高 (~40% 浪费) | RFC-0031 (Phase 4) |
| 无 daemon 模式 | 命令执行慢 (100ms vs 5ms) | RFC-0034 |
| Skills 文档分散 | AI 需多次加载上下文 | 本 RFC |
| 缺少 AI 常用工具 | 部分 CLI 工具未集成 | RFC-0030, RFC-0016 |
| 无 AI 会话状态持久化 | 每次命令需重新初始化 | 本 RFC |

### AI Agent 使用 vx 的典型场景

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    AI Agent 典型工作流                                   │
├─────────────────────────────────────────────────────────────────────────┤
│  1. 项目分析                                                            │
│     vx analyze --json          → 获取语言、依赖、脚本                   │
│     vx check --json            → 检查工具约束                           │
│                                                                         │
│  2. 环境准备                                                            │
│     vx sync                    → 安装所有工具                           │
│     vx which node --json       → 确认工具路径                           │
│                                                                         │
│  3. 命令执行                                                            │
│     vx npm test                → 运行测试                               │
│     vx cargo build             → 构建项目                               │
│     vx just lint               → 代码检查                               │
│                                                                         │
│  4. 结果解析                                                            │
│     vx list --json             → 查看已安装工具                         │
│     vx versions node --json    → 查询可用版本                           │
└─────────────────────────────────────────────────────────────────────────┘
```

## 设计方案

### 1. 完善 `--json` 输出覆盖

#### 1.1 命令优先级

**P0 - 必须实现**（AI 高频使用）：

| 命令 | 输出结构 | 场景 |
|------|----------|------|
| `vx list` | `ListOutput` | 查看已安装工具 |
| `vx versions <tool>` | `VersionsOutput` | 版本选择 |
| `vx which <tool>` | `WhichOutput` | 确认工具路径 |
| `vx check` | `CheckOutput` | 验证环境就绪 |
| `vx analyze` | `AnalyzeOutput` | 项目分析（已有） |

**P1 - 应该实现**：

| 命令 | 输出结构 | 场景 |
|------|----------|------|
| `vx install` | `InstallOutput` | 安装结果解析 |
| `vx search` | `SearchOutput` | 工具搜索 |
| `vx sync` | `SyncOutput` | 同步状态 |
| `vx env` | `EnvOutput` | 环境变量 |

**P2 - 可选实现**：

| 命令 | 输出结构 | 场景 |
|------|----------|------|
| `vx lock` | `LockOutput` | 锁文件信息 |
| `vx cache` | `CacheOutput` | 缓存状态 |
| `vx version` | `VersionOutput` | 版本信息 |

#### 1.2 输出结构定义

```rust
// crates/vx-cli/src/output/mod.rs

/// 所有命令输出的统一 trait
pub trait CommandOutput: Serialize {
    /// 人类可读的文本渲染
    fn render_text(&self, shell: &mut Shell) -> Result<()>;
    
    /// AI 优化的 TOON 格式渲染（可选）
    fn render_toon(&self) -> Result<String> {
        // 默认实现：TOON 序列化
        toon::to_string(self)
    }
}

/// vx list 输出
#[derive(Serialize)]
pub struct ListOutput {
    pub runtimes: Vec<RuntimeEntry>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct RuntimeEntry {
    pub name: String,
    pub version: String,
    pub active: bool,
    pub path: String,
    pub ecosystem: Option<String>,
    pub source: RuntimeSource,
}

#[derive(Serialize)]
pub enum RuntimeSource {
    VxManaged,
    System,
    Project,
}

/// vx versions 输出
#[derive(Serialize)]
pub struct VersionsOutput {
    pub runtime: String,
    pub versions: Vec<VersionEntry>,
    pub total: usize,
    pub latest: Option<String>,
    pub lts: Option<String>,
}

#[derive(Serialize)]
pub struct VersionEntry {
    pub version: String,
    pub installed: bool,
    pub lts: bool,
    pub lts_name: Option<String>,
    pub date: String,
    pub security: bool,
}

/// vx check 输出
#[derive(Serialize)]
pub struct CheckOutput {
    pub project_file: Option<String>,
    pub requirements: Vec<RequirementStatus>,
    pub all_satisfied: bool,
    pub missing_tools: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Serialize)]
pub struct RequirementStatus {
    pub runtime: String,
    pub required: String,
    pub installed: Option<String>,
    pub satisfied: bool,
    pub action: Option<String>,
}

/// vx sync 输出
#[derive(Serialize)]
pub struct SyncOutput {
    pub installed: Vec<InstallResult>,
    pub skipped: Vec<SkippedResult>,
    pub failed: Vec<FailedResult>,
    pub duration_ms: u64,
}

#[derive(Serialize)]
pub struct InstallResult {
    pub runtime: String,
    pub version: String,
    pub path: String,
    pub duration_ms: u64,
}

#[derive(Serialize)]
pub struct SkippedResult {
    pub runtime: String,
    pub reason: String,
}

#[derive(Serialize)]
pub struct FailedResult {
    pub runtime: String,
    pub version: String,
    pub error: String,
}
```

### 2. TOON 格式支持

#### 2.1 为什么需要 TOON

TOON (Token-Oriented Object Notation) 是专为 LLM prompt 设计的数据格式：

| 场景 | JSON tokens | TOON tokens | 节省 |
|------|-------------|-------------|------|
| 50 个版本列表 | ~2000 | ~800 | **60%** |
| 20 个已安装工具 | ~400 | ~180 | **55%** |
| 项目分析结果 | ~600 | ~300 | **50%** |

#### 2.2 使用方式

```bash
# JSON 格式（兼容性好）
vx list --json

# TOON 格式（AI 优化）
vx list --format toon

# 或通过环境变量
export VX_OUTPUT=toon
vx list
```

#### 2.3 TOON 输出示例

**JSON 输出** (`vx versions node --json`):
```json
{
  "runtime": "node",
  "versions": [
    {"version": "22.0.0", "installed": false, "lts": false, "date": "2024-10-01"},
    {"version": "20.18.0", "installed": true, "lts": true, "lts_name": "iron", "date": "2024-09-15"},
    {"version": "20.17.0", "installed": false, "lts": true, "lts_name": "iron", "date": "2024-08-20"}
  ],
  "total": 3,
  "latest": "22.0.0",
  "lts": "20.18.0"
}
```

**TOON 输出** (`vx versions node --format toon`):
```
versions[3]{version,installed,lts,lts_name,date}:
  22.0.0,false,false,,2024-10-01
  20.18.0,true,true,iron,2024-09-15
  20.17.0,false,true,iron,2024-08-20

meta{runtime,total,latest,lts}:
  node,3,22.0.0,20.18.0
```

### 3. AI Context 命令

新增 `vx ai context` 命令，为 AI Agent 提供项目上下文：

```bash
# 生成项目上下文（用于 AI prompt）
vx ai context

# 输出示例
```

```markdown
# Project Context

## Environment
- Runtime: node@22.0.0 (vx-managed)
- Runtime: uv@0.5.14 (vx-managed)
- Runtime: go@1.22.0 (vx-managed)

## Project Type
- Language: TypeScript, Go
- Framework: Next.js 14
- Package Manager: pnpm

## Available Scripts
- `vx run dev` - Start development server
- `vx run build` - Build for production
- `vx run test` - Run tests
- `vx run lint` - Run linter

## Tool Constraints
- Node.js >= 18 (required for Next.js 14)
- pnpm >= 8 (project uses pnpm-lock.yaml)

## Important Files
- vx.toml - Tool version configuration
- package.json - Node.js dependencies
- go.mod - Go dependencies
```

#### 3.1 输出格式

```bash
# 默认：Markdown 格式（适合 prompt）
vx ai context

# JSON 格式（程序化处理）
vx ai context --json

# 最小格式（仅关键信息）
vx ai context --minimal
```

#### 3.2 Context 结构

```rust
/// AI context 输出
#[derive(Serialize)]
pub struct AiContext {
    /// 项目基本信息
    pub project: ProjectInfo,
    /// 已安装的工具
    pub tools: Vec<ToolInfo>,
    /// 项目脚本
    pub scripts: Vec<ScriptInfo>,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// 工具约束
    pub constraints: Vec<ConstraintInfo>,
    /// 重要文件路径
    pub important_files: Vec<String>,
    /// 推荐命令
    pub recommended_commands: Vec<String>,
}

#[derive(Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub root: String,
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub package_managers: Vec<String>,
}

#[derive(Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub source: String,  // "vx", "system", "project"
    pub ecosystem: String,
}
```

### 4. 增强 Skills 系统

#### 4.1 当前 Skills 架构

```
~/.claude/skills/vx-usage/SKILL.md      # Claude Code
~/.cursor/skills/vx-usage/SKILL.md      # Cursor
~/.copilot/skills/vx-usage/SKILL.md     # Copilot
... (14+ agents)
```

#### 4.2 新增 Skills 模块

**vx-cli/src/skills/** 目录结构：

```
skills/
├── vx-usage/
│   └── SKILL.md              # 基础使用指南（已有）
├── vx-commands/
│   └── SKILL.md              # 命令参考
├── vx-project/
│   └── SKILL.md              # 项目管理指南
├── vx-troubleshooting/
│   └── SKILL.md              # 故障排除指南
└── vx-best-practices/
    └── SKILL.md              # 最佳实践
```

#### 4.3 Skills 内容优化

**vx-commands/SKILL.md**:
```markdown
---
name: vx-commands
description: |
  Complete command reference for vx CLI. Use this skill when you need
  to look up specific command syntax, flags, or output formats.
---

# vx Command Reference

## Structured Output Commands (AI-Optimized)

All commands support `--json` for structured output and `--format toon` for token-optimized output.

### Project Analysis

```bash
vx analyze --json           # Analyze project structure
vx check --json             # Verify tool constraints
```

**Output fields**:
- `languages[]` - Detected languages
- `dependencies[]` - Project dependencies
- `scripts[]` - Available scripts
- `required_tools[]` - Tools needed

### Tool Management

```bash
vx list --json              # List installed tools
vx versions node --json     # List available versions
vx which node --json        # Find tool location
```

### Installation

```bash
vx install node@22 --json   # Install tool
vx sync --json              # Sync from vx.toml
```

...
```

#### 4.4 动态 Skills 生成

```rust
/// 动态生成基于项目的 skills
pub async fn generate_project_skill(project_root: &Path) -> Result<String> {
    let context = generate_ai_context(project_root).await?;
    
    Ok(format!(
        r#"---
name: vx-project-context
description: |
  Auto-generated project context for {name}.
  Use this skill to understand the project's tool setup.
---

# Project: {name}

## Tools
{tools}

## Scripts
{scripts}

## Quick Commands
{commands}
"#,
        name = context.project.name,
        tools = format_tools(&context.tools),
        scripts = format_scripts(&context.scripts),
        commands = generate_quick_commands(&context),
    ))
}
```

### 5. AI 常用工具集成

#### 5.1 高优先级新增工具

| 工具 | 用途 | License | 理由 |
|------|------|---------|------|
| **aider** | AI Pair Programming | Apache-2.0 | 与 AI Agent 协同工作 |
| **delta** | Git diff 美化 | MIT | AI 查看代码差异 |
| **hyperfine** | 基准测试 | MIT/Apache-2.0 | AI 性能分析 |
| **tokei** | 代码统计 | MIT/Apache-2.0 | 项目分析 |
| **xq** | XML 处理 | MIT | 数据处理三件套 |
| **llm** | LLM CLI 工具 | Apache-2.0 | 统一 LLM 接口 |

#### 5.2 工具别名优化

```toml
# AI 常用工具别名
[aliases]
# 代码搜索 (AI 高频)
grep = "rg"
find = "fd"
cat = "bat"

# 数据处理
xml = "xq"
yaml = "yq"
json = "jq"

# Git 增强
diff = "delta"
```

### 6. 命令执行优化

#### 6.1 快速模式

新增 `--quick` 模式，跳过非必要检查：

```bash
# 正常模式：~100ms
vx node --version

# 快速模式：~10ms（跳过版本检查、远程查询）
vx --quick node --version
```

#### 6.2 批量命令支持

```bash
# 执行多个命令（减少进程启动开销）
vx exec --batch <<'EOF'
node --version
npm install
npm test
EOF
```

#### 6.3 输出缓存

```rust
/// 缓存常用命令输出
pub struct OutputCache {
    /// 缓存目录
    cache_dir: PathBuf,
    /// TTL (默认 5 分钟)
    ttl: Duration,
}

impl OutputCache {
    /// 获取缓存的 versions 输出
    pub fn get_versions(&self, runtime: &str) -> Option<CachedOutput> {
        let key = format!("versions-{}", runtime);
        self.get(&key)
    }
    
    /// 缓存 versions 输出
    pub fn cache_versions(&self, runtime: &str, output: &str) {
        let key = format!("versions-{}", runtime);
        self.set(&key, output, Duration::from_secs(300));
    }
}
```

### 7. AI 会话状态

#### 7.1 状态文件

```bash
# AI 会话状态存储
~/.vx/ai-session.json
```

```json
{
  "session_id": "abc123",
  "project_root": "/path/to/project",
  "active_tools": {
    "node": "22.0.0",
    "uv": "0.5.14"
  },
  "last_check": "2026-02-19T10:30:00Z",
  "context_hash": "sha256:..."
}
```

#### 7.2 状态命令

```bash
# 初始化 AI 会话
vx ai session init

# 获取会话状态
vx ai session status --json

# 清理会话
vx ai session cleanup
```

## 实施计划

### Phase 1: 结构化输出完善 (v0.15.0)

- [ ] 为 `list`, `versions`, `which`, `check` 实现 `CommandOutput`
- [ ] 统一 `--json` 参数到全局
- [ ] 添加输出格式验证测试
- [ ] 更新 Skills 文档

### Phase 2: AI Context 命令 (v0.15.0)

- [ ] 实现 `vx ai context` 命令
- [ ] 支持 Markdown/JSON 输出
- [ ] 集成项目分析结果
- [ ] 添加缓存机制

### Phase 3: TOON 格式支持 (v0.16.0)

- [ ] 实现 `ToonSerializer`
- [ ] 为所有 `CommandOutput` 实现 TOON 渲染
- [ ] 添加 `--format toon` 参数
- [ ] 更新 Skills 指导 AI 使用 TOON

### Phase 4: Skills 增强 (v0.16.0)

- [ ] 创建 vx-commands skill
- [ ] 创建 vx-project skill
- [ ] 创建 vx-troubleshooting skill
- [ ] 创建 vx-best-practices skill
- [ ] 实现动态 skill 生成

### Phase 5: AI 工具集成 (v0.17.0)

- [ ] 添加 aider provider
- [ ] 添加 delta provider
- [ ] 添加 hyperfine provider
- [ ] 添加 tokei provider
- [ ] 添加 xq provider

### Phase 6: 执行优化 (v0.18.0)

- [ ] 实现 `--quick` 模式
- [ ] 实现批量命令执行
- [ ] 实现输出缓存
- [ ] 实现 AI 会话状态

### Phase 7: Daemon 模式 (v0.19.0)

- [ ] 实现 `vx daemon start/stop`
- [ ] IPC 通信 (RFC-0034)
- [ ] 命令执行加速 (100ms → 5ms)
- [ ] IDE 集成支持

## 向后兼容性

### 完全兼容

- 所有现有命令不变
- 默认输出格式不变
- Skills 安装位置不变

### 新增功能

- `--json` / `--format toon` 为可选参数
- `vx ai context` 为新命令
- `vx ai session` 为新命令

## 测试策略

### 单元测试

```rust
#[test]
fn test_list_output_json_schema() {
    let output = ListOutput::sample();
    let json = serde_json::to_value(&output).unwrap();
    
    assert!(json["runtimes"].is_array());
    assert!(json["total"].is_u64());
}

#[test]
fn test_toon_output_token_reduction() {
    let output = VersionsOutput::sample_50_versions();
    let json = serde_json::to_string(&output).unwrap();
    let toon = toon::to_string(&output).unwrap();
    
    // TOON should be at least 40% smaller
    assert!(toon.len() < json.len() * 6 / 10);
}
```

### 集成测试

```bash
# E2E 测试
vx list --json | jq '.runtimes[0].name'
vx versions node --format toon | head -5
vx ai context --json | jq '.tools'
```

### AI Agent 测试

```
# 验证 AI 可靠解析
1. AI 执行 vx list --json
2. AI 解析 JSON 获取工具列表
3. AI 执行 vx install <missing-tool>
4. AI 验证安装成功
```

## 成功指标

| 指标 | 当前 | 目标 |
|------|------|------|
| `--json` 覆盖率 | 33% (6/18) | 100% |
| 命令执行时间 | ~100ms | ~5ms (daemon) |
| Token 消耗 (版本列表) | ~2000 | ~800 (TOON) |
| Skills 模块数 | 1 | 5 |
| AI 工具 Provider | 0 | 6 |

## 参考资料

- [TOON Specification](https://github.com/toon-format/toon)
- [Claude Code Skills](https://docs.anthropic.com/claude-code/skills)
- [Cursor Rules](https://docs.cursor.com/advanced/rules)
- RFC-0031: 统一结构化输出
- RFC-0034: IPC 集成
- RFC-0030: Provider 扩展计划

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-02-19 | Draft | 初始草案 |
