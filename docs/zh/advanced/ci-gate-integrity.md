# CI 门禁完整性

vx 如何阻止 `[skip ci]` 标记静默关掉 `main` 上的 CI。

## 失效模式

当 head commit 的提交信息包含以下任一标记时，GitHub 会抑制该 push 事件的**全部**
workflow 运行：

```text
[skip ci]   [ci skip]   [no ci]   [skip actions]   [actions skip]
```

squash 合并的提交信息由两部分组成：PR 标题作为 subject，GitHub 再追加一段自动生成的、
逐条列出**分支上每个提交**的列表。因此分支上某个提交里的标记会一路带进合并提交：

```text
fix(ci): stop skipping every test job in change detection

* fix(ci): stop skipping every test job in change detection
* chore: regenerate workspace-hack (cargo-hakari) [skip ci]   ← 继承了这个标记
* fix(ci): harden change detection base resolution
```

于是这个合并提交带着一个看起来很正常的 subject 落到 `main` 上，并且**没有任何 workflow
运行**。没有任何检查会变红，因为根本没有东西运行过；执行合并的人同样看不见——标记藏在
自动生成的正文里，Actions 标签页在那个提交旁只是没有运行记录。

2026-09-19 就发生过一次：PR #1097 的 squash 合并（`4562a26a`）带着生成列表里的标记
进入 `main`，push 事件产生 0 个运行。代码本身是对的（在之后的提交上验证为绿），而这正
是这种失效模式危险的地方：除非有人按 head SHA 查运行记录，否则它完全不可见。

## 只是引用标记同样会生效

GitHub 是对**整个提交信息**做纯文本匹配。反引号、引号、缩进都不影响匹配结果，所以一条
只是*讨论*这个标记的提交信息，和一条真的想跳过 CI 的提交信息效果完全一样：

```text
fix: explain why the housekeeping commit used a skip marker

The commit read: chore: regenerate workspace-hack (cargo-hakari) [skip ci]
```

这条提交同样不会运行任何 workflow。**提交信息里请用文字描述这个标记**（例如「skip
marker」「跳过标记」），不要粘贴带方括号的原始形式；否则守卫会拒绝该提交，包括这条本身。

文件内容不受影响：被扫描的只有提交信息。

## 三层防护

| 层次 | 位置 | 作用 |
| --- | --- | --- |
| 阻断 | [CI Skip Marker Guard](https://github.com/loonghao/vx/actions/workflows/pr-ci-marker-guard.yml) | PR 标题或分支任一提交带标记时让 PR 失败；它使用 `pull_request_target`，GitHub 只按默认分支上的定义执行，因此本次改动合入 `main` 后即生效 |
| 预防 | `ci.yml` | CI 自己推回 PR 分支的提交不再写这个标记 |
| 发现 | [CI Gate Sentinel](https://github.com/loonghao/vx/actions/workflows/ci-gate-sentinel.yml) | 每小时扫描 `main`，找出没有 push 事件运行的提交 |

前两层在合并前拦住标记；第三层负责它们看不见的情况：在守卫上线之前开启的 PR、直接推到
`main` 的提交，以及通过其他路径进来的标记。

### 第一层：PR 守卫

`CI Skip Marker Guard` 在 `pull_request_target` 上触发，并向 PR head 提交一个
`CI Skip Marker` 状态。它通过 API 读取提交信息，不执行 PR 里的任何代码，因此对 fork 也
是安全的。以下位置出现标记即失败：

- PR 标题（它会成为 squash 的 subject）；
- 分支上任一提交的信息。

合并前可在本地先跑一遍：

```bash
just check-pr-ci-markers 1097      # 或：bash scripts/check_pr_ci_markers.sh 1097
```

### 第二层：不要把标记写进去

`ci.yml` 会重新生成 `workspace-hack` 并推回 PR 分支。过去这个提交带 `[skip ci]`，目的是
避免提交循环；但这个标记从来就不必要——使用仓库 `GITHUB_TOKEN` 的推送不会触发新的
workflow 运行，这是 GitHub 自带的防递归机制。那个标记唯一的实际效果就是造成绕过。

本仓库中任何自动生成提交信息的自动化都不应写入标记；一旦重新出现，PR 守卫会让 PR 失败。

### 第三层：哨兵

`CI Gate Sentinel` 每小时运行一次，检查 `main` 上最近的提交，逐个统计 push 事件的
workflow 运行数，并对没有运行的提交分类：

| 类型 | 级别 | 含义 |
| --- | --- | --- |
| `bypass` | 错误 | 标记位于 squash 合并生成的提交列表里 —— 即不可见的那种情况 |
| `intentional-skip` | 警告 | 标记在作者自己写的正文里，属于有意为之 |
| `missing-runs` | 警告 | 没有标记也没有运行，通常是多提交推送中的中间提交 |
| `marker-ineffective` | 警告 | 有标记但也有运行（例如被重新触发的运行） |
| `bot-commit` | 信息 | 机器人作者；`GITHUB_TOKEN` 推送不触发运行，属预期 |
| `too-recent` | 信息 | 早于宽限期，运行可能还没产生 |

只有 `bypass` 会让运行失败。定时运行失败会通知 workflow 作者，同时该 workflow 会开一个
带 `ci-gate` 标签的 issue，让这次绕过在运行日志过期后依然可见。

本地运行：

```bash
just check-main-ci 25              # 或：python3 scripts/check_main_ci_runs.py --branch main
```

## 合并 PR

squash 时显式指定 subject 与正文，不要采用自动生成的列表。不逐条列举分支提交的正文就
不可能携带标记：

```bash
gh pr merge 1097 --squash \
  --subject "fix(ci): harden change detection base resolution" \
  --body "Diff against the merge base instead of the base tip."
```

合并后确认门禁确实跑过。push 运行数为 0 就说明没有：

```bash
gh api "repos/loonghao/vx/actions/runs?head_sha=<sha>&event=push" --jq .total_count
```

如果结果是 `0`，需要为这个提交重新触发检查——在 `main` 上补一个空提交，或针对该合并
提交重跑 CI workflow。

## 建议的后续动作

`main` 目前**不是**受保护分支，因此任何检查（包括 PR 守卫）都无法单独阻止一次合并。开启
分支保护并把 `CI Skip Marker`、`CI Success` 设为必需状态检查，就能把第一层从「可见的警告」
变成「硬性阻断」。这属于仓库设置而非代码改动，需要仓库所有者决定。

## 相关

- [贡献指南](./contributing.md) —— 通用贡献流程
- [发布流程](./release-process.md) —— release 提交如何进入 `main`
