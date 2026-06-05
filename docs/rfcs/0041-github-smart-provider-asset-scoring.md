# RFC 0041: github_smart_provider 资产评分与智能匹配

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2026-06-05
> **目标版本**: v0.10.0

---

## 摘要

本 RFC 提出在 vx 的 Provider 模板体系中新增 `github_smart_provider` 模板，借鉴 [eget](https://github.com/zyedidia/eget) 的 Detect 阶段设计，实现基于资产文件名评分的自动匹配。

当前 vx 拥有 4 个 Provider 模板（`github_rust_provider`、`github_go_provider`、`github_binary_provider`、`system_provider`），每个模板要求 Provider 作者预先知道目标项目的精确资产命名规则（triple、Go os/arch 等）。当项目的资产命名不规则时（如 hugo 按平台使用不同扩展名、ffmpeg 使用自定义子目录结构），Provider 作者必须手写 `download_url` 和 `install_layout`，导致大量重复的平台分支代码。

`github_smart_provider` 的目标：Provider 作者只需指定 owner/repo，系统自动从 GitHub Release 的 asset 列表中选出最佳匹配。

---

## 动机

### 现状痛点

对现有 Provider 的分析显示，约 35% 的 Provider 无法使用现有 4 个模板中的任何一个，需要手写 `download_url`：

```
┌──────────────────────────────────────────────────────────────────┐
│                  Provider 模板覆盖率                              │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  github_rust_provider    ████████████████████████  ~55 (39%)     │
│  github_go_provider      ██████████████            ~32 (23%)     │
│  github_binary_provider  ████                      ~6  (4%)      │
│  system_provider         ██████                    ~12 (8%)      │
│  手写 download_url        ████████████              ~50 (35%)     │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│  手写类全部可以从 github_smart_provider 受益                     │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

手写 Provider 的样板代码模式（来自 dive、gitleaks、hugo 等）：

```python
# 每个手写 Provider 都在重复这段逻辑
_PLATFORMS = {
    "windows/x64":   ("windows", "amd64"),
    "windows/arm64": ("windows", "arm64"),
    "macos/x64":     ("darwin",  "amd64"),
    "macos/arm64":   ("darwin",  "arm64"),
    "linux/x64":     ("linux",   "amd64"),
    "linux/arm64":   ("linux",   "arm64"),
}

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/{owner}/{repo}/releases/download/v{}/{}_{}_{}_{}.{}".format(
        version, name, version, os_str, arch_str, ext)
```

### eget 的四阶段模型

eget 的安装流程分为四个阶段，其中 Detect 是本 RFC 的核心参考：

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Find    │───→│  Detect  │───→│  Verify  │───→│  Extract │
│ 发现版本  │    │ 匹配资产  │    │ 校验完整性 │    │ 解压安装  │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
                     ▲
                     │ 本 RFC 聚焦
                     │
      ┌──────────────┴──────────────┐
      │ 1. 列出 Release 所有 assets │
      │ 2. 按 OS / arch / libc /    │
      │    format 评分              │
      │ 3. 选择最高分 asset         │
      │ 4. 分数低于阈值时 fallback  │
      └─────────────────────────────┘
```

vx 已实现 Find（`fetch_versions`）和 Extract（`install_layout` + Rust 执行），缺少的是 Detect 阶段的智能化。

---

## 主流方案调研

### eget (zyedidia/eget)

eget 的 `detect.go` 实现了约 200 行的资产检测逻辑，核心评分维度：

| 维度 | 权重 | 检测方式 |
|------|------|----------|
| OS 匹配 | 高 | 文件名包含 `linux`/`darwin`/`windows`/`win` |
| Arch 匹配 | 高 | 文件名包含 `amd64`/`x86_64`/`arm64`/`aarch64` |
| 格式偏好 | 中 | `.tar.gz` > `.tar.xz` > `.tar.bz2` > `.zip` |
| 版本匹配 | 必须 | 文件名包含版本号字符串 |
| 反匹配 | 排除 | 文件名包含 `checksum`/`sha256`/`sbom`/`deb`/`rpm` |

eget 的处理完全在 Go 端完成，资产列表通过 GitHub API 获取。

### mise (jdx/mise)

mise 使用类似的"后端列表 + 正则匹配"模式：

- 从 GitHub Release API 获取 assets 列表
- 使用预定义的正则模式匹配不同项目（`mise-{version}-{target}.tar.xz`）
- 每个 plugin 可以注册自定义的 asset 匹配函数

### mise 的缓存策略

- 使用 HTTP ETag / If-None-Match 做条件请求
- GitHub API 响应返回 `304 Not Modified` 时不消耗 rate limit
- 本地 TTL：24 小时（版本列表）/ 1 小时（release assets）

### 对本 RFC 的启示

1. eget 的评分逻辑可以移植到 vx 的 Starlark DSL 层
2. mise 的 ETag 缓存策略可以复用 vx 现有的 HTTP 缓存基础设施
3. 智能匹配不是银弹 — 必须保留显式 `asset` 参数作为 fallback/precision 选项

---

## 详细设计

### 1. Asset 评分规则

#### 1.1 评分维度

每个 asset 文件名经过以下维度打分，满分 100：

| 维度 | 满分 | 评分规则 |
|------|------|----------|
| `os_match` | 35 | 精确匹配操作系统 |
| `arch_match` | 30 | 精确匹配 CPU 架构 |
| `libc_pref` | 15 | Linux 下 libc 偏好（musl > gnu） |
| `format_pref` | 15 | 归档格式偏好 |
| `keyword_bonus` | 5 | 额外关键字加分 |
| **总计** | **100** | |

额外约束：
- **硬排除**：文件名包含 `checksum`, `sha256`, `sha512`, `md5`, `sbom`, `attestation`, `source`, `src.tar`, `-src.`, `.deb`, `.rpm`, `.apk`, `.msi`, `.pkg`, `.dmg`, `.appimage` → 直接排除
- **最低阈值**：总分 < 40 → 视为不匹配，不返回结果
- **版本必须出现**：版本号字符串（去除 `v` 前缀后）必须在文件名中出现 → 否则排除

#### 1.2 OS 匹配（35 分）

采用 **别名归一化** 后精确匹配。归一化规则：

```
输入 (ctx.platform.os)    归一化后
─────────────────────────────────
windows                    windows
macos                      darwin  (注意: vx 用 "macos", 资产名常用 "darwin")
linux                      linux
```

别名映射表（资产文件名中的 os 标识 → 归一化）：

| 资产中的 OS 标识 | 归一化为 |
|-----------------|---------|
| `windows`, `win64`, `win32`, `windows-x64`, `pc-windows`, `win` | `windows` |
| `darwin`, `macos`, `macosx`, `mac`, `apple-darwin`, `osx` | `darwin` |
| `linux`, `unknown-linux`, `linux-gnu`, `linux-musl` | `linux` |

匹配规则：
- 完整别名匹配：+35 分
- 部分子串匹配（如 `linux` 在 `linux-musl` 中被找到）：+25 分
- 无匹配：0 分（如果另一个 os 完全匹配则整体排除）

#### 1.3 Arch 匹配（30 分）

架构别名归一化：

| 资产中的 Arch 标识 | 归一化为 |
|-------------------|---------|
| `x86_64`, `x64`, `amd64`, `x86-64`, `win64` | `x86_64` |
| `arm64`, `aarch64`, `armv8`, `arm64-v8a` | `aarch64` |
| `x86`, `i686`, `i386`, `386`, `win32` | `i686` |

匹配规则：
- 精确别名匹配：+30 分
- 同位数交叉匹配（如 `universal` 在 macOS 上同时匹配 x64 和 arm64）：+20 分
- 通用/多架构标记（`all`, `any`, `portable`, `multi`, `universal`, `fat`）：+15 分
- 无匹配：0 分

#### 1.4 Libc 偏好（15 分，仅 Linux）

Linux 平台下评估 libc 类型。非 Linux 平台此项固定 +15 分（因为无关）。

检测方式：在 asset 文件名中搜索子串：

```
musl 标记: "musl", "static", "alpine"
gnu  标记: "gnu", "glibc", "gnueabihf"

偏好顺序: musl > gnu (musl 静态链接，跨发行版兼容)
```

评分：
- 偏好 libc (musl) 匹配：+15 分
- 非偏好 libc (gnu) 匹配：+5 分
- 未检测到 libc 标记：+8 分（中性）
- Linux 平台且文件名明显是另一个 libc（当前是 musl 偏好但文件名含 `gnu`）：+3 分

#### 1.5 格式偏好（15 分）

平台感知的格式偏好：

| 平台 | 偏好顺序 | 说明 |
|------|---------|------|
| Linux | `.tar.gz` > `.tar.xz` > `.tar.bz2` > `.tgz` > `.zip` | tar.gz 是 Linux 最通用格式 |
| macOS | `.tar.gz` > `.tar.xz` > `.zip` > `.tar.bz2` | macOS 原生支持 zip，但 tar.gz 更通用 |
| Windows | `.zip` > `.tar.gz` > `.7z` > `.tar.xz` > `.tar.bz2` | zip 在 Windows 上原生支持最好 |

评分：
- 首选格式：+15 分
- 次选格式：+10 分
- 第三选择：+5 分
- 其他可解压格式：+2 分
- 不可处理的格式（如 `.msi`, `.dmg`）：已在硬排除中处理

#### 1.6 关键字加分（5 分）

Asset 文件名中的额外关键字可以提供"确认信号"：

| 关键字 | 加分 | 说明 |
|--------|------|------|
| `static` | +3 | 静态链接二进制，跨发行版兼容 |
| `portable` | +3 | 便携版，无外部依赖 |
| `standalone` | +3 | 独立运行 |
| 非平台特有尾缀 | +1 | 精确匹配的冗余确认 |

**关键字加分上限**：即使匹配多个关键字，此项总分不超过 5 分。

#### 1.7 平分决胜规则

当多个 asset 得分相同时，按以下优先级决胜：

| 优先级 | 规则 | 说明 |
|--------|------|------|
| 1 | 文件大小较小 | 优先选择更小的下载，减少带宽消耗 |
| 2 | 文件名长度较短 | 较短的文件名通常意味着更简洁的命名 |
| 3 | 字母序靠前 | 确定性决胜，保证跨平台一致 |

#### 1.8 评分示例

以 Linux x64 (musl 偏好) 安装 ripgrep 14.1.1 为例：

```
Asset                                                     OS  Arch Libc Fmt Key Total
─────────────────────────────────────────────────────────────────────────────────────
ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz          35  30   15   15  0   95 ✓
ripgrep-14.1.1-x86_64-unknown-linux-gnu.tar.gz           35  30   5    15  0   85
ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz.sha256   ── 硬排除 (sha256) ──
ripgrep-14.1.1-x86_64-apple-darwin.tar.gz                0   30   15   15  0   ── 排除(macOS)
ripgrep-14.1.1-x86_64-pc-windows-msvc.zip                0   30   15   5   0   ── 排除(Windows)
ripgrep_14.1.1-1_amd64.deb                               ── 硬排除 (deb) ──
```

Windows x64 安装同一个版本：

```
Asset                                                     OS  Arch Libc Fmt Key Total
─────────────────────────────────────────────────────────────────────────────────────
ripgrep-14.1.1-x86_64-pc-windows-msvc.zip                35  30   15   15  0   95 ✓
ripgrep-14.1.1-i686-pc-windows-msvc.zip                  35  0    15   15  0   65 (>40，保留)
ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz          0   30   15   2   0   ── 排除(Linux)
```

### 2. Fallback 语义

`github_smart_provider` 采用 **三级 fallback** 机制：

```
┌─────────────────────────────┐
│ Level 1: Smart Detect       │  github_smart_provider 默认行为
│ 自动评分选择最佳 asset       │
├─────────────────────────────┤
│ Level 2: Explicit Template  │  用户传入 asset 参数
│ 使用精确 asset 模板匹配      │
├─────────────────────────────┤
│ Level 3: system_install     │  author 手写兜底
│ 回退到系统包管理器           │
└─────────────────────────────┘
```

#### 2.1 模板 API

```python
# 最小用法：只需 owner/repo
_p = github_smart_provider("BurntSushi", "ripgrep", executable="rg")

# 带 fallback：smart 检测失败时使用显式模板
_p = github_smart_provider(
    "cli", "cli",
    executable = "gh",
    asset      = "gh_{version}_{os}_{arch}.{ext}",  # fallback 模板
    strip      = "gh_{version}_{os}_{arch}",         # fallback strip_prefix
)

# 完整签名
def github_smart_provider(
    owner, repo,
    executable = None,
    store = None,
    asset = None,          # fallback 显式模板（smart 失败时使用）
    strip = None,          # fallback strip_prefix
    tag_prefix = "v",
    linux_libc = "musl",   # Linux libc 偏好
    prereleases = False,
    path_env = True,
    # 高级参数（调优评分）
    score_threshold = 40,  # 最低分数阈值
    extra_excludes = [],   # 额外的排除模式
    os_overrides = {},     # OS 别名覆盖
    arch_overrides = {},   # Arch 别名覆盖
    format_prefs = None,   # 格式偏好覆盖
):
```

#### 2.2 行为矩阵

| 场景 | Smart Detect 结果 | 行为 |
|------|------------------|------|
| 找到 >= 阈值 | 最佳 asset URL | 使用 smart 结果 |
| 找到 < 阈值 | 无结果 | 尝试 `asset` fallback 模板 |
| 未找到 + 有 `asset` | 使用显式模板 | 生成 URL，行为同 `github_binary_provider` |
| 未找到 + 无 `asset` | None | 调用方可叠加 `system_install` 兜底 |
| GitHub API 故障 | 异常 | 缓存命中 → 用缓存；缓存未命中 → fallback 到 asset 模板 |

### 3. Starlark ↔ Rust 边界

#### 3.1 设计原则

保持 vx 的两层架构不变：**Starlark 提供 descriptor，Rust 执行操作**。

新增的 Rust 能力只有一个原语：

```
ctx.list_release_assets(owner, repo, tag) → list[AssetInfo]
```

#### 3.2 新增类型

```rust
// Rust side — 新增 context 方法
#[derive(Debug, Clone, serde::Serialize)]
struct AssetInfo {
    name: String,               // 文件名
    size: u64,                  // 大小（字节）
    download_count: u64,        // 下载次数（Phase 1 不使用，预留供未来分析）
    browser_download_url: String, // 直接下载 URL
}
```

#### 3.3 职责划分

```
┌─ Starlark (DSL) ─────────────────────────────────────────────┐
│                                                               │
│  github_smart_provider(owner, repo, ...):                      │
│    1. call ctx.list_release_assets(owner, repo, tag)          │
│    2. score each asset against ctx.platform                   │
│    3. pick best match → return asset.browser_download_url     │
│    4. if no match → try asset template fallback               │
│                                                               │
│  评分逻辑完全在 Starlark stdlib (smart_detect.star) 中实现     │
│  用户可覆盖 _score_asset() 自定义评分逻辑                     │
│                                                               │
├─ Rust (Execution) ───────────────────────────────────────────┤
│                                                               │
│  ctx.list_release_assets(owner, repo, tag):                   │
│    1. check cache → return (with ETag)                        │
│    2. call GET /repos/{owner}/{repo}/releases/tags/{tag}      │
│    3. extract assets[] array                                  │
│    4. cache with TTL + ETag                                   │
│    5. return list[AssetInfo] to Starlark                      │
│                                                               │
│  不包含任何评分逻辑                                           │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

**为什么评分放 Starlark 而非 Rust？**

1. **可覆盖性**：Provider 作者可以在 Starlark 中覆盖评分逻辑（如排除某些 asset、调整权重）
2. **可观测性**：`--debug` 日志可以直接在 Starlark 侧用 `print()` 输出
3. **迭代速度**：stdlib 的 `.star` 文件可以独立更新，不需要重新编译 Rust
4. **一致性**：现有的 `github_rust_provider` 等模板也在 Starlark 侧做 URL 构建

#### 3.4 `list_release_assets` 接口

Rust 侧暴露给 Starlark 的接口：

```rust
// 在 StarlarkEngine 中注册
fn list_release_assets(
    ctx: &ProviderContext,
    args: Vec<String>,
) -> Result<Vec<AssetInfo>> {
    // args[0] = owner, args[1] = repo, args[2] = tag
    // 内部处理：缓存检查 → GitHub API 调用 → 缓存写入
}
```

Starlark 调用方式：

```python
# 在 provider.star 中
assets = ctx.list_release_assets(owner, repo, "v" + version)
# assets 是一个 list[dict]，每个 dict 有 name, size, browser_download_url
```

### 4. 缓存策略

#### 4.1 缓存层级

```
┌──────────────────────────────────────────────────────────────┐
│ Layer 1: 内存缓存 (per-session)                               │
│ TTL: session lifetime                                        │
│ 命中即返回，不发起任何 HTTP 请求                               │
├──────────────────────────────────────────────────────────────┤
│ Layer 2: HTTP ETag 缓存 (per-release tag)                    │
│ TTL: 1 小时（磁盘缓存）                                      │
│ 二次请求时带 If-None-Match → 304 不消耗 rate limit           │
├──────────────────────────────────────────────────────────────┤
│ Layer 3: 预取缓存（install 时并发预热）                       │
│ 在 install 命令执行时，提前 fetch 目标版本的 assets           │
│ 利用并行下载的时间窗口完成 API 调用                           │
└──────────────────────────────────────────────────────────────┘
```

#### 4.2 缓存键设计

```
disk: {vx_cache_dir}/github/assets/{owner}/{repo}/{tag}.json
mem:  HashMap<String, (Vec<AssetInfo>, Instant)>
```

缓存内容：

```json
{
  "tag": "v14.1.1",
  "etag": "\"abc123def456\"",
  "fetched_at": "2026-06-05T12:00:00Z",
  "assets": [
    {
      "name": "ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz",
      "size": 2048576,
      "browser_download_url": "https://github.com/..."
    }
  ]
}
```

#### 4.3 GitHub API Rate Limit 预算

```
未认证:  60 req/hour → 只能支持 ~4 个不同 release 的查询/小时
已认证 (GITHUB_TOKEN):  5000 req/hour
已认证 (vx app token):  5000 req/hour

推荐：vx 默认携带 GITHUB_TOKEN（如果环境变量已设置）
     fallback 到未认证请求
     当 rate limit 耗尽时 → 走缓存 → 走 fallback
```

#### 4.4 缓存失效

| 触发条件 | 行为 |
|----------|------|
| `vx install <tool>@<version>` | 检查 ETag；304 → 用缓存；200 → 更新缓存 |
| `vx versions --refresh <tool>` | 主动失效该 repo 的所有 release 缓存 |
| 缓存 TTL 过期（1h） | 下次请求带 If-None-Match 条件请求 |
| `vx cache clean` | 清除所有 GitHub asset 缓存 |

### 5. Debug 可观测性

#### 5.1 输出格式

`vx --debug install <tool>` 输出评分日志：

```
[DEBUG vx::smart_detect] Scoring 12 GitHub release assets for ripgrep v14.1.1 (tag=v14.1.1):
[DEBUG vx::smart_detect]   ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz
                                  os=+35(triple:linux) arch=+30(x86_64) libc=+15(musl) fmt=+15(tar.gz) kw=+0 → 95
[DEBUG vx::smart_detect]   ripgrep-14.1.1-x86_64-unknown-linux-gnu.tar.gz
                                  os=+35(triple:linux) arch=+30(x86_64) libc=+5(gnu)   fmt=+15(tar.gz) kw=+0 → 85
[DEBUG vx::smart_detect]   ripgrep-14.1.1-x86_64-apple-darwin.tar.gz
                                  os=+0(os:mismatch:darwin) → excluded
[DEBUG vx::smart_detect]   ripgrep-14.1.1-x86_64-pc-windows-msvc.zip
                                  os=+0(os:mismatch:windows) → excluded
[DEBUG vx::smart_detect]   ripgrep-14.1.1-i686-pc-windows-msvc.zip
                                  os=+0(os:mismatch:windows) → excluded
[DEBUG vx::smart_detect]   ripgrep-14.1.1-aarch64-apple-darwin.tar.gz
                                  os=+0(os:mismatch:darwin) → excluded
[DEBUG vx::smart_detect]   ripgrep_14.1.1-1_amd64.deb
                                  → hard-excluded (keyword: deb)
[DEBUG vx::smart_detect]   ---
[DEBUG vx::smart_detect]   ✓ Selected: ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz (score: 95/100)
[DEBUG vx::smart_detect]   Threshold: 40 | Candidates above threshold: 2 | Fallback: not needed
```

#### 5.2 非 debug 模式输出

`vx install ripgrep`（无 `--debug`）：

```
ℹ Installing ripgrep v14.1.1 (auto-detected asset)
  Downloading ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz...
```

#### 5.3 Fallback 触发日志

```
[DEBUG vx::smart_detect] Smart detect: no asset scored ≥40 for tag v1.2.3
[DEBUG vx::smart_detect] Falling back to explicit asset template
[DEBUG vx::smart_detect]   Template: "mytool-{vversion}-{triple}.{ext}"
[DEBUG vx::smart_detect]   Resolved: mytool-v1.2.3-x86_64-unknown-linux-musl.tar.gz
```

### 6. 新 stdlib 文件 `smart_detect.star`

评分逻辑封装在 `crates/vx-starlark/stdlib/smart_detect.star`：

```python
# @vx//stdlib:smart_detect.star
# Asset scoring and smart detection for GitHub releases

# ── 硬排除列表 ──
_EXCLUDE_KEYWORDS = [
    "checksum", "sha256", "sha512", "md5", "sha1",
    "sbom", "attestation", "spdx",
    "source", "src.tar", "-src.",
    ".deb", ".rpm", ".apk", ".msi", ".pkg", ".dmg", ".appimage",
    ".sig", ".asc", ".pem",
]

# ── OS 别名映射 ──
# ...

# ── Arch 别名映射 ──
# ...

# ── Libc 标记 ──
# ...

def score_asset(name, ctx, version, linux_libc = "musl"):
    """Score a single asset against the current platform.
    
    Returns:
        None if excluded, dict with scores otherwise.
    """

def detect_best_asset(assets, ctx, version,
                       threshold = 40,
                       linux_libc = "musl",
                       extra_excludes = None):
    """Score all assets and return the best match.
    
    Returns:
        The best AssetInfo if score >= threshold, otherwise None.
        Also returns all candidates above threshold for debug logging.
    """

def build_smart_download_url(ctx, version, owner, repo,
                              tag_prefix = "v",
                              linux_libc = "musl",
                              score_threshold = 40,
                              fallback_asset = None):
    """Full detect-and-fallback pipeline.
    
    Returns:
        Download URL string, or None.
    """
```

### 7. 用户影响与兼容性

#### 7.1 向后兼容

- 现有 142 个 Provider 行为不变
- `github_smart_provider` 是**可选 opt-in 模板**，不影响任何已有 Provider
- 新旧模板可共存：已有 Provider 无需迁移

#### 7.2 迁移路径

对于手写 `download_url` 的 Provider（如 dive、gitleaks），可以选择迁移到 `github_smart_provider`：

```python
# 迁移前 (dive, 60+ 行手写)
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions", ...)
_PLATFORMS = { ... }
def download_url(ctx, version):
    key = ...; platform = _PLATFORMS.get(key); ...
install_layout = archive_layout("dive")

# 迁移后 (dive, ~20 行)
load("@vx//stdlib:provider_templates.star", "github_smart_provider")
_p = github_smart_provider("wagoodman", "dive",
    asset = "dive_{version}_{os}_{arch}.{ext}",  # fallback
    strip = "dive_{version}_{os}_{arch}",
)
```

注意：对于命名规则已知且稳定的项目（如 goreleaser 出品的 Go 工具），继续使用 `github_go_provider` 仍然是最优选择。`github_smart_provider` 主要面向：
1. 命名不规则的项目（如 hugo、ffmpeg）
2. 新 Provider 的快速原型（不确定命名规则时）
3. 作为 fallback 保障（在显式模板失败时兜底）

---

## 技术实现

### 需要变更的文件

```
crates/vx-starlark/stdlib/smart_detect.star     # 新增：评分引擎
crates/vx-starlark/stdlib/provider_templates.star # 修改：新增 github_smart_provider
crates/vx-starlark/stdlib/provider.star           # 修改：re-export github_smart_provider
crates/vx-runtime/src/                               # 修改：新增 GitHub asset listing
crates/vx-runtime/src/cache/                         # 修改：ETag 缓存支持
```

### 分步实现计划

**Phase 1: 核心评分（本 RFC 实现）**

1. 实现 `smart_detect.star` 中的评分函数
2. 在 `provider_templates.star` 中添加 `github_smart_provider`
3. 在 Rust 侧实现 `list_release_assets` 原语
4. 添加测试（至少覆盖：ripgrep、hugo、fd 三种 asset 模式）
5. 添加 `--debug` 评分日志输出

**Phase 2: 缓存与优化（follow-up）**

1. ETag 条件请求缓存
2. 并发预热（install 时提前 fetch assets）
3. 评分缓存（同平台+同 tag 不重复评分）

**Phase 3: 迁移与推广（follow-up）**

1. 选择 5-10 个手写 Provider 迁移到 smart 模板
2. 文档更新（Provider 开发指南添加 smart 模板章节）
3. 对比基准测试（smart detect vs 显式模板的性能差异）

### 风险与缓解

| 风险 | 概率 | 缓解措施 |
|------|------|----------|
| 评分误匹配（选了错误平台的 asset） | 低 | 40 分阈值 + OS/arch 硬约束 + fallback 模板 |
| GitHub API rate limit 耗尽 | 中 | ETag 缓存 + fallback to 显式模板 |
| 新命名模式不被现有规则覆盖 | 中 | 允许 Provider 覆盖评分函数 + extra_excludes 参数 |
| 性能回归（多加一次 API 调用） | 低 | 缓存 + 预取 + install 的下载时间远大于 API 调用时间 |
| GitHub API 返回错误（403/429/timeout） | 中 | 缓存优先；缓存未命中时 fallback 到显式模板；超时重试 1 次 |
| Asset 命名不含版本号 | 低 | 跳过版本号检查降级为仅 OS/arch 匹配；通过 `--debug` 输出警告 |

---

## 未决问题

1. **`list_release_assets` 是否需要支持 Gitee 等非 GitHub 源？**
   建议 Phase 1 仅支持 GitHub，后续版本按需扩展。

2. **评分函数是否需要考虑 asset 下载量？**
   eget 不考量；下载量高可能只是 Windows 用户多。建议先不引入此维度。

3. **是否允许 Provider 完全自定义评分函数？**
   Provider 可以定义 `_custom_score(name, ctx, version) → int | None`，若存在则替代默认评分。但 Phase 1 暂不暴露此扩展点。

4. **Asset 文件名不含版本号时如何处理？**
   部分项目（如某些 nightly/stable 单通道工具）的 asset 命名不含版本号（如 `tool-linux-amd64.tar.gz`）。当前要求版本号必须出现，此类项目需使用 `asset` 显式模板。是否应支持 `version_check = False` 选项？

5. **Windows 上是否需要区分 MSVC / MinGW / static 链接？**
   当前 libc 维度在非 Linux 平台固定满分，但 Windows 上 MSVC 运行时版本差异可能导致二进制不兼容。Phase 1 暂不处理。

---

## 参考

- [eget — Easy pre-built binary installation](https://github.com/zyedidia/eget) — Detect 阶段的设计参考
- [mise — dev tools, env vars, task runner](https://github.com/jdx/mise) — 版本缓存策略参考
- [RFC 0036: Starlark Provider Support](0036-starlark-provider-support.md) — Starlark DSL 架构
- [RFC 0037: Provider Star Unified Facade](0037-provider-star-unified-facade.md) — ProviderHandle 统一门面
- [RFC 0038: provider.star — 简洁优先的统一 Provider 格式](0038-provider-star-replaces-toml.md) — Provider 格式统一
- [RFC 0039: Star-Only Providers and Dynamic Registry](0039-star-only-providers-and-dynamic-registry.md) — 动态注册架构
- [GitHub Releases API](https://docs.github.com/en/rest/releases/releases) — Release assets 接口
