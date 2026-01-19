# RFC 0021: System Package Manager Integration & Manifest-Driven Runtimes

> **状态**: Implemented (Phase 1-4 Complete)
> **作者**: vx team
> **创建日期**: 2026-01-13
> **目标版本**: v0.5.0

## 摘要

本 RFC 提出两个相互关联的增强：

1. **系统包管理器集成** - 使 vx 能够自动检测、安装和管理系统级工具及其依赖（VCRedist、KB 更新等）
2. **Manifest-Driven Runtimes** - 允许通过纯 `provider.toml` 配置文件定义系统工具，无需编写 Rust 代码

这两个特性结合后，用户可以：
- 执行 `vx curl` 自动处理所有依赖和安装
- 在本地 `~/.vx/providers/mytools/provider.toml` 中定义自己的工具
- 大幅降低系统工具的维护成本

## 主流方案调研

### 1. Chocolatey (chocolatey/choco)

**架构**: 基于 NuGet 的 Windows 包管理器，使用 `.nuspec` 定义包元数据和依赖。

**依赖定义格式**:
```xml
<dependencies>
  <dependency id="vcredist140" version="14.0" />
  <dependency id="dotnetfx" version="4.8" />
  <dependency id="kb2919355" />
</dependencies>
```

**关键特性**:
- 支持 `--params` 传递安装参数：`choco install git --params "'/GitAndUnixToolsOnPath'"`
- 支持 `--install-arguments` 传递原生安装器参数
- 依赖链自动解析和安装
- 支持 KB 更新作为依赖（如 `kb2919355`）

**系统依赖处理**:
- VCRedist: `choco install vcredist140`
- .NET Framework: `choco install dotnetfx`
- KB 更新: `choco install kb2919355`

### 2. winget (microsoft/winget-cli)

**架构**: Microsoft 官方包管理器，使用 YAML 清单定义包。

**依赖定义格式**:
```yaml
Dependencies:
  WindowsFeatures:
    - NetFx4
  WindowsLibraries:
    - Microsoft.VCLibs.140.00
  PackageDependencies:
    - PackageIdentifier: Microsoft.VCRedist.2015+.x64
      MinimumVersion: 14.0.24215.1
  ExternalDependencies:
    - python
```

**关键特性**:
- 区分 Windows 功能、库、包依赖和外部依赖
- 支持最低版本约束
- 自动解析依赖顺序

### 3. Scoop (ScoopInstaller/Scoop)

**架构**: 轻量级 Windows 包管理器，使用 JSON 清单。

**依赖定义格式**:
```json
{
    "depends": ["git", "7zip"],
    "suggest": {
        "vcredist": "extras/vcredist2022"
    }
}
```

**关键特性**:
- `depends`: 硬依赖，必须安装
- `suggest`: 软依赖，建议安装
- 便携式安装，不污染系统

### 4. Homebrew (Homebrew/brew)

**架构**: macOS/Linux 包管理器，使用 Ruby DSL 定义 Formula。

**依赖定义格式**:
```ruby
class Example < Formula
  depends_on "openssl@3"
  depends_on "python@3.11" => :build
  depends_on "xcode" => :build if OS.mac?
  
  uses_from_macos "zlib"
end
```

**关键特性**:
- 区分运行时依赖和构建时依赖
- 支持平台条件依赖
- `uses_from_macos` 复用系统库

### 5. APT (Debian/Ubuntu)

**架构**: Debian 系包管理器，使用 `control` 文件定义依赖。

**依赖定义格式**:
```
Depends: libc6 (>= 2.17), libssl3 (>= 3.0.0)
Pre-Depends: dpkg (>= 1.14.0)
Recommends: ca-certificates
Suggests: openssl
```

**关键特性**:
- `Pre-Depends`: 安装前必须满足的依赖
- `Depends`: 运行时依赖
- `Recommends`: 推荐安装
- `Suggests`: 可选增强

### 方案对比

| 特性 | Chocolatey | winget | Scoop | Homebrew | APT |
|------|------------|--------|-------|----------|-----|
| 系统级依赖 | ✓ KB/VCRedist | ✓ WindowsFeatures | ✗ | ✓ uses_from_macos | ✓ Pre-Depends |
| 版本约束 | ✓ | ✓ MinimumVersion | ✗ | ✓ | ✓ |
| 条件依赖 | ✗ | ✗ | ✗ | ✓ | ✓ |
| 安装参数 | ✓ --params | ✗ | ✗ | ✗ | ✗ |
| 软依赖 | ✗ | ✓ External | ✓ suggest | ✓ recommends | ✓ Recommends |

### 设计启示

基于以上调研，本 RFC 应采用：

1. **分层依赖模型** - 采用 APT 的 `Pre-Depends`/`Depends`/`Recommends` 分层思想
2. **系统依赖类型** - 采用 winget 的 `WindowsFeatures`/`WindowsLibraries`/`PackageDependencies` 分类
3. **安装参数支持** - 采用 Chocolatey 的 `--params` 和 `--install-arguments` 机制
4. **条件依赖** - 采用 Homebrew 的平台条件依赖语法
5. **软依赖** - 采用 Scoop 的 `suggest` 机制

## 动机

### 当前状态分析

1. **系统工具安装困难**: 用户需要手动安装 curl、git 等系统工具
2. **依赖链断裂**: 某些工具需要 VCRedist、.NET Framework 等运行时
3. **包管理器缺失**: 新系统可能没有安装 Chocolatey/Homebrew
4. **KB 更新依赖**: 某些软件需要特定 Windows 更新才能运行
5. **维护成本高**: 每个系统工具都需要编写 Rust 代码实现 Runtime trait

### Provider 分类分析

通过分析现有 37 个 providers，我们发现可以分为两类：

#### 需要 Rust 代码的 Providers（版本敏感型）

这些工具需要严格版本控制或有复杂逻辑：

| Provider | 保留理由 |
|----------|----------|
| `node` | 版本解析复杂，npm/npx 捆绑逻辑，pre_run hooks |
| `python` | 版本管理复杂，虚拟环境集成 |
| `go` | pre_run hooks (go mod download) |
| `rust` | rustup 集成复杂 |
| `uv` | Python 生态核心，版本重要 |
| `bun/deno` | 版本重要，Node.js 替代品 |
| `java` | JDK 版本管理复杂 |
| `pnpm/yarn` | Node.js 生态，pre_run hooks |
| `ffmpeg` | 复杂的多组件 |
| `awscli/azcli/gcloud` | 云 CLI 有复杂认证逻辑 |
| `msvc` | Windows 特殊处理 |
| `rez/spack` | 包管理器本身 |
| `cmake` | 构建系统核心，版本敏感（CMake 3.x vs 2.x 语法差异大） |
| `ninja` | 构建系统，版本敏感 |
| `protoc` | Protocol Buffers 版本敏感 |

#### 可简化为纯配置的 Providers（版本不敏感型）

这些系统工具对版本要求不高，可以通过 `vx-system-pm` 管理：

| Provider | 简化理由 | 当前代码行数 |
|----------|----------|-------------|
| `curl` | 已是纯 toml | 0 |
| `git` | 版本要求低 | ~100 |
| `just` | 任务运行器 | ~90 |
| `helm` | K8s 工具 | ~70 |
| `kubectl` | K8s 工具 | ~80 |
| `terraform` | IaC 工具 | ~100 |
| `task` | 任务运行器 | ~90 |
| `ollama` | AI 工具 | ~100 |
| `rcedit` | Windows 工具 | ~130 |
| `nasm` | 汇编器 | ~140 |
| `docker` | 容器运行时 | ~100 |

#### 新增 Unix 哲学工具（Manifest-Driven）

这些是 AI 和开发者常用的 Unix 风格工具，通过 `provider.toml` 配置：

| 工具 | 描述 | 用途 |
|------|------|------|
| **文本处理** | | |
| `jq` | JSON 处理器 | `vx jq '.key' file.json` |
| `yq` | YAML 处理器 | `vx yq '.key' file.yaml` |
| `xq` | XML 处理器 | `vx xq '//element' file.xml` |
| `sed` | 流编辑器 | 文本替换 |
| `awk` | 文本处理 | 数据提取 |
| `grep` | 文本搜索 | 模式匹配 |
| **现代 CLI 替代品** | | |
| `rg` (ripgrep) | 快速 grep | `vx rg "pattern" .` |
| `fd` | 快速 find | `vx fd "*.rs"` |
| `bat` | 带语法高亮的 cat | `vx bat file.rs` |
| `eza` | 现代 ls | `vx eza -la` |
| `delta` | Git diff 美化 | 与 git 集成 |
| `zoxide` | 智能 cd | 快速目录跳转 |
| `fzf` | 模糊搜索 | 交互式选择 |
| `sd` | 现代 sed | `vx sd 'old' 'new' file` |
| `dust` | 磁盘使用分析 | `vx dust` |
| `duf` | 磁盘空间 | `vx duf` |
| `procs` | 现代 ps | `vx procs` |
| `bottom`/`btm` | 现代 top | `vx btm` |
| `hyperfine` | 基准测试 | `vx hyperfine 'cmd1' 'cmd2'` |
| **网络工具** | | |
| `curl` | HTTP 客户端 | `vx curl https://...` |
| `wget` | 下载工具 | `vx wget https://...` |
| `httpie`/`http` | 现代 HTTP 客户端 | `vx http GET api.example.com` |
| `xh` | 快速 HTTP 客户端 | `vx xh GET api.example.com` |
| **版本控制** | | |
| `git` | 版本控制 | `vx git clone/pull/push` |
| `gh` | GitHub CLI | `vx gh pr create` |
| `glab` | GitLab CLI | `vx glab mr create` |
| **容器/K8s** | | |
| `docker` | 容器运行时 | `vx docker run` |
| `kubectl` | K8s CLI | `vx kubectl get pods` |
| `helm` | K8s 包管理 | `vx helm install` |
| `k9s` | K8s TUI | `vx k9s` |
| **数据库工具** | | |
| `sqlite3` | SQLite CLI | `vx sqlite3 db.sqlite` |
| `pgcli` | PostgreSQL CLI | `vx pgcli` |
| `mycli` | MySQL CLI | `vx mycli` |
| **开发辅助** | | |
| `make` | 构建工具 | `vx make build` |
| `just` | 现代 make | `vx just build` |
| `task` | 任务运行器 | `vx task build` |
| `watchexec` | 文件监控 | `vx watchexec -e rs cargo build` |
| `entr` | 文件监控 | `find . -name "*.rs" \| vx entr cargo build` |
| **压缩/归档** | | |
| `tar` | 归档工具 | `vx tar -xzf file.tar.gz` |
| `zip`/`unzip` | ZIP 工具 | `vx unzip file.zip` |
| `7z` | 7-Zip | `vx 7z x file.7z` |
| `zstd` | Zstandard 压缩 | `vx zstd -d file.zst` |
| **其他** | | |
| `tree` | 目录树 | `vx tree .` |
| `tokei` | 代码统计 | `vx tokei` |
| `scc` | 代码统计 | `vx scc` |
| `tldr` | 简化 man | `vx tldr git` |
| `age` | 加密工具 | `vx age -e -r KEY file` |

**潜在节省**: ~1000 行 Rust 代码 → 纯 TOML 配置

### 需求分析

1. **透明安装**: `vx curl` 自动处理所有依赖
2. **系统依赖**: 支持 VCRedist、.NET、KB 更新等
3. **多策略**: 支持包管理器、直接下载、脚本安装
4. **跨平台**: Windows/macOS/Linux 统一体验
5. **零代码扩展**: 用户可通过配置文件添加新工具
6. **降低维护成本**: 系统工具无需编写 Rust 代码
7. **AI 友好**: 让 AI 能够完整调用各种命令行工具

## 设计方案

### 核心架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         vx CLI                                   │
├─────────────────────────────────────────────────────────────────┤
│                      vx-resolver                                 │
│  ┌──────────────────────┬──────────────────────────────────┐    │
│  │   Rust Runtimes      │   Manifest-Driven Runtimes       │    │
│  │   (node, python,     │   (git, cmake, curl, ...)        │    │
│  │    go, rust, ...)    │                                  │    │
│  └──────────────────────┴──────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────┤
│                     vx-system-pm                                 │
│  ┌────────────┬────────────┬────────────┬────────────┐          │
│  │ Chocolatey │   winget   │  Homebrew  │    APT     │  ...     │
│  └────────────┴────────────┴────────────┴────────────┘          │
├─────────────────────────────────────────────────────────────────┤
│                   Provider Sources                               │
│  ┌────────────────┬────────────────┬────────────────┐           │
│  │  Built-in      │  User Local    │  Remote        │           │
│  │  providers/    │  ~/.vx/        │  (future)      │           │
│  │                │  providers/    │                │           │
│  └────────────────┴────────────────┴────────────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

### Provider 加载优先级

```
1. ~/.vx/providers/*/provider.toml     (用户本地，最高优先级)
2. $VX_PROVIDERS_PATH/*/provider.toml  (环境变量指定)
3. Built-in providers                   (内置，最低优先级)
```

### 1. Manifest-Driven Runtime

核心思想：对于版本不敏感的系统工具，完全通过 `provider.toml` 驱动，无需 Rust 代码。

```rust
// crates/vx-runtime/src/manifest_runtime.rs

use vx_manifest::{RuntimeDef, SystemInstallConfigDef};
use anyhow::Result;
use async_trait::async_trait;

/// 由 manifest 驱动的 Runtime 实现
/// 适用于版本不敏感的系统工具
pub struct ManifestDrivenRuntime {
    /// Runtime 定义（来自 provider.toml）
    def: RuntimeDef,
    /// Provider 名称
    provider_name: String,
    /// Provider 来源路径
    source_path: PathBuf,
}

impl ManifestDrivenRuntime {
    /// 从 provider.toml 加载
    pub fn from_manifest(path: &Path) -> Result<Vec<Self>> {
        let manifest = ProviderManifest::load(path)?;
        let source_path = path.parent().unwrap().to_path_buf();
        
        manifest.runtimes
            .into_iter()
            .filter(|r| r.is_system_tool())  // 只处理系统工具
            .map(|def| Self {
                def,
                provider_name: manifest.provider.name.clone(),
                source_path: source_path.clone(),
            })
            .collect()
    }
    
    /// 判断是否为 manifest-driven runtime
    /// 条件：有 system_install 配置，且没有自定义 Rust 实现
    fn is_manifest_driven(&self) -> bool {
        self.def.system_install.is_some()
    }
}

#[async_trait]
impl Runtime for ManifestDrivenRuntime {
    fn name(&self) -> &str {
        &self.def.name
    }
    
    fn description(&self) -> &str {
        self.def.description.as_deref().unwrap_or("System tool")
    }
    
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }
    
    fn aliases(&self) -> &[&str] {
        // 从 def.aliases 转换
        &[]
    }
    
    /// Manifest-driven runtime 不需要版本管理
    /// 直接使用系统包管理器安装的版本
    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // 返回 "system" 作为唯一版本
        Ok(vec![VersionInfo {
            version: "system".to_string(),
            released_at: None,
            prerelease: false,
            lts: true,
            download_url: None,
            checksum: None,
            metadata: HashMap::new(),
        }])
    }
    
    /// 通过系统包管理器安装
    async fn install(&self, _version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let system_install = self.def.system_install.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No system_install config"))?;
        
        // 选择最佳安装策略
        let strategy = self.select_best_strategy(system_install, ctx).await?;
        
        // 执行安装
        ctx.system_pm.execute_strategy(&strategy).await
    }
    
    /// 检测系统中是否已安装
    async fn is_installed(&self, _version: &str, ctx: &RuntimeContext) -> Result<bool> {
        // 使用 which 检测
        let executable = self.def.executable.as_deref()
            .unwrap_or(&self.def.name);
        Ok(which::which(executable).is_ok())
    }
    
    /// 获取已安装版本
    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>> {
        if self.is_installed("system", ctx).await? {
            // 尝试获取版本
            if let Some(detection) = &self.def.detection {
                if let Some(version) = self.detect_version(detection).await? {
                    return Ok(vec![version]);
                }
            }
            Ok(vec!["system".to_string()])
        } else {
            Ok(vec![])
        }
    }
    
    /// 不需要下载 URL，通过包管理器安装
    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // 如果有 direct_download 策略，返回对应 URL
        if let Some(system_install) = &self.def.system_install {
            for strategy in &system_install.strategies {
                if let InstallStrategyDef::DirectDownload { url, .. } = strategy {
                    return Ok(Some(url.clone()));
                }
            }
        }
        Ok(None)
    }
    
    /// 选择最佳安装策略
    async fn select_best_strategy(
        &self,
        config: &SystemInstallConfigDef,
        ctx: &RuntimeContext,
    ) -> Result<InstallStrategyDef> {
        let platform = Platform::current();
        let mut candidates: Vec<_> = config.strategies.iter()
            .filter(|s| s.matches_platform(&platform))
            .collect();
        
        // 按优先级排序
        candidates.sort_by(|a, b| b.priority().cmp(&a.priority()));
        
        // 选择第一个可用的策略
        for strategy in candidates {
            if self.is_strategy_available(strategy, ctx).await {
                return Ok(strategy.clone());
            }
        }
        
        Err(anyhow::anyhow!("No available installation strategy for {}", self.name()))
    }
    
    async fn is_strategy_available(&self, strategy: &InstallStrategyDef, ctx: &RuntimeContext) -> bool {
        match strategy {
            InstallStrategyDef::PackageManager { manager, .. } => {
                ctx.system_pm.is_manager_available(manager).await
            }
            InstallStrategyDef::DirectDownload { .. } => true,
            InstallStrategyDef::Script { .. } => true,
            InstallStrategyDef::ProvidedBy { provider, .. } => {
                // 检查 provider 是否已安装
                ctx.is_runtime_installed(provider).await.unwrap_or(false)
            }
        }
    }
}
```

### 2. 用户自定义 Provider

用户可以在 `~/.vx/providers/` 目录下创建自己的 provider：

```bash
# 创建自定义 provider
mkdir -p ~/.vx/providers/mytools

# 创建 provider.toml
cat > ~/.vx/providers/mytools/provider.toml << 'EOF'
[provider]
name = "mytools"
description = "My custom tools"
ecosystem = "system"

# 定义 fd (find alternative)
[[runtimes]]
name = "fd"
description = "A simple, fast and user-friendly alternative to find"
executable = "fd"

[runtimes.detection]
command = "{executable} --version"
pattern = "fd ([\\d.]+)"

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "choco"
package = "fd"
priority = 80

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "brew"
package = "fd"
priority = 90

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "apt"
package = "fd-find"
priority = 90

# 定义 bat (cat alternative)
[[runtimes]]
name = "bat"
description = "A cat clone with syntax highlighting"
executable = "bat"

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "choco"
package = "bat"
priority = 80

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "brew"
package = "bat"
priority = 90

# 定义 ripgrep
[[runtimes]]
name = "rg"
description = "ripgrep - fast grep alternative"
executable = "rg"
aliases = ["ripgrep"]

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "choco"
package = "ripgrep"
priority = 80

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "brew"
package = "ripgrep"
priority = 90

[[runtimes.system_install.strategies]]
type = "direct_download"
url = "https://github.com/BurntSushi/ripgrep/releases/download/{version}/ripgrep-{version}-x86_64-pc-windows-msvc.zip"
format = "zip"
executable_path = "rg.exe"
priority = 50
EOF
```

使用：

```bash
# 自动发现并使用
vx fd --version
vx bat README.md
vx rg "pattern" .

# 列出所有可用的 runtimes（包括用户自定义）
vx list --all
```

### 3. Provider 加载器

```rust
// crates/vx-runtime/src/provider_loader.rs

use std::path::PathBuf;
use anyhow::Result;

/// Provider 来源
#[derive(Debug, Clone)]
pub enum ProviderSource {
    /// 内置 provider（编译时）
    BuiltIn,
    /// 用户本地 provider
    UserLocal(PathBuf),
    /// 环境变量指定的路径
    EnvPath(PathBuf),
    /// 远程 provider（未来支持）
    Remote(String),
}

/// Provider 加载器
pub struct ProviderLoader {
    /// 搜索路径
    search_paths: Vec<PathBuf>,
    /// 已加载的 providers
    loaded: HashMap<String, LoadedProvider>,
}

impl ProviderLoader {
    pub fn new() -> Self {
        let mut search_paths = Vec::new();
        
        // 1. 用户本地 providers（最高优先级）
        // 使用 VxPaths 统一管理路径
        if let Ok(vx_paths) = VxPaths::new() {
            search_paths.push(vx_paths.providers_dir);
        }
        
        // 2. 环境变量指定的路径
        if let Ok(paths) = std::env::var("VX_PROVIDERS_PATH") {
            for path in paths.split(if cfg!(windows) { ';' } else { ':' }) {
                search_paths.push(PathBuf::from(path));
            }
        }
        
        Self {
            search_paths,
            loaded: HashMap::new(),
        }
    }
    
    /// 发现所有 manifest-driven providers
    pub fn discover_manifest_providers(&mut self) -> Result<Vec<ManifestDrivenRuntime>> {
        let mut runtimes = Vec::new();
        
        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }
            
            for entry in std::fs::read_dir(search_path)? {
                let entry = entry?;
                let provider_toml = entry.path().join("provider.toml");
                
                if provider_toml.exists() {
                    match ManifestDrivenRuntime::from_manifest(&provider_toml) {
                        Ok(provider_runtimes) => {
                            for runtime in provider_runtimes {
                                // 检查是否与内置 provider 冲突
                                if !self.is_builtin_override(&runtime) {
                                    runtimes.push(runtime);
                                } else {
                                    tracing::debug!(
                                        "User provider {} overrides built-in",
                                        runtime.name()
                                    );
                                    runtimes.push(runtime);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to load provider from {:?}: {}",
                                provider_toml, e
                            );
                        }
                    }
                }
            }
        }
        
        Ok(runtimes)
    }
    
    /// 检查是否覆盖内置 provider
    fn is_builtin_override(&self, runtime: &ManifestDrivenRuntime) -> bool {
        // 检查内置 provider 列表
        const BUILTIN_RUNTIMES: &[&str] = &[
            "node", "npm", "npx", "python", "go", "rust", "cargo",
            "uv", "uvx", "bun", "deno", "pnpm", "yarn",
        ];
        BUILTIN_RUNTIMES.contains(&runtime.name())
    }
}
```

### 4. 依赖类型定义

```rust
// crates/vx-manifest/src/provider/system_deps.rs

use serde::{Deserialize, Serialize};

/// 系统级依赖定义
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemDependency {
    /// 依赖类型
    #[serde(rename = "type")]
    pub dep_type: SystemDepType,
    
    /// 依赖标识符
    pub id: String,
    
    /// 版本约束（可选）
    #[serde(default)]
    pub version: Option<String>,
    
    /// 依赖原因说明
    #[serde(default)]
    pub reason: Option<String>,
    
    /// 平台条件
    #[serde(default)]
    pub platforms: Vec<String>,
    
    /// 是否可选
    #[serde(default)]
    pub optional: bool,
}

/// 系统依赖类型
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SystemDepType {
    /// Windows KB 更新
    WindowsKb,
    /// Windows 功能 (DISM)
    WindowsFeature,
    /// Visual C++ Redistributable
    VcRedist,
    /// .NET Framework / .NET Runtime
    DotNet,
    /// 系统包管理器包
    Package,
    /// 其他 vx 管理的 runtime
    Runtime,
}

/// 安装策略
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallStrategy {
    /// 使用系统包管理器
    PackageManager {
        /// 包管理器名称
        manager: String,
        /// 包名
        package: String,
        /// 安装参数 (Chocolatey --params)
        #[serde(default)]
        params: Option<String>,
        /// 原生安装器参数 (Chocolatey --install-arguments)
        #[serde(default)]
        install_args: Option<String>,
        /// 优先级
        #[serde(default = "default_priority")]
        priority: i32,
    },
    /// 直接下载
    DirectDownload {
        /// URL 模板
        url: String,
        /// 归档格式
        #[serde(default)]
        format: Option<String>,
        /// 可执行文件路径
        #[serde(default)]
        executable_path: Option<String>,
        #[serde(default = "default_priority")]
        priority: i32,
    },
    /// 运行脚本
    Script {
        /// 脚本 URL
        url: String,
        /// 脚本类型
        script_type: ScriptType,
        /// 脚本参数
        #[serde(default)]
        args: Vec<String>,
        #[serde(default = "default_priority")]
        priority: i32,
    },
    /// 由其他工具提供
    ProvidedBy {
        /// 提供者 runtime
        provider: String,
        /// 相对路径
        relative_path: String,
        #[serde(default = "default_priority")]
        priority: i32,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptType {
    PowerShell,
    Bash,
    Cmd,
}

fn default_priority() -> i32 { 50 }
```

### 2. provider.toml 扩展格式

```toml
# crates/vx-providers/git/provider.toml

[provider]
name = "git"
description = "Git version control system"
ecosystem = "system"

[[runtimes]]
name = "git"
executable = "git"
aliases = ["git-scm"]

# === 系统级依赖 ===
[runtimes.system_deps]

# 前置依赖（安装前必须满足）
[[runtimes.system_deps.pre_depends]]
type = "windows_kb"
id = "kb2919355"
platforms = ["windows"]
reason = "Required for Windows 8.1 systems"

[[runtimes.system_deps.pre_depends]]
type = "vc_redist"
id = "vcredist140"
version = ">=14.0"
platforms = ["windows"]
reason = "Visual C++ 2015-2022 Redistributable required"

# 运行时依赖
[[runtimes.system_deps.depends]]
type = "runtime"
id = "openssl"
platforms = ["linux"]
optional = true

# 推荐依赖
[[runtimes.system_deps.recommends]]
type = "package"
id = "git-lfs"
reason = "Git Large File Storage support"

# === 安装策略 ===
[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "choco"
package = "git"
params = "/GitAndUnixToolsOnPath /NoAutoCrlf /WindowsTerminal"
install_args = "/DIR=C:\\git"
priority = 80

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "winget"
package = "Git.Git"
priority = 70

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "brew"
package = "git"
priority = 90

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "apt"
package = "git"
priority = 90

[[runtimes.system_install.strategies]]
type = "direct_download"
url = "https://github.com/git-for-windows/git/releases/download/v{version}.windows.1/Git-{version}-64-bit.tar.bz2"
format = "tar.bz2"
priority = 30

# Git 提供的其他工具
[[runtimes.system_install.provides]]
name = "curl"
relative_path = "mingw64/bin/curl.exe"
platforms = ["windows"]

[[runtimes.system_install.provides]]
name = "ssh"
relative_path = "usr/bin/ssh.exe"
platforms = ["windows"]
```

### 3. 系统依赖解析器

```rust
// crates/vx-system-pm/src/resolver.rs

use crate::{SystemDependency, SystemDepType, InstallStrategy};
use anyhow::Result;
use std::collections::HashMap;

/// 系统依赖解析结果
#[derive(Debug)]
pub struct DependencyResolution {
    /// 需要安装的依赖（按顺序）
    pub to_install: Vec<ResolvedDependency>,
    /// 已满足的依赖
    pub satisfied: Vec<ResolvedDependency>,
    /// 无法解析的依赖
    pub unresolved: Vec<UnresolvedDependency>,
}

#[derive(Debug)]
pub struct ResolvedDependency {
    pub dep: SystemDependency,
    pub strategy: InstallStrategy,
    pub installed_version: Option<String>,
}

#[derive(Debug)]
pub struct UnresolvedDependency {
    pub dep: SystemDependency,
    pub reason: String,
}

/// 系统依赖解析器
pub struct SystemDependencyResolver {
    /// 已安装依赖缓存
    installed_cache: HashMap<String, InstalledInfo>,
    /// 包管理器检测器
    pm_detector: PackageManagerDetector,
}

impl SystemDependencyResolver {
    pub fn new() -> Self {
        Self {
            installed_cache: HashMap::new(),
            pm_detector: PackageManagerDetector::new(),
        }
    }

    /// 解析依赖
    pub async fn resolve(&mut self, deps: &[SystemDependency]) -> Result<DependencyResolution> {
        let mut to_install = Vec::new();
        let mut satisfied = Vec::new();
        let mut unresolved = Vec::new();

        for dep in deps {
            // 检查平台条件
            if !self.matches_platform(dep) {
                continue;
            }

            // 检查是否已安装
            match self.check_installed(dep).await? {
                InstallStatus::Installed(version) => {
                    satisfied.push(ResolvedDependency {
                        dep: dep.clone(),
                        strategy: InstallStrategy::default(),
                        installed_version: Some(version),
                    });
                }
                InstallStatus::NotInstalled => {
                    // 选择安装策略
                    match self.select_strategy(dep).await {
                        Some(strategy) => {
                            to_install.push(ResolvedDependency {
                                dep: dep.clone(),
                                strategy,
                                installed_version: None,
                            });
                        }
                        None if !dep.optional => {
                            unresolved.push(UnresolvedDependency {
                                dep: dep.clone(),
                                reason: "No installation strategy available".to_string(),
                            });
                        }
                        None => {
                            // 可选依赖，跳过
                        }
                    }
                }
                InstallStatus::VersionMismatch { installed, required } => {
                    // 版本不匹配，需要升级
                    match self.select_strategy(dep).await {
                        Some(strategy) => {
                            to_install.push(ResolvedDependency {
                                dep: dep.clone(),
                                strategy,
                                installed_version: Some(installed),
                            });
                        }
                        None => {
                            unresolved.push(UnresolvedDependency {
                                dep: dep.clone(),
                                reason: format!(
                                    "Installed version {} doesn't satisfy {}",
                                    installed, required
                                ),
                            });
                        }
                    }
                }
            }
        }

        // 拓扑排序
        self.topological_sort(&mut to_install)?;

        Ok(DependencyResolution {
            to_install,
            satisfied,
            unresolved,
        })
    }

    /// 检查依赖是否已安装
    async fn check_installed(&self, dep: &SystemDependency) -> Result<InstallStatus> {
        match dep.dep_type {
            SystemDepType::WindowsKb => self.check_kb_installed(&dep.id).await,
            SystemDepType::VcRedist => self.check_vcredist_installed(&dep.id, &dep.version).await,
            SystemDepType::DotNet => self.check_dotnet_installed(&dep.id, &dep.version).await,
            SystemDepType::WindowsFeature => self.check_feature_installed(&dep.id).await,
            SystemDepType::Package => self.check_package_installed(&dep.id, &dep.version).await,
            SystemDepType::Runtime => self.check_runtime_installed(&dep.id, &dep.version).await,
        }
    }

    /// 检查 Windows KB 更新
    #[cfg(windows)]
    async fn check_kb_installed(&self, kb_id: &str) -> Result<InstallStatus> {
        use std::process::Command;
        
        // 使用 WMIC 或 PowerShell 检查
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Get-HotFix -Id {} -ErrorAction SilentlyContinue | Select-Object -ExpandProperty HotFixID",
                    kb_id.to_uppercase()
                ),
            ])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().to_uppercase().contains(&kb_id.to_uppercase()) {
                return Ok(InstallStatus::Installed(kb_id.to_string()));
            }
        }

        Ok(InstallStatus::NotInstalled)
    }

    /// 检查 VCRedist
    #[cfg(windows)]
    async fn check_vcredist_installed(&self, id: &str, version: &Option<String>) -> Result<InstallStatus> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let uninstall_key = hklm.open_subkey(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"
        )?;

        for key_name in uninstall_key.enum_keys().filter_map(|k| k.ok()) {
            if let Ok(subkey) = uninstall_key.open_subkey(&key_name) {
                let display_name: Result<String, _> = subkey.get_value("DisplayName");
                if let Ok(name) = display_name {
                    if name.contains("Visual C++") && name.contains("Redistributable") {
                        // 检查版本
                        if let Some(required_version) = version {
                            let installed_version: Result<String, _> = subkey.get_value("DisplayVersion");
                            if let Ok(ver) = installed_version {
                                if self.version_satisfies(&ver, required_version) {
                                    return Ok(InstallStatus::Installed(ver));
                                }
                            }
                        } else {
                            return Ok(InstallStatus::Installed("unknown".to_string()));
                        }
                    }
                }
            }
        }

        Ok(InstallStatus::NotInstalled)
    }

    /// 选择最佳安装策略
    async fn select_strategy(&self, dep: &SystemDependency) -> Option<InstallStrategy> {
        // 根据依赖类型选择策略
        match dep.dep_type {
            SystemDepType::WindowsKb => {
                // KB 更新通过 Chocolatey 或 Windows Update
                if self.pm_detector.is_available("choco").await {
                    Some(InstallStrategy::PackageManager {
                        manager: "choco".to_string(),
                        package: dep.id.clone(),
                        params: None,
                        install_args: None,
                        priority: 80,
                    })
                } else {
                    None // 需要手动安装
                }
            }
            SystemDepType::VcRedist => {
                self.select_vcredist_strategy(&dep.id).await
            }
            SystemDepType::Package => {
                self.select_package_strategy(&dep.id).await
            }
            _ => None,
        }
    }

    async fn select_vcredist_strategy(&self, id: &str) -> Option<InstallStrategy> {
        // 优先使用 winget，其次 Chocolatey
        if self.pm_detector.is_available("winget").await {
            Some(InstallStrategy::PackageManager {
                manager: "winget".to_string(),
                package: format!("Microsoft.VCRedist.2015+.x64"),
                params: None,
                install_args: None,
                priority: 90,
            })
        } else if self.pm_detector.is_available("choco").await {
            Some(InstallStrategy::PackageManager {
                manager: "choco".to_string(),
                package: "vcredist140".to_string(),
                params: None,
                install_args: None,
                priority: 80,
            })
        } else {
            // 直接下载
            Some(InstallStrategy::DirectDownload {
                url: "https://aka.ms/vs/17/release/vc_redist.x64.exe".to_string(),
                format: None,
                executable_path: None,
                priority: 50,
            })
        }
    }
}

#[derive(Debug)]
enum InstallStatus {
    Installed(String),
    NotInstalled,
    VersionMismatch { installed: String, required: String },
}
```

### 4. 包管理器抽象

```rust
// crates/vx-system-pm/src/managers/mod.rs

use async_trait::async_trait;
use anyhow::Result;

pub mod chocolatey;
pub mod winget;
pub mod homebrew;
pub mod apt;
pub mod scoop;

/// 系统包管理器 trait
#[async_trait]
pub trait SystemPackageManager: Send + Sync {
    /// 包管理器名称
    fn name(&self) -> &str;
    
    /// 支持的平台
    fn supported_platforms(&self) -> Vec<&str>;
    
    /// 检查是否已安装
    async fn is_installed(&self) -> bool;
    
    /// 安装包管理器自身
    async fn install_self(&self) -> Result<()>;
    
    /// 安装包
    async fn install_package(&self, spec: &PackageInstallSpec) -> Result<InstallResult>;
    
    /// 卸载包
    async fn uninstall_package(&self, package: &str) -> Result<()>;
    
    /// 检查包是否已安装
    async fn is_package_installed(&self, package: &str) -> Result<bool>;
    
    /// 获取已安装版本
    async fn get_installed_version(&self, package: &str) -> Result<Option<String>>;
    
    /// 优先级（越高越优先）
    fn priority(&self) -> i32 { 50 }
}

/// 包安装规格
#[derive(Debug, Clone)]
pub struct PackageInstallSpec {
    /// 包名
    pub package: String,
    /// 版本约束
    pub version: Option<String>,
    /// 安装参数 (Chocolatey --params)
    pub params: Option<String>,
    /// 原生安装器参数
    pub install_args: Option<String>,
    /// 静默安装
    pub silent: bool,
    /// 安装目录
    pub install_dir: Option<std::path::PathBuf>,
}

/// 安装结果
#[derive(Debug)]
pub struct InstallResult {
    pub success: bool,
    pub version: Option<String>,
    pub install_path: Option<std::path::PathBuf>,
    pub message: Option<String>,
}
```

### 5. Chocolatey 实现

```rust
// crates/vx-system-pm/src/managers/chocolatey.rs

use super::{SystemPackageManager, PackageInstallSpec, InstallResult};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::process::Command;

pub struct ChocolateyManager;

impl ChocolateyManager {
    pub fn new() -> Self {
        Self
    }

    /// 检查是否需要管理员权限
    fn needs_elevation() -> bool {
        #[cfg(windows)]
        {
            use std::mem;
            use windows_sys::Win32::Security::*;
            use windows_sys::Win32::System::Threading::*;
            
            unsafe {
                let mut token: HANDLE = 0;
                if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                    return true;
                }
                
                let mut elevation: TOKEN_ELEVATION = mem::zeroed();
                let mut size = mem::size_of::<TOKEN_ELEVATION>() as u32;
                
                if GetTokenInformation(
                    token,
                    TokenElevation,
                    &mut elevation as *mut _ as *mut _,
                    size,
                    &mut size,
                ) == 0 {
                    return true;
                }
                
                elevation.TokenIsElevated == 0
            }
        }
        #[cfg(not(windows))]
        false
    }
}

#[async_trait]
impl SystemPackageManager for ChocolateyManager {
    fn name(&self) -> &str {
        "choco"
    }

    fn supported_platforms(&self) -> Vec<&str> {
        vec!["windows"]
    }

    async fn is_installed(&self) -> bool {
        which::which("choco").is_ok()
    }

    async fn install_self(&self) -> Result<()> {
        if Self::needs_elevation() {
            return Err(anyhow!("Administrator privileges required to install Chocolatey"));
        }

        let script = r#"
            Set-ExecutionPolicy Bypass -Scope Process -Force;
            [System.Net.ServicePointManager]::SecurityProtocol = 
                [System.Net.ServicePointManager]::SecurityProtocol -bor 3072;
            iex ((New-Object System.Net.WebClient).DownloadString(
                'https://community.chocolatey.org/install.ps1'))
        "#;

        let status = Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to install Chocolatey"))
        }
    }

    async fn install_package(&self, spec: &PackageInstallSpec) -> Result<InstallResult> {
        let mut args = vec!["install", &spec.package, "-y"];

        if let Some(version) = &spec.version {
            args.extend(["--version", version]);
        }

        if let Some(params) = &spec.params {
            args.extend(["--params", &format!("\"{}\"", params)]);
        }

        if let Some(install_args) = &spec.install_args {
            args.extend(["--install-arguments", &format!("\"{}\"", install_args)]);
        }

        if let Some(dir) = &spec.install_dir {
            // 某些包支持通过 install_args 指定目录
            let dir_arg = format!("/D={}", dir.display());
            if spec.install_args.is_none() {
                args.extend(["--install-arguments", &format!("\"{}\"", dir_arg)]);
            }
        }

        let output = Command::new("choco")
            .args(&args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            // 解析安装版本
            let version = self.get_installed_version(&spec.package).await?;
            
            Ok(InstallResult {
                success: true,
                version,
                install_path: None,
                message: Some(stdout.to_string()),
            })
        } else {
            Err(anyhow!("Chocolatey install failed: {}", stderr))
        }
    }

    async fn uninstall_package(&self, package: &str) -> Result<()> {
        let status = Command::new("choco")
            .args(["uninstall", package, "-y"])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to uninstall {}", package))
        }
    }

    async fn is_package_installed(&self, package: &str) -> Result<bool> {
        let output = Command::new("choco")
            .args(["list", "--local-only", "--exact", package])
            .output()?;

        Ok(output.status.success() && 
           String::from_utf8_lossy(&output.stdout).contains(package))
    }

    async fn get_installed_version(&self, package: &str) -> Result<Option<String>> {
        let output = Command::new("choco")
            .args(["list", "--local-only", "--exact", package])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // 解析输出: "package 1.2.3"
            for line in stdout.lines() {
                if line.starts_with(package) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        return Ok(Some(parts[1].to_string()));
                    }
                }
            }
        }
        Ok(None)
    }

    fn priority(&self) -> i32 {
        80
    }
}
```

### 6. 系统工具执行器集成

```rust
// crates/vx-resolver/src/system_executor.rs

use vx_system_pm::{SystemDependencyResolver, PackageManagerRegistry};
use anyhow::Result;

/// 系统工具执行器
pub struct SystemToolExecutor {
    dep_resolver: SystemDependencyResolver,
    pm_registry: PackageManagerRegistry,
}

impl SystemToolExecutor {
    pub async fn execute(&self, tool_name: &str, args: &[String]) -> Result<i32> {
        // 1. 检查工具是否已安装
        if let Some(path) = which::which(tool_name).ok() {
            return self.run_tool(&path, args).await;
        }

        // 2. 获取工具的安装规格
        let spec = self.get_tool_spec(tool_name)?;

        // 3. 解析并安装系统依赖
        if let Some(sys_deps) = &spec.system_deps {
            // 处理前置依赖
            self.install_dependencies(&sys_deps.pre_depends).await?;
            // 处理运行时依赖
            self.install_dependencies(&sys_deps.depends).await?;
        }

        // 4. 安装工具本身
        let install_result = self.install_tool(&spec).await?;

        // 5. 执行工具
        self.run_tool(&install_result.path, args).await
    }

    async fn install_dependencies(&self, deps: &[SystemDependency]) -> Result<()> {
        let resolution = self.dep_resolver.resolve(deps).await?;

        // 检查是否有无法解析的必需依赖
        if !resolution.unresolved.is_empty() {
            let missing: Vec<_> = resolution.unresolved
                .iter()
                .filter(|d| !d.dep.optional)
                .map(|d| format!("{}: {}", d.dep.id, d.reason))
                .collect();
            
            if !missing.is_empty() {
                return Err(anyhow::anyhow!(
                    "Cannot resolve required dependencies:\n  {}",
                    missing.join("\n  ")
                ));
            }
        }

        // 按顺序安装依赖
        for resolved in &resolution.to_install {
            println!("Installing dependency: {} ...", resolved.dep.id);
            self.execute_strategy(&resolved.strategy).await?;
        }

        Ok(())
    }

    async fn execute_strategy(&self, strategy: &InstallStrategy) -> Result<()> {
        match strategy {
            InstallStrategy::PackageManager { manager, package, params, install_args, .. } => {
                let pm = self.pm_registry.get(manager)?;
                
                // 确保包管理器已安装
                if !pm.is_installed().await {
                    println!("Installing {} package manager...", manager);
                    pm.install_self().await?;
                }

                // 安装包
                let spec = PackageInstallSpec {
                    package: package.clone(),
                    version: None,
                    params: params.clone(),
                    install_args: install_args.clone(),
                    silent: true,
                    install_dir: None,
                };
                
                pm.install_package(&spec).await?;
            }
            InstallStrategy::DirectDownload { url, format, executable_path, .. } => {
                // 使用 vx-installer 下载安装
                todo!("Implement direct download")
            }
            InstallStrategy::Script { url, script_type, args, .. } => {
                // 下载并执行脚本
                todo!("Implement script execution")
            }
            InstallStrategy::ProvidedBy { provider, relative_path, .. } => {
                // 确保 provider 已安装，然后使用其提供的工具
                todo!("Implement provided-by")
            }
        }
        Ok(())
    }
}
```

### 7. Crate 目录结构

```
crates/
└── vx-system-pm/
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   ├── dependency.rs       # SystemDependency 定义
    │   ├── strategy.rs         # InstallStrategy 定义
    │   ├── resolver.rs         # SystemDependencyResolver
    │   ├── registry.rs         # PackageManagerRegistry
    │   ├── detector.rs         # 系统检测工具
    │   ├── error.rs            # 错误类型
    │   └── managers/
    │       ├── mod.rs          # SystemPackageManager trait
    │       ├── chocolatey.rs   # Chocolatey 实现
    │       ├── winget.rs       # winget 实现
    │       ├── scoop.rs        # Scoop 实现
    │       ├── homebrew.rs     # Homebrew 实现
    │       ├── apt.rs          # APT 实现
    │       ├── yum.rs          # YUM/DNF 实现
    │       └── pacman.rs       # Pacman 实现
    └── tests/
        ├── resolver_tests.rs
        ├── chocolatey_tests.rs
        └── integration_tests.rs
```

### 8. vx-manifest 扩展

```rust
// crates/vx-manifest/src/provider/system_deps.rs

use serde::{Deserialize, Serialize};

/// 系统依赖配置
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SystemDepsConfig {
    /// 前置依赖（安装前必须满足）
    #[serde(default)]
    pub pre_depends: Vec<SystemDependency>,
    
    /// 运行时依赖
    #[serde(default)]
    pub depends: Vec<SystemDependency>,
    
    /// 推荐依赖
    #[serde(default)]
    pub recommends: Vec<SystemDependency>,
    
    /// 可选依赖
    #[serde(default)]
    pub suggests: Vec<SystemDependency>,
}

/// 系统安装配置
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SystemInstallConfig {
    /// 安装策略列表
    #[serde(default)]
    pub strategies: Vec<InstallStrategyDef>,
    
    /// 此 runtime 提供的其他工具
    #[serde(default)]
    pub provides: Vec<ProvidedTool>,
}

/// 提供的工具定义
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProvidedTool {
    /// 工具名称
    pub name: String,
    /// 相对路径
    pub relative_path: String,
    /// 支持的平台
    #[serde(default)]
    pub platforms: Vec<String>,
}
```

## 向后兼容性

### 兼容策略

1. **可选字段**: `system_deps` 和 `system_install` 都是可选字段
2. **渐进增强**: 现有 provider.toml 无需修改即可继续工作
3. **默认行为**: 未配置系统依赖时，行为与现有一致
4. **优先级覆盖**: 用户本地 provider 可以覆盖内置 provider

### 迁移路径

```bash
# 验证 provider.toml 格式
vx manifest validate

# 检查系统依赖配置
vx manifest check-deps

# 列出所有 providers（包括用户自定义）
vx provider list --all

# 查看 provider 来源
vx provider info git
# Output: git (built-in, overridden by ~/.vx/providers/mytools)
```

## 实现计划

### Phase 1: 核心框架 (v0.5.0) ✅ 已完成

- [x] `vx-system-pm` crate 基础结构
  - 实现位置: `crates/vx-system-pm/`
- [x] `SystemPackageManager` trait 定义
  - 实现位置: `crates/vx-system-pm/src/managers/mod.rs`
- [x] Chocolatey 实现
  - 实现位置: `crates/vx-system-pm/src/managers/chocolatey.rs`
- [x] winget 实现
  - 实现位置: `crates/vx-system-pm/src/managers/winget.rs`
- [x] 系统依赖解析器
  - 实现位置: `crates/vx-system-pm/src/resolver.rs`

### Phase 2: Manifest-Driven Runtime (v0.5.1) ✅ 已完成

- [x] `ManifestDrivenRuntime` 实现
  - 实现位置: `crates/vx-runtime/src/manifest_runtime.rs`
- [x] `ProviderLoader` 实现
  - 实现位置: `crates/vx-runtime/src/provider_loader.rs`
- [x] 用户本地 provider 支持 (`~/.vx/providers/`)
  - 通过 `VxPaths.providers_dir` 支持
- [x] `VX_PROVIDERS_PATH` 环境变量支持
  - 在 `ProviderLoaderConfig::default()` 中实现
- [x] Provider 优先级和覆盖机制
  - 用户 provider 优先于内置 provider

### Phase 3: 跨平台支持 (v0.5.2) ✅ 已完成

- [x] Homebrew 实现
  - 实现位置: `crates/vx-system-pm/src/managers/homebrew.rs`
- [x] APT 实现
  - 实现位置: `crates/vx-system-pm/src/managers/apt.rs`
- [x] Scoop 实现
  - 实现位置: `crates/vx-system-pm/src/managers/scoop.rs`
- [ ] YUM/DNF 实现 (待实现)
- [ ] Pacman 实现 (待实现)

### Phase 4: 依赖检测 (v0.5.3) ✅ 已完成

- [x] Windows KB 检测
  - 实现位置: `crates/vx-system-pm/src/resolver.rs` (`check_kb_installed`)
- [x] VCRedist 检测
  - 实现位置: `crates/vx-system-pm/src/resolver.rs` (`check_vcredist_installed`)
- [x] .NET Framework 检测
  - 实现位置: `crates/vx-system-pm/src/resolver.rs` (`check_dotnet_installed`)
- [x] Windows Feature 检测
  - 实现位置: `crates/vx-system-pm/src/resolver.rs` (`check_feature_installed`)

### Phase 5: 迁移内置 Providers (v0.6.0)

迁移以下 providers 为纯配置驱动：

| Provider | 预计节省代码 | 优先级 |
|----------|-------------|--------|
| git | ~100 行 | P0 |
| cmake | ~110 行 | P0 |
| ninja | ~100 行 | P1 |
| curl | 已完成 | - |
| just | ~90 行 | P1 |
| helm | ~70 行 | P1 |
| kubectl | ~80 行 | P1 |
| terraform | ~100 行 | P2 |
| task | ~90 行 | P2 |
| protoc | ~100 行 | P2 |
| ollama | ~100 行 | P2 |
| docker | ~100 行 | P2 |

### Phase 6: CLI 和文档 (v0.6.1)

- [ ] `vx provider` 子命令
- [ ] `vx system-deps` 子命令
- [ ] 用户文档更新
- [ ] Provider 开发指南

## CLI 命令设计

### Provider 管理

```bash
# 列出所有 providers
vx provider list
# Output:
# NAME       SOURCE          RUNTIMES
# node       built-in        node, npm, npx
# python     built-in        python, pip
# mytools    ~/.vx/providers fd, bat, rg
# git        built-in        git

# 查看 provider 详情
vx provider info mytools
# Output:
# Provider: mytools
# Source: /home/user/.vx/providers/mytools/provider.toml
# Runtimes:
#   - fd: A simple, fast alternative to find
#   - bat: A cat clone with syntax highlighting
#   - rg: ripgrep - fast grep alternative

# 验证 provider 配置
vx provider validate ~/.vx/providers/mytools/provider.toml

# 创建新 provider（交互式）
vx provider init mytools
```

### 系统依赖管理

```bash
# 检查系统依赖状态
vx system-deps check git
# Output:
# Checking system dependencies for git...
# ✓ vcredist140 (14.36.32532) - installed
# ✗ kb2919355 - not installed (optional)

# 安装系统依赖
vx system-deps install git

# 列出可用的包管理器
vx system-deps managers
# Output:
# NAME        STATUS      PRIORITY
# choco       installed   80
# winget      installed   70
# scoop       not found   60
```

## 替代方案

### 方案 A: 仅使用直接下载

**优点**: 简单，不依赖外部包管理器
**缺点**: 无法处理系统级依赖（VCRedist、KB 更新）

### 方案 B: 完全依赖系统包管理器

**优点**: 利用成熟的依赖解析
**缺点**: 需要用户预先安装包管理器

### 方案 C: 所有 Provider 都用 Rust 实现

**优点**: 完全控制，类型安全
**缺点**: 维护成本高，扩展困难

### 选择理由

采用混合方案：
1. **版本敏感型工具**（node, python, go）保留 Rust 实现
2. **版本不敏感型工具**（git, cmake, curl）使用 manifest-driven
3. **系统依赖**通过包管理器处理，fallback 到直接下载

这样既能处理复杂依赖，又能在包管理器不可用时提供备选方案，同时大幅降低维护成本。

## 安全考虑

### 用户 Provider 安全

1. **来源验证**: 用户 provider 只从本地文件系统加载
2. **权限隔离**: 系统包管理器操作需要明确的用户确认
3. **脚本执行**: Script 类型的安装策略需要用户确认

### 包管理器安全

1. **官方源**: 优先使用官方包管理器源
2. **校验和**: 支持下载文件的校验和验证
3. **审计日志**: 记录所有系统级安装操作

## 参考资料

### 主流项目源码
- [Chocolatey Source](https://github.com/chocolatey/choco) - Windows 包管理器
- [winget-cli](https://github.com/microsoft/winget-cli) - Microsoft 官方包管理器
- [Scoop](https://github.com/ScoopInstaller/Scoop) - 轻量级 Windows 包管理器
- [Homebrew](https://github.com/Homebrew/brew) - macOS/Linux 包管理器
- [asdf](https://github.com/asdf-vm/asdf) - 插件式版本管理器

### 依赖库
- [which-rs](https://crates.io/crates/which) - 跨平台命令查找
- [winreg](https://crates.io/crates/winreg) - Windows 注册表操作

### 相关文档
- [Chocolatey Package Dependencies](https://docs.chocolatey.org/en-us/create/create-packages#package-dependencies)
- [winget Manifest Dependencies](https://learn.microsoft.com/en-us/windows/package-manager/winget/)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-01-13 | Draft | 初始草案 |
| 2026-01-13 | Draft v2 | 添加 Manifest-Driven Runtime 设计 |
