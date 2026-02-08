# vx metrics

查看 vx 命令的性能指标和诊断信息。

## 概述

每次执行 `vx` 命令时，系统会自动（通过 OpenTelemetry）收集性能数据，并写入 `~/.vx/metrics/`。`vx metrics` 命令可用于查看和分析这些数据。

## 用法

```bash
# 查看最近一次运行的指标
vx metrics

# 查看最近 N 次运行
vx metrics --last 10

# 导出为 JSON（AI 友好格式）
vx metrics --json

# 生成交互式 HTML 报告
vx metrics --html report.html

# 清理旧指标文件
vx metrics --clean
```

## 选项

| 选项 | 描述 |
|------|------|
| `--last N` | 仅显示最近 N 次运行 |
| `--json` | 以结构化 JSON 格式输出指标 |
| `--html <路径>` | 生成带有 Chart.js 的交互式 HTML 报告 |
| `--clean` | 删除所有指标文件 |

## 管道阶段

vx 为每个命令追踪四个执行管道阶段：

| 阶段 | 描述 |
|------|------|
| `resolve` | 解析运行时版本和依赖 |
| `ensure` | 确保运行时已安装 |
| `prepare` | 准备环境变量和 PATH |
| `execute` | 执行实际命令 |

## 分层追踪过滤器

vx 使用分层过滤来分离控制台输出和指标收集：

- **普通模式**：stderr 上只显示警告和错误。所有 `vx=trace` 级别的 span 仍然会被 OpenTelemetry 层捕获用于指标收集。
- **`--verbose` 模式** (`-v`)：在 stderr 上显示 `vx` 的 debug 消息和其他 crate 的 info 消息。
- **`--debug` 模式**：在 stderr 上显示所有 debug 级别的消息。
- **`RUST_LOG` 环境变量**：使用用户指定的过滤指令覆盖控制台和 OTel 过滤器。

这意味着 `vx node --version` 会产生干净的输出（没有 debug 信息干扰），同时 `vx metrics` 仍然可以分析完整的执行追踪。

## 输出格式

### 终端（默认）

显示管道阶段的瀑布图及时间信息：

```
╭─ vx node --version ──────────────────────────╮
│ resolve  ███                           50ms   │
│ ensure   ████████████████████████████  800ms   │
│ prepare  █                             10ms   │
│ execute  ███████████                   374ms   │
│                                               │
│ Total: 1234ms  Exit: 0                        │
╰───────────────────────────────────────────────╯
```

### JSON (`--json`)

适用于 AI 分析和 CI 集成的结构化输出：

```json
{
  "runs_analyzed": 5,
  "total_ms": { "avg": 1234, "min": 800, "max": 2000, "p50": 1100, "p95": 1900 },
  "stages": {
    "resolve": { "avg_ms": 50 },
    "ensure": { "avg_ms": 800 },
    "prepare": { "avg_ms": 10 },
    "execute": { "avg_ms": 374 }
  },
  "bottleneck": "ensure"
}
```

### HTML (`--html`)

生成包含以下内容的交互式报告：
- 显示性能趋势的折线图
- 阶段分解的堆叠面积图
- 时间分布的饼图
- 运行历史表格

## 指标存储

指标文件以 JSON 格式存储在 `~/.vx/metrics/`：

```
~/.vx/metrics/
├── 20260208_103000_123.json
├── 20260208_103500_456.json
└── ...
```

仅保留最近 50 个文件（旧文件会自动清理）。

## CI 基准测试集成

vx 包含 E2E 基准测试，可在 CI 中运行以检测性能回归。参见 `benchmark.yml` GitHub 工作流，它会在每个 PR 中跨 Linux、Windows 和 macOS 平台运行。

各平台性能阈值：

| 测试 | Linux/macOS | Windows |
|------|------------|---------|
| CLI help | < 350ms | < 500ms |
| CLI version | < 350ms | < 500ms |
| CLI startup | < 3000ms | < 3000ms |
| 配置解析（小） | < 1000ms | < 1500ms |
| 配置解析（大） | < 3000ms | < 3000ms |
| Setup dry-run（小） | < 1000ms | < 1000ms |
| Setup dry-run（大） | < 3000ms | < 3000ms |
| 脚本列表 | < 1000ms | < 1000ms |
| 配置验证 | < 1000ms | < 1500ms |
