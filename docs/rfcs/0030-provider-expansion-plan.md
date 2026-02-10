# RFC 0030: Provider 扩展计划 — 基于 x-cmd 生态的工具集成分析

> **状态**: Draft
> **作者**: VX Team
> **创建日期**: 2026-02-10
> **目标版本**: v0.4.0
> **关联**: RFC-0016 (Unix CLI Tool Providers), RFC-0012 (Provider Manifest), RFC-0013 (Manifest-Driven Registration)

## 摘要

本 RFC 分析 x-cmd 项目（500+ 模块）的工具生态，筛选适合 vx 集成的工具，并制定分阶段实施计划。目标是将 vx 从"语言运行时管理器"扩展为"开发者全工具链管理器"，覆盖 CLI 工具、语言运行时、DevOps 工具等领域。

### 背景

x-cmd (AGPL-3.0) 的许可证不允许我们直接集成，但其覆盖的工具生态值得参考。通过分析其 500+ 模块，我们筛选出适合 vx 以 manifest-driven provider 方式集成的独立工具。

---

## 现有 Provider 覆盖（52 个）

### 语言运行时
node, deno, bun, python, go, rust, zig, java, dotnet, uv

### 包管理器
pnpm, yarn, nuget, brew, choco, winget, spack, rez

### 构建工具
cmake, make, meson, ninja, nasm, msbuild, msvc, just, task, vite, protoc

### DevOps/Cloud
docker, kubectl, helm, terraform, awscli, azcli, gcloud, gh

### 其他工具
git, curl, jq, ffmpeg, imagemagick, ollama, openssl, pwsh, dagu, rcedit, pre-commit, release-please, vscode, systemctl, xcodebuild

---

## 工具筛选标准

### 必须满足

1. **许可证兼容** — 非 AGPL/SSPL/CC BY-NC（详见许可证审计章节）
2. **有跨平台二进制发布** — GitHub Releases 或官方下载页提供 .tar.gz / .zip
3. **独立可执行** — 不是 shell 内建或系统组件

### 加分项

- 开发者高频使用
- 能与现有 vx 工具形成生态互补
- 有明确的版本语义（semver）
- 有官方版本 API（避免 GitHub 限流）

### 排除标准

- Shell 内建命令（cd, ls, cp, mv, rm 等）
- 系统级包管理器（apt, dnf, yum, pacman 等）
- 消息平台客户端（dingtalk, discord, telegram, slack 等）
- AI Provider API wrapper（纯 API 客户端，无独立二进制）
- x-cmd 自有模块（bakman, cfgy, hok, xx 等，依赖 x-cmd 运行时）

---

## 推荐新增 Provider

### Tier 1 — Modern CLI Toolkit（最高优先级）

开发者日常高频使用的现代命令行工具，全部有完美的 GitHub 二进制发布。

| 工具 | 说明 | License | 源 | 平台 | 理由 |
|------|------|---------|-----|------|------|
| **fzf** | 通用模糊搜索 | MIT | `junegunn/fzf` | Win/Mac/Linux | 开发者必备，shell 集成 |
| **ripgrep (rg)** | 极速文本搜索 | MIT/Unlicense | `BurntSushi/ripgrep` | Win/Mac/Linux | 替代 grep，vscode 内置使用 |
| **fd** | 现代 find 替代 | MIT/Apache-2.0 | `sharkdp/fd` | Win/Mac/Linux | 与 fzf 搭配，直觉式语法 |
| **bat** | 语法高亮 cat | MIT/Apache-2.0 | `sharkdp/bat` | Win/Mac/Linux | 开发者体验工具 |
| **yq** | YAML/TOML/XML 处理器 | MIT | `mikefarah/yq` | Win/Mac/Linux | 与 jq 配对，DevOps 必备 |
| **starship** | 跨 shell 提示符 | ISC | `starship/starship` | Win/Mac/Linux | 终端美化，Rust 编写 |

**实施建议**: 可作为 `modern-unix` bundle 一起推出。这些工具通常一起安装。

### Tier 2 — 开发工具链扩展

补充语言运行时和开发工作流工具。

| 工具 | 说明 | License | 源 | 平台 | 理由 |
|------|------|---------|-----|------|------|
| **ruby** | Ruby 运行时 | BSD-2-Clause | ruby-lang.org | Win/Mac/Linux | 主流语言，Rails/DevOps |
| **kotlin** | Kotlin 编译器 | Apache-2.0 | JetBrains releases | Win/Mac/Linux | JVM 生态主流语言 |
| **lua** | Lua 运行时 | MIT | lua.org | Win/Mac/Linux | 游戏/嵌入式/Neovim 配置 |
| **delta** | Git diff 美化 | MIT | `dandavison/delta` | Win/Mac/Linux | 搭配 git 使用 |
| **hyperfine** | 命令行基准测试 | MIT/Apache-2.0 | `sharkdp/hyperfine` | Win/Mac/Linux | 性能测试工具 |
| **tokei** | 代码行数统计 | MIT/Apache-2.0 | `XAMPPRocky/tokei` | Win/Mac/Linux | 项目分析，CI 集成 |
| **sd** | 现代 sed 替代 | MIT | `chmln/sd` | Win/Mac/Linux | 简洁的文本替换 |
| **pandoc** | 万能文档转换 | GPL-2.0 | `jgm/pandoc` | Win/Mac/Linux | 文档工作流必备 |
| **hugo** | 静态网站生成器 | Apache-2.0 | `gohugoio/hugo` | Win/Mac/Linux | 博客/文档站点 |
| **xq** | XML 处理器 | MIT | `sibprogrammer/xq` | Win/Mac/Linux | 与 jq/yq 形成 query 三件套 |

### Tier 3 — 垂直领域工具

特定领域高价值工具。

| 工具 | 说明 | License | 源 | 平台 | 理由 |
|------|------|---------|-----|------|------|
| **caddy** | 现代 Web 服务器 | Apache-2.0 | `caddyserver/caddy` | Win/Mac/Linux | 自动 HTTPS，开发服务器 |
| **pixi** | Conda 包管理器 | BSD-3 | `prefix-dev/pixi` | Win/Mac/Linux | 数据科学/Python 生态 |
| **btop** | 资源监控器 | Apache-2.0 | `aristocratos/btop` | Mac/Linux | 现代 top 替代 |
| **ncdu** | 磁盘使用分析 | MIT | 官方发布 | Mac/Linux | 实用系统工具 |
| **nvim** | Neovim 编辑器 | Apache-2.0 | `neovim/neovim` | Win/Mac/Linux | 开发者编辑器 |
| **micro** | 终端编辑器 | MIT | `zyedidia/micro` | Win/Mac/Linux | 轻量级 nano 替代 |
| **jj** | Jujutsu VCS | Apache-2.0 | `jj-vcs/jj` | Win/Mac/Linux | 下一代版本控制 |
| **skopeo** | 容器镜像管理 | Apache-2.0 | `containers/skopeo` | Linux | 配合 docker 使用 |
| **p7zip** | 7-Zip 命令行 | LGPL-2.1 | 官方发布 | Win/Mac/Linux | 解压缩工具 |

### Tier 4 — AI 开发工具（可考虑）

| 工具 | 说明 | License | 源 | 平台 | 理由 |
|------|------|---------|-----|------|------|
| **claude** | Claude CLI | Proprietary (free) | npm 包 | Win/Mac/Linux | AI 编程工具 |
| **aider** | AI Pair Programming | Apache-2.0 | pip 包 | Win/Mac/Linux | AI 辅助编程 |
| **groovy** | Groovy 运行时 | Apache-2.0 | 官方发布 | Win/Mac/Linux | Jenkins/JVM 生态 |

---

## 明确不集成的工具

### x-cmd 有但 vx 不适合集成

| 类别 | 工具示例 | 排除理由 |
|------|----------|----------|
| **Shell 内建** | cd, cp, mv, rm, ls, cat, grep, find, sed, awk, less, head, tail | 操作系统自带，版本管理无意义 |
| **Shell 本身** | fish, elvish (elv), nushell (nu), csh, tcsh | 管理 shell 不是 vx 核心场景 |
| **系统包管理器** | apt, dnf, yum, pacman, scoop, snap, apk, aur, paru, yay | 系统级工具，有自己的生态 |
| **x-cmd 内建模块** | bakman, cfgy, hok, xx, zuz, ccmd, llist, eclist... | 依赖 x-cmd 运行时 |
| **消息平台** | dingtalk, discord, feishu, telegram, slack, qywx (企业微信) | 消息服务不是开发工具 |
| **AI Provider API** | deepseek, gemini, openai, zhipu, moonshot, mistral, siliconflow | 纯 API wrapper，无独立二进制 |
| **系统网络工具** | ssh, gpg, ping, route, sysctl, ifconfig, arp, dns | 系统核心组件 |
| **终端 TUI 封装** | htop, btop (仅 Linux 二进制) | 平台限制太大 |

---

## 许可证审计

### 推荐新增工具的许可证状态

| License | 工具 | 状态 |
|---------|------|------|
| **MIT** | fzf, bat, fd, sd, yq, xq, ncdu, delta, hyperfine, tokei, micro | ✅ 无限制 |
| **MIT/Unlicense** | ripgrep | ✅ 双许可 |
| **MIT/Apache-2.0** | bat, fd, hyperfine, tokei | ✅ 双许可 |
| **Apache-2.0** | kotlin, hugo, caddy, btop, nvim, jj, skopeo, aider, groovy | ✅ 无限制 |
| **ISC** | starship | ✅ 类 MIT |
| **BSD-2-Clause** | ruby | ✅ 无限制 |
| **BSD-3-Clause** | pixi | ✅ 无限制 |
| **GPL-2.0** | pandoc | ⚠️ 仅下载/执行，不链接，安全 |
| **LGPL-2.1** | p7zip | ⚠️ 仅下载/执行，安全 |
| **Proprietary (free)** | claude | ⚠️ 免费使用，需标注 |

### 已阻止

| License | 工具 | 处理 |
|---------|------|------|
| **AGPL-3.0** | x-cmd | ❌ 已删除，copyleft 传染风险 |

---

## 实施路线

### Phase 1: Modern CLI Toolkit（v0.4.0）

**目标**: 6 个工具，全部 manifest-driven provider

```
fzf, ripgrep, fd, bat, yq, starship
```

**工作量**: 每个工具约 1 个 provider.toml（利用现有 manifest 基础设施）

**交付物**:
- 6 个 `provider.toml` manifest
- `vx install fzf`, `vx rg`, `vx fd` 等命令可用
- 可选: `modern-unix` bundle 定义

### Phase 2: 语言运行时 + 开发工具（v0.5.0）

**目标**: 10 个工具

```
ruby, kotlin, lua, delta, hyperfine, tokei, sd, pandoc, hugo, xq
```

**附带任务**:
- Ruby/Kotlin/Lua 需要对应的 project-analyzer 语言模块
- pandoc 需要处理 GPL-2.0 license_note

### Phase 3: 垂直领域工具（v0.6.0）

**目标**: 9 个工具

```
caddy, pixi, btop, ncdu, nvim, micro, jj, skopeo, p7zip
```

### Phase 4: AI 工具（评估后决定）

```
claude, aider, groovy
```

**注意**: AI 工具生态变化快，需评估稳定性后再决定。

---

## 与 RFC-0016 的关系

RFC-0016 (Unix CLI Tool Providers) 已规划了部分工具（jq, curl, ffmpeg 等已实现）。本 RFC 是其扩展，补充了：

1. **Modern Rust/Go CLI 工具** — fzf, ripgrep, fd, bat, delta, sd, hyperfine, tokei
2. **更多语言运行时** — ruby, kotlin, lua, groovy
3. **DevOps/Cloud 工具** — caddy, pixi, skopeo, jj
4. **许可证治理框架** — 所有新 provider 必须通过许可证审计

---

## Bundle 设计（可选）

为常见使用场景定义工具集合，一键安装：

```toml
# bundles/modern-unix.toml
[bundle]
name = "modern-unix"
description = "Modern replacements for classic Unix commands"
tools = ["fzf", "rg", "fd", "bat", "sd", "delta", "hyperfine", "tokei"]

# bundles/data-processing.toml
[bundle]
name = "data-processing"
description = "Data query and transformation tools"
tools = ["jq", "yq", "xq", "pandoc"]

# bundles/devops-essentials.toml
[bundle]
name = "devops-essentials"
description = "Essential DevOps tools"
tools = ["docker", "kubectl", "helm", "terraform", "caddy"]

# bundles/full-stack.toml
[bundle]
name = "full-stack"
description = "Full-stack development toolkit"
tools = ["node", "python", "go", "rust", "docker", "git", "gh"]
```

使用方式：

```bash
vx bundle install modern-unix     # 一键安装 8 个现代 CLI 工具
vx bundle install data-processing # 一键安装数据处理工具
```

---

## 统计总结

| 类别 | 数量 | 说明 |
|------|------|------|
| 现有 Provider | 52 | 已实现 |
| Tier 1 新增 | 6 | Modern CLI, 最高优先 |
| Tier 2 新增 | 10 | 语言 + 开发工具 |
| Tier 3 新增 | 9 | 垂直领域 |
| Tier 4 新增 | 3 | AI 工具（待定） |
| **规划总计** | **~80** | 覆盖主流开发场景 |
| x-cmd 排除 | 300+ | Shell 内建/平台/API 等不适合 |

---

## 参考

- [x-cmd 模块列表](https://x-cmd.com) — 500+ 模块参考（AGPL-3.0, 仅参考设计不集成代码）
- [mise (jdx/mise)](https://github.com/jdx/mise) — 类似工具管理器，Registry 设计参考
- [asdf plugins](https://github.com/asdf-community) — 社区插件生态参考
- RFC-0016: Unix CLI Tool Providers
- RFC-0012: Provider Manifest
- RFC-0013: Manifest-Driven Registration
