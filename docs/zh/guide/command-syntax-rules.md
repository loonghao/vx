# 统一命令语法规则表

本文定义 vx 在所有生态与运行时上的**统一语法契约**。  
它是未来 CLI 演进与文档一致性的权威参考。

## 目标

- 用一套语法模型降低学习成本。
- 在保持简洁用法（`vx <runtime> ...`）的同时提供显式语法。
- 让解析规则可预测、无歧义。
- 在不破坏现有用户习惯的前提下推进命令整合。

## 核心原则

- **一个前缀**：所有托管执行都以 `vx` 开头。
- **一个意图一个主语法**：每类意图只推荐一个规范写法。
- **兼容优先**：迁移期保留旧语法作为别名。
- **确定性解析**：不做隐式重解释。

## 语法规则表

| 意图 | 规范语法 | 示例 | 兼容策略 |
|---|---|---|---|
| 运行时执行 | `vx <runtime>[@runtime_version] [args...]` | `vx node@22 --version` | 保留现有直接运行时写法。 |
| 捆绑运行时执行 | `vx <bundled_runtime>[@parent_version] [args...]` | `vx npx@20 create-react-app my-app` | 对于捆绑运行时，`@version` 指父运行时的版本。 |
| 运行时可执行覆盖 | `vx <runtime>[@runtime_version]::<executable> [args...]` | `vx msvc@14.42::cl main.cpp` | 兼容 `runtime::exe@version`，但规范写法为版本前置。 |
| 包执行 | `vx <ecosystem>[@runtime_version]:<package>[@package_version][::executable] [args...]` | `vx uvx:pyinstaller::pyinstaller --version` | 这是唯一包语法。 |
| 多运行时组装执行 | `vx --with <runtime>[@runtime_version] [--with <runtime>[@runtime_version] ...] <target_command>` | `vx --with bun@1.1.0 --with deno node app.js` | `--with` 仅为本次执行注入伴随运行时。 |
| Shell 启动 | `vx shell <runtime>[@runtime_version] [shell]` | `vx shell node@22 powershell` | `vx node::powershell` 作为兼容别名保留。 |
| 环境组装管理 | `vx env <create|use|list|show|add|remove|sync|shell|delete> ...` | `vx env sync` | `vx dev` 仍用于项目优先的交互式场景。 |
| 项目脚本运行 | `vx run <script> [-- <args...>]` | `vx run test -- --nocapture` | 脚本定义来源于 `vx.toml`。 |
| 项目状态同步 | `vx sync [--check]` + `vx lock [--check]` | `vx sync --check` | `sync` 与 `lock` 共同保证可复现性。 |
| 全局包管理 | `vx pkg <add|rm|ls|info|shim-update> ...` | `vx pkg add npm:typescript@5.3` | `vx global ...` 作为兼容别名保留。 |
| 项目工具链管理 | `vx project <init|add|rm|sync|lock|check> ...` | `vx project sync` | 现有顶层命令继续可用并映射为别名。 |

## 保留符号

| 符号 | 含义 | 规则 |
|---|---|---|
| `@` | 版本选择符 | 运行时版本放在 `:` 前，包版本放在包名后。 |
| `:` | 生态/包分隔符 | 仅用于包执行语法。 |
| `::` | 可执行选择符 | 仅用于显式选择可执行文件或 shell。 |

## 解析优先级（确定性）

1. 第一个参数命中显式子命令，则进入子命令解析。
2. 否则若命中包语法（`<eco>...:<package>...`），按包执行解析。
3. 否则若包含 `::`，按运行时可执行/ shell 语法解析。
4. 否则按运行时或已安装 shim 执行解析。

## 版本决策策略（统一）

所有执行路径（runtime/package/shim）统一为：

1. CLI 显式版本
2. `vx.lock`
3. `vx.toml`
4. 最新兼容版本

## 全局执行选项契约（输出 + 缓存）

这些参数属于跨场景语法契约，runtime/package/project 执行都应保持一致：

| 关注点 | 规范语法 | 规则 |
|---|---|---|
| JSON 结构化输出 | `--json` | `--output-format json` 的快捷方式；与 `--output-format` 同时出现时优先。 |
| TOON（LLM 友好）输出 | `--toon` | `--output-format toon` 的快捷方式；与 `--output-format` 同时出现时优先。 |
| 显式输出模式 | `--output-format <text|json|toon>` | 未使用快捷参数时的显式主写法。 |
| 缓存策略 | `--cache-mode <normal|refresh|offline|no-cache>` | 所有执行路径统一缓存控制语义。 |

决策说明：

- 输出参数优先级：快捷参数（`--json` / `--toon`）> `--output-format` > 环境默认值。
- `cache-mode` 的解析和执行语义必须在解析器、执行器、文档示例中保持一致。

## 能力覆盖矩阵（核心场景）


| 场景 | 范围 | 规范入口 |
|---|---|---|
| 单运行时/包/shim 执行 | 日常命令执行 | `vx <runtime>...` / 包语法 |
| 多运行时组装 | 单次注入伴随运行时 | `vx --with ... <target_command>` |
| 项目感知执行 | 按项目上下文解析工具链 | `vx run` / `vx sync` / `vx lock` |
| 环境组装与复用 | 构建并复用命名/项目环境 | `vx env ...` / `vx dev` |
| 解析 + 状态同步 | 保持解析器、文档、锁文件一致 | 本规范 + 解析测试 + `vx sync/lock` 检查 |

## 多环境组装规则

- `--with` 使用 `runtime[@version]` 语法，可重复声明。
- 伴随运行时只在**当前这次执行**中生效，不改变全局默认状态。
- 每个 `--with` 运行时都按统一版本策略进行解析。
- `--with` 不替代主目标命令，只补齐执行前置依赖。

## 项目感知执行契约（`vx.toml` + `vx.lock`）

- 项目上下文从当前目录向上发现。
- 在项目上下文中，runtime/package/shim 执行必须遵循同一版本策略。
- `vx sync` 是从 `vx.toml` 到本地运行时状态的期望态对齐命令。
- `vx lock` 负责固化可复现版本，`vx lock --check` / `vx sync --check` 负责漂移检测。
- `vx run <script>` 与直接命令执行共享同一解析/决策语义。

## 解析与同步契约（治理规则）

任何语法变更都必须同步更新：

1. CLI 解析行为与测试（`crates/vx-cli/tests/`）
2. 规范文档（`docs/guide/command-syntax-rules.md` 与 `/zh/` 对应文档）
3. Agent 护栏（`AGENTS.md`）
4. CLI 帮助示例（`crates/vx-cli/src/cli.rs` 长帮助）

禁止对歧义 token 做静默重解释。保留兼容别名时必须提供显式迁移提示。

## 禁止/弃用模式


- 禁止：`vx uvx::pyinstaller`（非法包语法），应使用 `vx uvx:pyinstaller`。
- 弃用文档示例：`vx install <runtime> <version>`，统一改为 `vx install <runtime>@<version>`。
- `runtime::shell` 不再作为主推荐写法，文档默认使用 `vx shell ...`。

## CLI 命令整合路线

### 阶段 1（文档统一 + 别名）

- 引入规范分组：
  - `vx pkg ...`（全局包生命周期）
  - `vx project ...`（项目工具链生命周期）
- 现有命令继续作为别名保留（`global`、`add`、`remove`、`sync`、`lock`、`check`、`init`）。

### 阶段 2（交互提示）

- 对非规范调用输出一行迁移提示。
- 提供 `--no-hints` 用于 CI 静默场景。

### 阶段 3（规则固化）

- 冻结语法并补齐解析测试矩阵。
- 高频兼容别名长期保留，低频历史写法按使用率逐步下线。

### 阶段 4（历史能力清理）

- 先清理非规范文档/示例，再对低价值历史别名增加告警并逐步隐藏。
- 优先清理对象：
  - 已被 `vx shell ...` 覆盖的重复 shell 写法，
  - 已弃用安装写法（`vx install <runtime> <version>`），
  - 语义歧义或低使用率历史拼写。
- 任何行为级下线都必须先满足“使用率评估 + 迁移提示窗口”。


## 文档约束

所有新增 CLI/指南/工具文档必须包含：

- 一行规范语法。
- 一组简短示例。
- 若存在别名，明确标注“兼容别名”。

## 测试约束（解析 + 文档）

- 增加 runtime/package/shell/shim 冲突场景解析测试矩阵。
- 增加文档 lint，拦截禁用旧语法。
- 保证 `README.md`、`README_zh.md` 与 CLI 文档在规范语法上严格一致。
