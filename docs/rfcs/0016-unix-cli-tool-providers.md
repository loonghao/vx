# RFC 0016: Unix CLI Tool Providers - 遵循 Unix Philosophy 的工具集成

> **状态**: Draft
> **作者**: VX Team
> **创建日期**: 2026-01-07
> **目标版本**: v0.3.0
> **关联**: RFC-0015 (System Tool Discovery), RFC-0014 (Platform-Aware Providers)

## 摘要

为 vx 添加一系列遵循 **Unix Philosophy** 和 **Scriptability** 原则的命令行工具 Provider。这些工具是现代开发工作流的基石,涵盖文本处理、数据转换、网络通信、媒体处理等领域。

设计目标:
- **统一版本管理**: 像管理 Node.js/Python 一样管理 grep、sed、jq、curl、ffmpeg 等工具
- **跨平台一致性**: 在 Windows/macOS/Linux 上提供一致的工具版本和行为
- **现代替代品**: 同时支持经典 Unix 工具和其 Rust/Go 编写的现代替代品
- **AI/脚本友好**: 所有工具通过 `vx <cmd>` 直接调用,保持 Unix 习惯

---

## AI 工具使用分析 (Claude Code 视角)

### Claude Code 的工具使用层次

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Claude Code 工具优先级                           │
├─────────────────────────────────────────────────────────────────────┤
│ 第一层: 内置专用工具 (最优先)                                        │
│   Read       → 代替 cat, head, tail                                 │
│   Edit       → 代替 sed, awk (单文件编辑)                           │
│   Write      → 代替 echo >, cat <<EOF                               │
│   Grep       → 代替 grep, rg (内容搜索)                             │
│   Glob       → 代替 find, fd (文件查找)                             │
│   WebFetch   → 代替 curl, wget (HTTP 请求)                          │
├─────────────────────────────────────────────────────────────────────┤
│ 第二层: vx 管理的工具 (本 RFC 目标)                                  │
│   vx jq      → JSON 处理 (无内置替代)                               │
│   vx yq      → YAML 处理                                            │
│   vx ffmpeg  → 媒体处理                                             │
│   vx pandoc  → 文档转换                                             │
│   vx git     → 版本控制 (可能有内置,但 git 操作通常用 Bash)         │
├─────────────────────────────────────────────────────────────────────┤
│ 第三层: 系统工具 (后备)                                              │
│   系统已安装的工具,通过 PATH 发现                                    │
└─────────────────────────────────────────────────────────────────────┘
```

### AI 最需要的 Unix 工具 (按需求优先级)

| 优先级 | 工具 | 场景 | AI 使用方式 |
|--------|------|------|-------------|
| **P0 必备** | jq | JSON 处理 | `vx jq '.data[]' response.json` |
| **P0 必备** | yq | YAML/TOML 处理 | `vx yq '.services' docker-compose.yml` |
| **P0 必备** | git | 版本控制 | `vx git status`, `vx git diff` |
| **P1 高频** | rg (ripgrep) | 代码搜索 | 可用内置 Grep,但 rg 更快 |
| **P1 高频** | fd | 文件查找 | 可用内置 Glob,但 fd 更灵活 |
| **P1 高频** | bat | 语法高亮查看 | 调试/展示时有用 |
| **P2 中频** | ffmpeg | 媒体处理 | 用户请求时 |
| **P2 中频** | pandoc | 文档转换 | 用户请求时 |
| **P2 中频** | imagemagick | 图片处理 | 用户请求时 |
| **P3 低频** | sqlite3/duckdb | 数据库操作 | 数据分析任务 |
| **P3 低频** | curl/xh | HTTP 调试 | 已有 WebFetch,但原生更灵活 |

### 设计决策: `vx <cmd>` vs Bundle

**结论: 直接执行 `vx <cmd>` 是最佳方案**

```bash
# ✅ 推荐方式 (Unix Philosophy)
vx jq '.data' file.json
vx ffmpeg -i input.mp4 output.mp3
vx rg "pattern" --type rust

# ❌ 不推荐 (过度抽象)
vx bundle exec data-processing -- jq '.data' file.json
vx run media:ffmpeg -i input.mp4 output.mp3
```

**原因:**

| 因素 | `vx <cmd>` | Bundle 执行 |
|------|------------|-------------|
| Unix 哲学兼容 | ✅ 完全符合 | ❌ 增加抽象层 |
| AI 学习成本 | ✅ 零 (已知工具语法) | ❌ 需学习新语法 |
| 脚本可移植性 | ✅ 替换 `cmd` 为 `vx cmd` | ❌ 需重写 |
| 命令补全 | ✅ 工具原生补全 | ❌ 需自定义 |
| 错误信息 | ✅ 工具原生错误 | ⚠️ 可能被包装 |

**Bundle 的正确定位:**

```bash
# Bundle 用于: 批量安装
$ vx bundle install modern-unix      # 一次安装 12 个工具
$ vx bundle install data-processing  # 一次安装 jq, yq, xq 等

# 执行时: 直接调用工具
$ vx rg "TODO" .
$ vx jq '.items[]' data.json
$ vx ffmpeg -i video.mp4 audio.mp3
```

## 主流方案调研

### 1. mise-en-place (jdx/mise)

**架构**: Polyglot 工具版本管理器,asdf 的 Rust 重写

**核心设计**:
```bash
# mise 支持 500+ 工具,通过插件系统
mise use jq@1.7
mise use ripgrep@14
mise exec -- jq '.foo' data.json
```

**关键特性**:
- 基于 asdf 插件生态 (兼容 asdf 插件)
- 支持 shim 和 PATH 两种模式
- `.mise.toml` 或 `.tool-versions` 配置
- 自动检测项目目录并激活

**依赖库**: Rust 实现,无运行时依赖

### 2. asdf-vm (asdf-vm/asdf)

**架构**: 可扩展版本管理器,shell 脚本实现

**核心设计**:
```bash
# asdf 通过插件支持各种工具
asdf plugin add jq https://github.com/lsegal/asdf-jq.git
asdf install jq 1.7
asdf global jq 1.7
```

**关键特性**:
- 社区维护的插件生态 (400+ 插件)
- 简单的 `.tool-versions` 文件格式
- Shell 脚本实现,跨平台但较慢

**缺点**: Shell 脚本性能差,Windows 支持有限

### 3. uutils/coreutils (uutils/coreutils)

**架构**: GNU coreutils 的跨平台 Rust 重写

**核心设计**:
```rust
// 每个工具作为独立 crate
// coreutils/src/uu/cat/src/cat.rs
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    // ...
}
```

**关键特性**:
- 单一二进制或独立工具模式
- 100% Rust,跨平台兼容
- 与 GNU 工具高度兼容

**依赖库**: 
- `clap` - 命令行解析
- `uucore` - 共享核心功能

### 4. Modern Unix Tools 项目

基于 [maintained-modern-unix](https://github.com/johnalanwoods/maintained-modern-unix) 的调研:

| 经典工具 | 现代替代 | 语言 | 特点 |
|----------|----------|------|------|
| grep | **ripgrep (rg)** | Rust | 10x 速度,gitignore 集成 |
| sed | **sd** | Rust | 直观语法,正则替换 |
| awk/cut | **choose** | Rust | 人性化切分 |
| find | **fd** | Rust | 快速,友好语法 |
| cat | **bat** | Rust | 语法高亮,Git 集成 |
| ls | **eza** | Rust | 现代 ls,图标支持 |
| du | **dust** | Rust | 可视化磁盘用量 |
| diff | **delta** | Rust | Git diff 美化 |
| curl | **xh** | Rust | 现代 HTTP 客户端 |
| top | **bottom (btm)** | Rust | 跨平台系统监控 |
| ps | **procs** | Rust | 现代进程查看 |
| man | **tldr** | 多语言 | 简化手册页 |

### 方案对比

| 特性 | mise | asdf | vx (本 RFC) |
|------|------|------|-------------|
| 实现语言 | Rust | Shell | Rust |
| Windows 支持 | ✓ 良好 | ✗ 差 | ✓ 原生 |
| 插件模式 | asdf 兼容 | 插件 | 内置 Provider |
| 工具发现 | 手动 | 手动 | 自动 |
| 现代工具 | ✓ | ✓ | ✓ 优先 |
| AI 集成 | ✗ | ✗ | ✓ MCP |

### 设计启示

基于以上调研,本 RFC 应采用:

1. **内置 Provider 优于插件** - 减少依赖,提高可靠性
2. **优先现代工具** - ripgrep > grep, fd > find
3. **经典工具作为后备** - 兼容现有脚本
4. **统一 URL Builder 模式** - 参考 Node/Go provider 的实现

---

## AI 推荐 Provider 清单 (Claude Code 视角)

基于 Claude Code 的实际使用场景,以下是**最推荐优先实现的 Provider**:

### Tier 1: 必备工具 (首批实现)

这些工具填补了 Claude Code 内置工具无法覆盖的场景:

| 工具 | 用途 | 为什么必备 | 示例 |
|------|------|-----------|------|
| **jq** | JSON 处理 | 无内置替代,数据处理核心 | `vx jq '.data[]' api.json` |
| **yq** | YAML 处理 | 配置文件处理必备 | `vx yq '.services' compose.yml` |
| **ripgrep (rg)** | 代码搜索 | 比内置 Grep 更快更强 | `vx rg "TODO" --type rust` |
| **fd** | 文件查找 | 比内置 Glob 更灵活 | `vx fd "\.rs$" --exec wc -l` |
| **git** | 版本控制 | 开发工作流核心 | `vx git status` |

### Tier 2: 高频工具 (第二批实现)

这些工具在特定场景下非常有用:

| 工具 | 用途 | 场景 | 示例 |
|------|------|------|------|
| **bat** | 语法高亮 | 代码展示/调试 | `vx bat src/main.rs` |
| **delta** | Diff 美化 | Git diff 可读性 | `vx git diff \| vx delta` |
| **sd** | 文本替换 | 比 sed 更直观 | `vx sd 'foo' 'bar' file.txt` |
| **xh** | HTTP 客户端 | API 调试 | `vx xh POST api.example.com/data` |
| **hyperfine** | 性能测试 | 基准测试 | `vx hyperfine 'cmd1' 'cmd2'` |
| **fzf** | 模糊查找 | 交互式选择 | `vx fd \| vx fzf` |

### Tier 3: 专业工具 (按需实现)

用户请求时使用的专业工具:

| 工具 | 用途 | 场景 |
|------|------|------|
| **ffmpeg** | 媒体处理 | 视频/音频转换 |
| **imagemagick** | 图片处理 | 图片转换/调整 |
| **pandoc** | 文档转换 | Markdown/PDF/Word |
| **yt-dlp** | 视频下载 | 媒体获取 |
| **sqlite3** | 数据库 | 本地数据管理 |
| **duckdb** | 分析数据库 | 数据分析 |
| **shellcheck** | Shell 检查 | 脚本质量 |
| **tokei** | 代码统计 | 项目分析 |

### Tier 4: 现代替代品 (可选实现)

提升开发体验的现代工具:

| 工具 | 替代 | 优势 |
|------|------|------|
| **eza** | ls | 更好的格式化,图标 |
| **dust** | du | 可视化磁盘用量 |
| **duf** | df | 更友好的磁盘信息 |
| **procs** | ps | 现代进程查看 |
| **btm** | top | 跨平台系统监控 |
| **zoxide** | cd | 智能目录跳转 |
| **starship** | prompt | 跨 shell 提示符 |

### 实现优先级总结

```
Phase 1 (v0.3.0): jq, yq, rg, fd, git
Phase 2 (v0.3.1): bat, delta, sd, xh, hyperfine, fzf  
Phase 3 (v0.4.0): ffmpeg, imagemagick, pandoc, sqlite3
Phase 4 (v0.4.1): 现代替代品 (eza, dust, duf, procs, btm)
Phase 5 (v0.5.0): Bundle 系统 + 其他工具
```

---

## 动机

### 当前状态分析

1. **版本碎片化**: 不同机器上的 grep/sed/awk 版本不同,脚本行为不一致
2. **Windows 缺失**: Windows 缺少原生 Unix 工具,需要 Git Bash/WSL/MSYS2
3. **工具冲突**: macOS 自带的 BSD 工具与 GNU 工具语法差异大
4. **手动管理**: 开发者需要手动安装和更新这些工具

### 需求分析

1. **跨平台一致性** - 在所有平台提供相同版本的工具
2. **版本锁定** - 项目可以锁定特定版本,避免"在我机器上能跑"问题
3. **自动安装** - `vx jq .foo data.json` 自动安装 jq
4. **现代优先** - 默认使用现代工具 (ripgrep),同时支持经典工具 (grep)

---

## 设计方案

### 工具分类

#### 第一类: 文本处理 (Text Processing)

| 工具 | 经典 | 现代替代 | 用途 |
|------|------|----------|------|
| grep | GNU grep | **ripgrep (rg)** | 文本搜索 |
| sed | GNU sed | **sd** | 流编辑器 |
| awk | GNU awk (gawk) | **choose** | 文本处理 |
| diff | GNU diff | **delta** / **difftastic** | 差异比较 |
| tr | GNU tr | - | 字符转换 |
| cut | GNU cut | **choose** | 列提取 |
| sort | GNU sort | - | 排序 |
| uniq | GNU uniq | - | 去重 |
| wc | GNU wc | - | 计数 |
| head/tail | GNU head/tail | - | 头尾提取 |

#### 第二类: 数据格式处理 (Data Format)

| 工具 | 用途 | 特点 |
|------|------|------|
| **jq** | JSON 处理 | 标准 JSON 处理器 |
| **yq** | YAML 处理 | jq 语法处理 YAML |
| **xq** | XML 处理 | jq 语法处理 XML |
| **htmlq** | HTML 处理 | jq 语法处理 HTML |
| **csvq** | CSV/TSV 处理 | SQL-like CSV 查询 |
| **dasel** | 统一数据选择器 | JSON/YAML/TOML/XML |
| **fx** | 交互式 JSON 查看 | TUI JSON 浏览 |
| **gron** | JSON 转换 | 使 JSON greppable |

#### 第三类: 网络工具 (Network)

| 工具 | 经典 | 现代替代 | 用途 |
|------|------|----------|------|
| curl | curl | **xh** / **httpie** | HTTP 客户端 |
| wget | wget | **aria2** | 下载工具 |
| dig | bind-dig | **doggo** / **dog** | DNS 查询 |
| ping | ping | **gping** | 网络探测 |
| nc | netcat | - | 网络瑞士军刀 |
| ssh | OpenSSH | - | 远程连接 |

#### 第四类: 媒体处理 (Media)

| 工具 | 用途 | 特点 |
|------|------|------|
| **ffmpeg** | 视频/音频处理 | 媒体转换神器 |
| **ffprobe** | 媒体信息提取 | ffmpeg 配套工具 |
| **imagemagick** | 图片处理 | convert/identify |
| **graphicsmagick** | 图片处理 | ImageMagick 替代 |
| **sox** | 音频处理 | 命令行音频瑞士军刀 |
| **yt-dlp** | 视频下载 | youtube-dl 维护版 |
| **gifsicle** | GIF 处理 | GIF 优化/编辑 |
| **pngquant** | PNG 压缩 | PNG 有损压缩 |
| **svgo** | SVG 优化 | SVG 压缩优化 |

#### 第五类: 文档转换 (Document)

| 工具 | 用途 | 特点 |
|------|------|------|
| **pandoc** | 文档转换 | 万能文档转换器 |
| **latex** | 排版系统 | 专业文档排版 |
| **groff** | 文档格式化 | Unix 文档系统 |
| **asciidoctor** | AsciiDoc 处理 | 技术文档 |
| **mdbook** | Markdown 书籍 | Rust 文档工具 |

#### 第六类: 文件系统 (Filesystem)

| 工具 | 经典 | 现代替代 | 用途 |
|------|------|----------|------|
| find | find | **fd** | 文件查找 |
| ls | ls | **eza** / **lsd** | 目录列表 |
| cat | cat | **bat** | 文件查看 |
| tree | tree | **broot** | 目录树 |
| du | du | **dust** / **dua** | 磁盘用量 |
| df | df | **duf** | 文件系统 |
| touch | touch | - | 创建/更新时间 |
| mkdir | mkdir | - | 创建目录 |
| cp/mv/rm | coreutils | - | 文件操作 |
| tar | tar | - | 归档 |
| gzip/bzip2/xz | 各自工具 | **zstd** / **lz4** | 压缩 |
| zip/unzip | zip | **7z** | ZIP 压缩 |

#### 第七类: 系统监控 (System Monitor)

| 工具 | 经典 | 现代替代 | 用途 |
|------|------|----------|------|
| top | top | **btm** / **htop** / **btop** | 进程监控 |
| ps | ps | **procs** | 进程列表 |
| lsof | lsof | - | 打开文件 |
| strace | strace | - | 系统调用跟踪 |
| time | time | **hyperfine** | 性能测量 |
| watch | watch | **viddy** | 周期执行 |

#### 第八类: 终端增强 (Terminal Enhancement)

| 工具 | 用途 | 特点 |
|------|------|------|
| **tmux** | 终端复用 | 会话管理 |
| **screen** | 终端复用 | 经典方案 |
| **zellij** | 终端复用 | 现代 tmux 替代 |
| **fzf** | 模糊查找 | 交互式选择器 |
| **starship** | Shell 提示符 | 跨 shell 美化 |
| **zoxide** | 智能 cd | 记忆常用目录 |

#### 第九类: 开发工具 (Development)

| 工具 | 用途 | 特点 |
|------|------|------|
| **shellcheck** | Shell 脚本检查 | 静态分析 |
| **shfmt** | Shell 格式化 | 格式化 bash/sh |
| **tokei** | 代码统计 | 代码行数统计 |
| **cloc** | 代码统计 | 经典代码统计 |
| **scc** | 代码统计 | 快速代码统计 |
| **gitleaks** | 密钥检测 | Git 仓库密钥扫描 |
| **trufflehog** | 密钥检测 | 凭证扫描 |

#### 第十类: 数据库 CLI (Database)

| 工具 | 用途 | 特点 |
|------|------|------|
| **sqlite3** | SQLite CLI | 嵌入式数据库 |
| **duckdb** | DuckDB CLI | 分析型数据库 |
| **usql** | 通用 SQL CLI | 多数据库客户端 |
| **pgcli** | PostgreSQL CLI | 增强 psql |
| **mycli** | MySQL CLI | 增强 mysql |
| **litecli** | SQLite CLI | 增强 sqlite3 |

---

### Provider 实现架构

#### 目录结构

```
crates/vx-providers/
├── text/                    # 文本处理
│   ├── grep/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── provider.rs
│   │       └── config.rs    # GNU grep + ripgrep
│   ├── sed/
│   ├── awk/
│   └── jq/
├── network/                 # 网络工具
│   ├── curl/               # 已存在
│   ├── wget/
│   └── httpie/
├── media/                   # 媒体处理
│   ├── ffmpeg/
│   ├── imagemagick/
│   └── sox/
├── filesystem/              # 文件系统
│   ├── fd/
│   ├── bat/
│   └── eza/
├── system/                  # 系统监控
│   ├── htop/
│   ├── btm/
│   └── procs/
└── terminal/                # 终端工具
    ├── tmux/
    ├── fzf/
    └── starship/
```

#### 典型 Provider 实现 (以 jq 为例)

```rust
// crates/vx-providers/text/jq/src/provider.rs

use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

pub struct JqProvider;

impl Provider for JqProvider {
    fn name(&self) -> &str {
        "jq"
    }

    fn description(&self) -> &str {
        "Lightweight and flexible command-line JSON processor"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(JqRuntime)]
    }
}

pub fn create_provider() -> Box<dyn Provider> {
    Box::new(JqProvider)
}
```

```rust
// crates/vx-providers/text/jq/src/runtime.rs

use async_trait::async_trait;
use vx_core::{Platform, Version};
use vx_runtime::{Runtime, RuntimeContext, VersionInfo, InstallResult};

pub struct JqRuntime;

#[async_trait]
impl Runtime for JqRuntime {
    fn name(&self) -> &str {
        "jq"
    }

    fn description(&self) -> &str {
        "Command-line JSON processor"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::CLI
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        vec![
            Platform::new("linux", "x86_64"),
            Platform::new("linux", "aarch64"),
            Platform::new("darwin", "x86_64"),
            Platform::new("darwin", "aarch64"),
            Platform::new("windows", "x86_64"),
        ]
    }

    fn executable_name(&self) -> &str {
        "jq"
    }

    fn executable_extensions(&self) -> &[&str] {
        #[cfg(windows)]
        return &["exe"];
        #[cfg(not(windows))]
        return &[""];
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // 从 GitHub Releases 获取版本列表
        let releases = ctx.github_client()
            .get_releases("jqlang", "jq")
            .await?;
        
        releases.iter()
            .filter_map(|r| {
                let version = r.tag_name.strip_prefix("jq-")?;
                Some(VersionInfo {
                    version: version.to_string(),
                    lts: false,
                    stable: !r.prerelease,
                    release_date: Some(r.published_at.clone()),
                })
            })
            .collect()
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        let url = JqUrlBuilder::new(version, platform).build()?;
        Ok(Some(url))
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        // jq 是单一二进制,下载后直接使用
        let platform = ctx.platform();
        let url = self.download_url(version, &platform).await?.unwrap();
        
        let binary = ctx.download_binary(&url).await?;
        let install_dir = ctx.install_dir(self.name(), version);
        let bin_path = install_dir.join("bin").join(self.executable_name());
        
        std::fs::create_dir_all(bin_path.parent().unwrap())?;
        std::fs::write(&bin_path, binary)?;
        
        #[cfg(unix)]
        std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755))?;
        
        Ok(InstallResult {
            installed_version: version.to_string(),
            install_dir,
            executables: vec![self.executable_name().to_string()],
        })
    }
}
```

```rust
// crates/vx-providers/text/jq/src/config.rs

use vx_core::Platform;

pub struct JqUrlBuilder {
    version: String,
    platform: Platform,
}

impl JqUrlBuilder {
    pub fn new(version: &str, platform: &Platform) -> Self {
        Self {
            version: version.to_string(),
            platform: platform.clone(),
        }
    }

    pub fn build(&self) -> Result<String, Error> {
        // jq 1.7+ 使用新的下载 URL 格式
        // https://github.com/jqlang/jq/releases/download/jq-1.7/jq-linux-amd64
        
        let os = match self.platform.os.as_str() {
            "linux" => "linux",
            "darwin" => "macos",
            "windows" => "windows",
            _ => return Err(Error::UnsupportedPlatform),
        };
        
        let arch = match self.platform.arch.as_str() {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            _ => return Err(Error::UnsupportedPlatform),
        };
        
        let ext = if self.platform.os == "windows" { ".exe" } else { "" };
        
        Ok(format!(
            "https://github.com/jqlang/jq/releases/download/jq-{}/jq-{}-{}{}",
            self.version, os, arch, ext
        ))
    }
}
```

#### 复杂工具 Provider (以 ffmpeg 为例)

```rust
// crates/vx-providers/media/ffmpeg/src/provider.rs

pub struct FfmpegProvider;

impl Provider for FfmpegProvider {
    fn name(&self) -> &str {
        "ffmpeg"
    }

    fn description(&self) -> &str {
        "Complete solution for recording, converting and streaming audio and video"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(FfmpegRuntime),
            Arc::new(FfprobeRuntime),  // ffprobe 作为附属工具
            Arc::new(FfplayRuntime),   // ffplay 作为附属工具
        ]
    }
}
```

```rust
// crates/vx-providers/media/ffmpeg/src/config.rs

/// ffmpeg 下载源配置
/// 
/// ffmpeg 官方不提供预编译二进制,需要使用第三方构建:
/// - Windows: https://www.gyan.dev/ffmpeg/builds/
/// - macOS: https://evermeet.cx/ffmpeg/
/// - Linux: https://johnvansickle.com/ffmpeg/
pub struct FfmpegUrlBuilder {
    version: String,
    platform: Platform,
    build_type: FfmpegBuild,
}

#[derive(Clone, Copy)]
pub enum FfmpegBuild {
    /// 完整版,包含所有编解码器
    Full,
    /// 精简版,常用编解码器
    Essentials,
    /// GPL 版本 (包含 x264, x265)
    Gpl,
    /// LGPL 版本 (无 GPL 编解码器)
    Lgpl,
}

impl FfmpegUrlBuilder {
    pub fn build(&self) -> Result<String, Error> {
        match self.platform.os.as_str() {
            "windows" => self.windows_url(),
            "darwin" => self.macos_url(),
            "linux" => self.linux_url(),
            _ => Err(Error::UnsupportedPlatform),
        }
    }

    fn windows_url(&self) -> Result<String, Error> {
        // gyan.dev 提供 Windows 构建
        // https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip
        let build = match self.build_type {
            FfmpegBuild::Full => "full",
            FfmpegBuild::Essentials => "essentials",
            _ => "essentials",
        };
        Ok(format!(
            "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-{}.zip",
            build
        ))
    }

    fn macos_url(&self) -> Result<String, Error> {
        // evermeet.cx 提供 macOS 构建
        // https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip
        Ok("https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip".to_string())
    }

    fn linux_url(&self) -> Result<String, Error> {
        // johnvansickle.com 提供 Linux 静态构建
        let arch = match self.platform.arch.as_str() {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            "armv7" => "armhf",
            _ => return Err(Error::UnsupportedPlatform),
        };
        Ok(format!(
            "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-{}-static.tar.xz",
            arch
        ))
    }
}
```

---

### 现代工具优先策略

vx 采用 **现代工具优先** 策略:

```toml
# vx.toml 配置示例

[tools]
# 使用现代工具 (推荐)
rg = "14"        # ripgrep
fd = "10"        # fd-find
bat = "0.24"     # bat
jq = "1.7"       # jq (已经很现代)
ffmpeg = "7"     # ffmpeg

# 或者使用经典工具 (兼容模式)
grep = "3.11"    # GNU grep
sed = "4.9"      # GNU sed
awk = "5.3"      # GNU awk
```

#### 别名映射

```toml
# ~/.vx/config.toml

[aliases]
# 现代工具别名
grep = "rg"          # vx grep -> vx rg
find = "fd"          # vx find -> vx fd
cat = "bat"          # vx cat -> vx bat

# 禁用别名 (使用原生工具)
# grep = "grep"
```

用户可以配置偏好:

```bash
# 使用现代工具
$ vx grep pattern file.txt
# 实际执行: rg pattern file.txt

# 强制使用经典工具
$ vx --classic grep pattern file.txt
# 实际执行: grep pattern file.txt

# 或者直接指定
$ vx rg pattern file.txt
```

---

### 工具包 (Bundles)

提供预定义的工具包,一次安装多个相关工具:

```toml
# vx.toml

[bundles]
# 使用预定义包
use = ["modern-unix", "data-processing"]
```

预定义工具包:

#### modern-unix

```toml
[bundles.modern-unix]
description = "Modern replacements for classic Unix tools"
tools = [
    "ripgrep",      # grep replacement
    "fd",           # find replacement
    "bat",          # cat replacement
    "eza",          # ls replacement
    "dust",         # du replacement
    "duf",          # df replacement
    "procs",        # ps replacement
    "sd",           # sed replacement
    "choose",       # awk/cut replacement
    "delta",        # diff replacement
    "zoxide",       # cd enhancement
    "fzf",          # fuzzy finder
]
```

#### data-processing

```toml
[bundles.data-processing]
description = "Data format processing tools"
tools = [
    "jq",           # JSON
    "yq",           # YAML
    "xq",           # XML
    "htmlq",        # HTML
    "csvq",         # CSV
    "dasel",        # Universal selector
    "gron",         # Make JSON greppable
]
```

#### media-toolkit

```toml
[bundles.media-toolkit]
description = "Media processing tools"
tools = [
    "ffmpeg",       # Video/audio processing
    "imagemagick",  # Image processing
    "yt-dlp",       # Video download
    "gifsicle",     # GIF processing
    "pngquant",     # PNG compression
    "svgo",         # SVG optimization
]
```

#### devops-essentials

```toml
[bundles.devops-essentials]
description = "DevOps and scripting tools"
tools = [
    "jq",
    "yq",
    "shellcheck",
    "shfmt",
    "hyperfine",
    "tokei",
    "gitleaks",
]
```

命令行使用:

```bash
# 安装工具包
$ vx bundle install modern-unix
Installing modern-unix bundle (12 tools)...
  ✓ ripgrep@14.0.0
  ✓ fd@10.0.0
  ✓ bat@0.24.0
  ...
Done in 5.2s

# 列出可用包
$ vx bundle list
Available bundles:
  modern-unix        12 tools    Modern Unix tool replacements
  data-processing     7 tools    Data format processing
  media-toolkit       6 tools    Media processing tools
  devops-essentials   7 tools    DevOps and scripting

# 查看包内容
$ vx bundle show modern-unix
modern-unix - Modern replacements for classic Unix tools

Tools:
  ripgrep (rg)    ✓ installed    v14.0.0
  fd              ✓ installed    v10.0.0
  bat             ✗ not installed
  ...
```

---

### 平台兼容性矩阵

| 工具 | Windows | macOS | Linux | 备注 |
|------|---------|-------|-------|------|
| **文本处理** |
| ripgrep | ✓ | ✓ | ✓ | 原生二进制 |
| sd | ✓ | ✓ | ✓ | 原生二进制 |
| jq | ✓ | ✓ | ✓ | 原生二进制 |
| GNU grep | ⚠️ MSYS2 | ⚠️ Homebrew | ✓ | 需要额外依赖 |
| GNU sed | ⚠️ MSYS2 | ⚠️ Homebrew | ✓ | 需要额外依赖 |
| **网络** |
| curl | ✓ | ✓ | ✓ | 系统自带或下载 |
| xh | ✓ | ✓ | ✓ | 原生二进制 |
| wget | ⚠️ | ✓ | ✓ | Windows 需下载 |
| **媒体** |
| ffmpeg | ✓ | ✓ | ✓ | 第三方构建 |
| imagemagick | ⚠️ | ✓ | ✓ | Windows 安装复杂 |
| **终端** |
| tmux | ⚠️ WSL | ✓ | ✓ | Windows 需 WSL |
| fzf | ✓ | ✓ | ✓ | 原生二进制 |

**图例:**
- ✓ 完全支持
- ⚠️ 需要额外配置或有限制
- ✗ 不支持

---

## 实现计划

### Phase 1: 核心文本工具 (v0.3.0)

- [ ] jq Provider (JSON 处理)
- [ ] yq Provider (YAML 处理)  
- [ ] ripgrep Provider (grep 替代)
- [ ] sd Provider (sed 替代)
- [ ] fd Provider (find 替代)

### Phase 2: 扩展工具 (v0.3.1)

- [ ] bat Provider (cat 替代)
- [ ] eza Provider (ls 替代)
- [ ] dust Provider (du 替代)
- [ ] delta Provider (diff 替代)
- [ ] fzf Provider

### Phase 3: 媒体工具 (v0.4.0)

- [ ] ffmpeg Provider (包含 ffprobe, ffplay)
- [ ] imagemagick Provider
- [ ] yt-dlp Provider
- [ ] pandoc Provider

### Phase 4: 经典工具支持 (v0.4.1)

- [ ] GNU grep Provider
- [ ] GNU sed Provider
- [ ] GNU awk Provider
- [ ] GNU coreutils Provider (打包)

### Phase 5: 工具包 (v0.5.0)

- [ ] Bundle 系统实现
- [ ] modern-unix bundle
- [ ] data-processing bundle
- [ ] media-toolkit bundle

---

## 向后兼容性

### 与现有 Provider 的关系

- 新 Provider 与现有 Provider (node, go, rust 等) 并行存在
- 不影响现有功能

### 经典工具兼容

```toml
# 用户可以选择使用经典工具
[settings]
prefer_classic = true  # 禁用现代工具别名
```

### 系统工具优先

```toml
# 优先使用系统已安装的工具
[settings]
system_first = true
```

---

## 替代方案

### 方案 A: 集成 asdf 插件

**优点**: 利用现有 400+ 插件生态
**缺点**: Shell 脚本性能差,Windows 支持差

### 方案 B: 集成 mise 后端

**优点**: Rust 实现,性能好
**缺点**: 引入外部依赖,架构复杂

### 方案 C: 内置 Provider (本方案)

**优点**: 完全控制,最优性能,原生 Windows
**缺点**: 需要逐个实现 Provider

**决定**: 采用方案 C,因为:
1. 与 vx 现有架构一致
2. 可以提供最优的用户体验
3. 可以逐步实现,按需添加

---

## 参考资料

### 主流项目源码

- [jqlang/jq](https://github.com/jqlang/jq) - JSON 处理器
- [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep) - 现代 grep
- [sharkdp/fd](https://github.com/sharkdp/fd) - 现代 find
- [sharkdp/bat](https://github.com/sharkdp/bat) - 现代 cat
- [eza-community/eza](https://github.com/eza-community/eza) - 现代 ls
- [FFmpeg/FFmpeg](https://github.com/FFmpeg/FFmpeg) - 媒体处理
- [uutils/coreutils](https://github.com/uutils/coreutils) - Rust coreutils

### 参考项目

- [mise-en-place](https://mise.jdx.dev/) - Polyglot 版本管理器
- [asdf-vm](https://asdf-vm.com/) - 可扩展版本管理器
- [maintained-modern-unix](https://github.com/johnalanwoods/maintained-modern-unix) - 现代 Unix 工具列表

### 相关 RFC

- RFC-0015: System Tool Discovery
- RFC-0014: Platform-Aware Providers

---

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-01-07 | Draft | 初始草案,完成主流方案调研和工具分类 |
