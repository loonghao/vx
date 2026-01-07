# RFC 0015: System Tool Discovery & Unified Execution

- **状态**: 草案
- **创建日期**: 2026-01-07
- **作者**: VX Team
- **关联**: RFC-0014 (Platform-Aware Providers)

## 摘要

让 vx 成为**统一的命令执行入口**，不仅管理需要安装的运行时，还能动态发现并执行系统工具。覆盖 **DevOps、AIOps、全栈开发** 的完整工具链。

设计目标：**零学习成本、AI 友好、环境一致、多环境共存**。

## 核心理念

```
┌─────────────────────────────────────────────────────────────────┐
│                     vx = 统一命令入口                           │
│                                                                 │
│   人类开发者: vx <command>  ──┐                                 │
│                              ├──► 一致的执行环境 ──► 结果       │
│   AI Agent:   vx <command>  ──┘                                 │
│                                                                 │
│   无需关心: 工具在哪里、如何安装、环境变量、平台差异            │
└─────────────────────────────────────────────────────────────────┘
```

## 设计原则

| 原则 | 说明 |
|------|------|
| **零学习成本** | 用户已知的命令语法不变，只需加 `vx` 前缀 |
| **AI 优先** | 所有命令通过 `vx` 执行，AI 无需了解平台差异 |
| **干净环境** | 每次执行创建干净环境，避免环境污染 |
| **多环境共存** | 支持多个项目环境同时存在，随时切换 |
| **自动激活** | 检测到 `vx.toml` 自动激活项目环境 |

---

## 一、优先集成的工具清单

### 1.1 按领域分类

#### 构建工具 (Build)

| 工具 | 平台 | 说明 |
|------|------|------|
| `xcodebuild` | macOS | Apple 项目构建 |
| `xcrun` | macOS | Xcode 工具链调用 |
| `msbuild` | Windows | .NET/C++ 构建 |
| `cmake` | 跨平台 | 跨平台构建系统 |
| `make` | 跨平台 | 经典构建工具 |
| `ninja` | 跨平台 | 高速构建系统 |
| `bazel` | 跨平台 | Google 构建系统 |
| `gradle` | 跨平台 | Java/Android 构建 |
| `maven` | 跨平台 | Java 构建 |

#### 容器与编排 (Container & Orchestration)

| 工具 | 平台 | 说明 |
|------|------|------|
| `docker` | 跨平台 | 容器运行时 |
| `docker-compose` | 跨平台 | 多容器编排 |
| `podman` | 跨平台 | 无守护进程容器 |
| `kubectl` | 跨平台 | Kubernetes CLI |
| `helm` | 跨平台 | K8s 包管理 |
| `k9s` | 跨平台 | K8s TUI |
| `minikube` | 跨平台 | 本地 K8s |
| `kind` | 跨平台 | K8s in Docker |

#### 云平台 CLI (Cloud)

| 工具 | 平台 | 说明 |
|------|------|------|
| `aws` | 跨平台 | AWS CLI |
| `az` | 跨平台 | Azure CLI |
| `gcloud` | 跨平台 | Google Cloud CLI |
| `terraform` | 跨平台 | 基础设施即代码 |
| `pulumi` | 跨平台 | 现代 IaC |
| `ansible` | 跨平台 | 配置管理 |

#### CI/CD 工具

| 工具 | 平台 | 说明 |
|------|------|------|
| `gh` | 跨平台 | GitHub CLI |
| `gitlab` | 跨平台 | GitLab CLI |
| `jenkins-cli` | 跨平台 | Jenkins CLI |
| `act` | 跨平台 | 本地运行 GitHub Actions |

#### 网络与调试 (Network & Debug)

| 工具 | 平台 | 说明 |
|------|------|------|
| `curl` | 跨平台 | HTTP 客户端 |
| `wget` | 跨平台 | 下载工具 |
| `httpie` | 跨平台 | 现代 HTTP 客户端 |
| `ssh` | 跨平台 | 远程连接 |
| `scp` | 跨平台 | 安全复制 |
| `rsync` | macOS/Linux | 文件同步 |
| `netstat` | 跨平台 | 网络状态 |
| `tcpdump` | macOS/Linux | 网络抓包 |
| `wireshark` | 跨平台 | 网络分析 |

#### 数据库工具 (Database)

| 工具 | 平台 | 说明 |
|------|------|------|
| `psql` | 跨平台 | PostgreSQL CLI |
| `mysql` | 跨平台 | MySQL CLI |
| `mongosh` | 跨平台 | MongoDB Shell |
| `redis-cli` | 跨平台 | Redis CLI |
| `sqlite3` | 跨平台 | SQLite CLI |

#### 监控与日志 (Monitoring & Logging)

| 工具 | 平台 | 说明 |
|------|------|------|
| `htop` | macOS/Linux | 进程监控 |
| `top` | 跨平台 | 系统监控 |
| `iotop` | Linux | IO 监控 |
| `journalctl` | Linux | 系统日志 |
| `dmesg` | macOS/Linux | 内核日志 |
| `prometheus` | 跨平台 | 监控系统 |
| `grafana-cli` | 跨平台 | 可视化 |

#### 安全工具 (Security)

| 工具 | 平台 | 说明 |
|------|------|------|
| `codesign` | macOS | 代码签名 |
| `signtool` | Windows | 代码签名 |
| `gpg` | 跨平台 | 加密签名 |
| `openssl` | 跨平台 | SSL/TLS 工具 |
| `certutil` | Windows | 证书管理 |
| `security` | macOS | 钥匙串管理 |

#### 版本控制 (VCS)

| 工具 | 平台 | 说明 |
|------|------|------|
| `git` | 跨平台 | 版本控制 |
| `git-lfs` | 跨平台 | 大文件存储 |
| `svn` | 跨平台 | Subversion |
| `hg` | 跨平台 | Mercurial |

#### 文件系统 (Filesystem)

| 工具 | 平台 | 说明 |
|------|------|------|
| `tar` | 跨平台 | 归档工具 |
| `zip` / `unzip` | 跨平台 | ZIP 压缩 |
| `7z` | 跨平台 | 7-Zip |
| `robocopy` | Windows | 高级复制 |
| `xcopy` | Windows | 扩展复制 |
| `find` | macOS/Linux | 文件查找 |
| `fd` | 跨平台 | 现代 find |
| `rg` (ripgrep) | 跨平台 | 高速搜索 |

#### AIOps / MLOps

| 工具 | 平台 | 说明 |
|------|------|------|
| `nvidia-smi` | 跨平台 | GPU 监控 |
| `nvcc` | 跨平台 | CUDA 编译器 |
| `mlflow` | 跨平台 | ML 生命周期 |
| `dvc` | 跨平台 | 数据版本控制 |
| `wandb` | 跨平台 | 实验跟踪 |

### 1.2 平台特定工具

#### macOS 专属

```toml
[tools.macos]
# Apple 开发
xcodebuild = { category = "build" }
xcrun = { category = "build" }
swift = { category = "language" }
swiftc = { category = "build" }
xcode-select = { category = "system" }
# 系统工具
codesign = { category = "security" }
notarytool = { category = "security" }
security = { category = "security" }
plutil = { category = "system" }
defaults = { category = "system" }
launchctl = { category = "system" }
diskutil = { category = "system" }
hdiutil = { category = "system" }
pkgbuild = { category = "package" }
productbuild = { category = "package" }
# Homebrew
brew = { category = "package" }
```

#### Windows 专属

```toml
[tools.windows]
# 构建工具
msbuild = { discover = "vswhere", category = "build" }
devenv = { discover = "vswhere", category = "build" }
cl = { discover = "vcvars", category = "build" }
link = { discover = "vcvars", category = "build" }
# 系统工具
robocopy = { category = "filesystem" }
xcopy = { category = "filesystem" }
sfc = { category = "system", requires_admin = true }
dism = { category = "system", requires_admin = true }
certutil = { category = "security" }
signtool = { discover = "windows_sdk", category = "security" }
# PowerShell
pwsh = { category = "shell" }
powershell = { category = "shell" }
# 包管理
winget = { category = "package" }
choco = { category = "package" }
scoop = { category = "package" }
```

#### Linux 专属

```toml
[tools.linux]
# 包管理
apt = { path = "/usr/bin/apt", category = "package" }
apt-get = { path = "/usr/bin/apt-get", category = "package" }
yum = { path = "/usr/bin/yum", category = "package" }
dnf = { path = "/usr/bin/dnf", category = "package" }
pacman = { path = "/usr/bin/pacman", category = "package" }
# 系统服务
systemctl = { category = "system" }
journalctl = { category = "system" }
service = { category = "system" }
# 网络
iptables = { category = "network", requires_sudo = true }
ip = { category = "network" }
ss = { category = "network" }
```

---

## 二、虚拟环境与隔离策略

### 2.1 设计目标

1. **环境隔离** - 防止用户随意安装导致全局环境冲突
2. **快速创建** - 秒级创建新的虚拟环境
3. **空间复用** - 相同版本的工具不重复占用磁盘
4. **PATH 优先级** - vx 管理的工具优先于系统工具

### 2.2 主流方案对比 (2025-2026)

| 方案 | 工具 | 实现方式 | 优点 | 缺点 |
|------|------|----------|------|------|
| **硬链接** | uv, pnpm | 同分区硬链接 | 极快、零额外空间 | 跨分区失败 |
| **软链接** | Nix, mise | 符号链接到 store | 快速、跨分区支持 | Windows 兼容性 |
| **Shim** | asdf, mise | 拦截器代理 | 灵活、动态切换 | 每次调用有开销 |
| **复制** | venv (fallback) | 完整复制文件 | 最兼容 | 慢、占空间 |

#### uv 的硬链接策略

```bash
# uv 默认使用硬链接，跨分区时回退到复制
$ uv venv
warning: Failed to hardlink files; falling back to full copy.

# 可配置链接模式
export UV_LINK_MODE=copy      # 强制复制
export UV_LINK_MODE=hardlink  # 强制硬链接
```

#### mise 的 Shim vs PATH 策略

```bash
# Shim 模式 - 适合 IDE/脚本
eval "$(mise activate --shims)"

# PATH 模式 - 适合交互式 Shell
eval "$(mise activate bash)"
```

### 2.3 vx 的混合策略 (推荐)

结合各方案优点，vx 采用**分层隔离 + 智能链接**策略：

```
┌─────────────────────────────────────────────────────────────────┐
│                    vx 存储架构                                   │
├─────────────────────────────────────────────────────────────────┤
│  ~/.vx/                                                         │
│  ├── store/              # 内容寻址存储 (Content-Addressable)   │
│  │   ├── node/                                                  │
│  │   │   ├── 22.0.0-darwin-arm64/     # 实际安装                │
│  │   │   └── 20.0.0-darwin-arm64/                              │
│  │   ├── python/                                                │
│  │   │   └── 3.11.0-darwin-arm64/                              │
│  │   └── ...                                                    │
│  │                                                              │
│  ├── envs/               # 虚拟环境 (软链接组合)                 │
│  │   ├── project-a/      # 项目 A 的环境                        │
│  │   │   └── bin/                                               │
│  │   │       ├── node -> ../../store/node/22.0.0/bin/node      │
│  │   │       ├── npm -> ../../store/node/22.0.0/bin/npm        │
│  │   │       └── python -> ../../store/python/3.11/bin/python  │
│  │   ├── project-b/      # 项目 B 的环境                        │
│  │   │   └── bin/                                               │
│  │   │       ├── node -> ../../store/node/20.0.0/bin/node      │
│  │   │       └── ...                                            │
│  │   └── _default/       # 默认环境                             │
│  │                                                              │
│  └── shims/              # 全局 Shim (用于 IDE/非交互式)         │
│      ├── node            # shim 脚本                            │
│      ├── npm                                                    │
│      └── ...                                                    │
└─────────────────────────────────────────────────────────────────┘
```

### 2.4 链接策略选择

```rust
/// 链接模式
pub enum LinkMode {
    /// 硬链接 (默认，同分区时最快)
    Hardlink,
    /// 软链接 (跨分区、Windows 兼容)
    Symlink,
    /// 复制 (最兼容，fallback)
    Copy,
    /// 自动选择最优策略
    Auto,
}

impl LinkMode {
    /// 自动选择最优链接策略
    pub fn auto_select(source: &Path, target: &Path) -> Self {
        // 1. 检查是否同一文件系统
        if same_filesystem(source, target) {
            return LinkMode::Hardlink;
        }
        
        // 2. Windows 检查符号链接权限
        #[cfg(windows)]
        if !has_symlink_privilege() {
            return LinkMode::Copy;
        }
        
        // 3. 默认使用软链接
        LinkMode::Symlink
    }
}
```

### 2.5 虚拟环境创建 (秒级)

```bash
# 创建新的虚拟环境 (基于软链接，极快)
$ vx env create my-env --with node@22 python@3.11
Creating environment 'my-env'...
  ✓ Linking node@22.0.0 (symlink)
  ✓ Linking python@3.11.0 (symlink)
Done in 0.3s

# 环境结构
~/.vx/envs/my-env/
└── bin/
    ├── node -> ../../store/node/22.0.0/bin/node
    ├── npm -> ../../store/node/22.0.0/bin/npm
    ├── python -> ../../store/python/3.11.0/bin/python
    └── pip -> ../../store/python/3.11.0/bin/pip
```

### 2.6 项目本地模式 (可选)

对于特殊场景（CI、离线环境、完全隔离），支持**项目本地存储**：

```toml
# vx.toml
[settings]
# 默认使用全局 store (推荐)
local_store = false

# 启用项目本地 store
# local_store = true
```

#### 本地模式目录结构

```
my-project/
├── vx.toml
├── vx.lock
├── .vx/                        # 项目本地 vx 目录 (gitignore)
│   ├── store/                  # 本地存储 (完整安装)
│   │   ├── node/
│   │   │   └── 22.0.0/
│   │   └── python/
│   │       └── 3.11.0/
│   └── bin/                    # 软链接到 store
│       ├── node -> ../store/node/22.0.0/bin/node
│       ├── npm -> ../store/node/22.0.0/bin/npm
│       └── python -> ../store/python/3.11.0/bin/python
└── src/
```

#### 使用场景

| 场景 | 推荐模式 | 原因 |
|------|----------|------|
| **日常开发** | 全局 store | 节省空间，共享版本 |
| **CI/CD** | 本地 store | 隔离，可缓存 `.vx/` |
| **离线环境** | 本地 store | 不依赖全局状态 |
| **Docker 构建** | 本地 store | 镜像层缓存友好 |
| **多用户共享** | 全局 store | 统一管理 |

#### 命令行切换

```bash
# 初始化项目为本地模式
$ vx init --local
Created vx.toml with local_store = true

# 将现有项目切换到本地模式
$ vx config set local_store true
$ vx sync  # 同步工具到本地 .vx/store

# 查看当前模式
$ vx config get local_store
local_store = false (using global store: ~/.vx/store)
```

#### 混合模式

可以选择性地将某些工具放在本地：

```toml
# vx.toml
[settings]
local_store = false  # 默认使用全局

[tools]
node = "22.0.0"      # 使用全局 store
python = "3.11"      # 使用全局 store

[tools.local]
# 这些工具强制使用项目本地 store
my-custom-tool = "1.0.0"
```

### 2.7 PATH 优先级管理

**核心原则**：vx 管理的工具 **始终优先于** 系统工具。

```bash
# vx 激活后的 PATH 顺序
PATH=
  ~/.vx/envs/current/bin      # 1. 当前项目环境 (最高优先级)
  ~/.vx/shims                  # 2. vx shims
  ~/.vx/bin                    # 3. vx 全局工具
  /usr/local/bin               # 4. 系统工具
  /usr/bin                     # 5. 系统基础
  ...
```

#### 实现方式

```rust
/// 构建 PATH 环境变量
pub fn build_path(env: &Environment) -> String {
    let mut paths = Vec::new();
    
    // 1. 当前项目环境 (最高优先级)
    if let Some(project_env) = &env.project {
        paths.push(project_env.bin_dir());
    }
    
    // 2. vx shims (用于动态解析)
    paths.push(vx_paths::shims_dir());
    
    // 3. vx 全局工具
    paths.push(vx_paths::global_bin());
    
    // 4. 过滤后的系统 PATH
    // 移除可能冲突的路径 (如 /usr/local/bin/node)
    for sys_path in std::env::var("PATH").unwrap_or_default().split(PATH_SEP) {
        if !is_conflicting_path(sys_path) {
            paths.push(PathBuf::from(sys_path));
        }
    }
    
    paths.iter()
        .map(|p| p.to_string_lossy())
        .collect::<Vec<_>>()
        .join(PATH_SEP)
}

/// 检查是否为冲突路径
fn is_conflicting_path(path: &str) -> bool {
    // 移除其他版本管理器的路径
    let conflicting = [
        ".nvm/", ".pyenv/", ".rbenv/", ".asdf/",
        "homebrew/opt/node", "homebrew/opt/python",
    ];
    conflicting.iter().any(|c| path.contains(c))
}
```

### 2.8 命令冲突解决

当系统中存在同名命令时，vx 的解决策略：

```bash
# 场景：系统有 node v18，项目需要 node v22

# 方案 1: vx 环境优先 (默认)
$ vx node --version
v22.0.0  # 使用 vx 管理的版本

# 方案 2: 显式使用系统版本
$ vx --system node --version
v18.0.0  # 使用系统 PATH 中的版本

# 方案 3: 查看所有可用版本
$ vx which --all node
node (vx managed):
  ~/.vx/envs/current/bin/node -> v22.0.0 (active)
  ~/.vx/store/node/20.0.0/bin/node
  
node (system):
  /usr/local/bin/node -> v18.0.0
  /usr/bin/node -> v16.0.0
```

### 2.9 多环境共存与切换

```bash
# 查看所有环境
$ vx env list
  NAME          TOOLS                    SIZE      CREATED
* project-a     node@22, python@3.11     12 KB     2 days ago
  project-b     node@20, go@1.23         8 KB      1 week ago
  ml-project    python@3.11, cuda@12     16 KB     3 days ago
  _default      node@lts, python@3       4 KB      1 month ago

# 切换环境
$ vx env use project-b
vx: switched to 'project-b'
  PATH updated: ~/.vx/envs/project-b/bin prepended

# 创建环境副本
$ vx env clone project-a project-a-test
vx: cloned 'project-a' to 'project-a-test' (0.1s)

# 删除环境 (只删除软链接，不删除 store)
$ vx env remove project-a-test
vx: removed 'project-a-test' (store untouched)
```

### 2.10 存储清理

```bash
# 查看存储使用情况
$ vx store status
Store: ~/.vx/store
  Total: 2.3 GB
  
  node:
    22.0.0    245 MB    used by: project-a, ml-project
    20.0.0    238 MB    used by: project-b
    18.0.0    230 MB    (unused)
    
  python:
    3.11.0    89 MB     used by: project-a, ml-project
    3.10.0    85 MB     (unused)

# 清理未使用的版本
$ vx store gc
Removing unused versions:
  node@18.0.0 (230 MB)
  python@3.10.0 (85 MB)
Freed: 315 MB
```

---

## 三、执行模式与环境激活

### 3.1 自动激活机制

当检测到 `vx.toml` 时，**自动激活项目环境**（类似 direnv）：

```bash
# 进入有 vx.toml 的目录
$ cd my-project
vx: activated 'my-project' (node 22.0.0, python 3.11)

# 自动使用项目配置的版本
$ node --version
v22.0.0

# 离开目录时自动退出
$ cd ..
vx: deactivated 'my-project'
```

### 3.2 手动 Shell 模式

没有 `vx.toml` 时，可以手动进入子 shell：

```bash
# 进入临时 vx shell
$ vx shell
(vx) $ node --version  # 使用 vx 默认版本
(vx) $ exit

# 指定工具版本进入
$ vx shell --with node@20 python@3.12
(vx) $ node --version
v20.0.0
```

### 3.3 多环境共存与切换

支持多个项目环境同时存在：

```bash
# 终端 1: 项目 A
$ cd project-a
vx: activated 'project-a' (node 18.0.0)

# 终端 2: 项目 B  
$ cd project-b
vx: activated 'project-b' (node 22.0.0)

# 两个环境独立，互不影响
```

#### 环境切换命令

```bash
# 查看当前激活的环境
$ vx env
Active: project-a
  node: 18.0.0
  python: 3.11

# 列出所有已知环境
$ vx env list
  project-a     ~/code/project-a     node@18, python@3.11
  project-b     ~/code/project-b     node@22, go@1.23
* project-c     ~/code/project-c     rust@1.84 (current)

# 手动切换到另一个项目环境
$ vx env use project-b
vx: switched to 'project-b'

# 临时使用不同版本（不修改配置）
$ vx env override node@20
vx: node overridden to 20.0.0 (session only)
```

### 3.4 环境激活实现

```rust
/// 环境激活策略
pub enum ActivationMode {
    /// 自动激活 (检测到 vx.toml)
    Auto,
    /// 手动 shell (vx shell)
    Manual,
    /// 单次执行 (vx <cmd>)
    Oneshot,
}

/// 环境管理器
pub struct EnvironmentManager {
    /// 当前激活的环境
    active: Option<ProjectEnvironment>,
    /// 所有已知环境
    known_envs: HashMap<String, ProjectEnvironment>,
    /// 临时覆盖
    overrides: HashMap<String, Version>,
}

impl EnvironmentManager {
    /// 检测并激活环境
    pub fn detect_and_activate(&mut self, cwd: &Path) -> Result<()> {
        if let Some(config_path) = find_vx_toml(cwd) {
            let env = ProjectEnvironment::load(&config_path)?;
            self.activate(env)?;
        }
        Ok(())
    }
    
    /// 切换环境
    pub fn switch_to(&mut self, name: &str) -> Result<()> {
        if let Some(env) = self.known_envs.get(name) {
            self.activate(env.clone())?;
        }
        Ok(())
    }
}
```

### 3.5 Shell 集成

#### Bash/Zsh 集成

```bash
# ~/.bashrc 或 ~/.zshrc
eval "$(vx hook bash)"  # 或 zsh

# 这会：
# 1. 设置 cd 钩子，自动检测 vx.toml
# 2. 设置提示符显示当前环境
# 3. 添加自动补全
```

#### PowerShell 集成

```powershell
# $PROFILE
Invoke-Expression (& vx hook pwsh)
```

#### Fish 集成

```fish
# ~/.config/fish/config.fish
vx hook fish | source
```

### 3.6 执行模式对比

| 特性 | 单次执行 `vx <cmd>` | 自动激活 | 手动 Shell `vx shell` |
|------|---------------------|----------|----------------------|
| 触发方式 | 显式调用 | cd 到项目目录 | 显式调用 |
| 环境生命周期 | 命令执行期间 | 直到离开目录 | 直到 exit |
| 项目配置 | 可选读取 | 自动读取 | 可选读取 |
| 多环境 | 不适用 | 自动切换 | 手动切换 |
| AI 使用 | ✅ 推荐 | ✅ 推荐 | 不适用 |

---

## 四、命令解析流程

```
vx <command> [args...]
     │
     ▼
┌──────────────────────────────────────────────────────────────┐
│ 1. 内置命令？ (install, list, shell, which, capabilities)   │
│    └─► 是 → 执行内置命令                                     │
├──────────────────────────────────────────────────────────────┤
│ 2. Provider Runtime？ (node, go, cargo, uv, npm...)         │
│    └─► 是 → 通过 Provider 执行（可能自动安装）               │
├──────────────────────────────────────────────────────────────┤
│ 3. 已注册系统工具？ (xcodebuild, msbuild, curl...)          │
│    └─► 是 → 设置环境后执行                                   │
├──────────────────────────────────────────────────────────────┤
│ 4. PATH 中存在？                                             │
│    └─► 是 → 直接执行                                         │
├──────────────────────────────────────────────────────────────┤
│ 5. 跨平台别名？ (copy → robocopy/cp)                        │
│    └─► 是 → 映射到平台命令后执行                             │
├──────────────────────────────────────────────────────────────┤
│ 6. 未找到 → 友好错误提示                                     │
└──────────────────────────────────────────────────────────────┘
```

---

## 五、跨平台命令映射

对于语法基本一致的命令，提供统一别名：

```toml
# vx 内置的跨平台别名
[aliases]
# 网络工具
http = { macos = "curl", linux = "curl", windows = "curl" }

# 文件操作 (语法差异大，不建议映射)
# copy = { macos = "cp", linux = "cp", windows = "robocopy" }  # ❌ 语法不同

# 构建工具
[aliases.build]
macos = "xcodebuild"
windows = "msbuild"
linux = "make"
```

### 映射原则

| 情况 | 是否映射 | 原因 |
|------|----------|------|
| `curl` (跨平台语法一致) | ✅ 直接使用 | 所有平台都有，语法相同 |
| `xcodebuild` / `msbuild` | ❌ 不映射 | 语法完全不同，映射会造成混乱 |
| `cp` / `robocopy` | ❌ 不映射 | 参数语法差异大 |
| `git`, `docker` | ✅ 直接使用 | 跨平台工具，语法一致 |

**结论**：只对语法完全一致的命令提供映射，其他保持原生命令名。

---

## 六、系统工具注册

### 6.1 工具分类

| 分类 | 说明 | 示例 |
|------|------|------|
| `build` | 构建工具 | xcodebuild, msbuild, make |
| `language` | 语言运行时 | swift, java |
| `network` | 网络工具 | curl, ssh, wget |
| `vcs` | 版本控制 | git, svn |
| `container` | 容器工具 | docker, kubectl |
| `filesystem` | 文件系统 | robocopy, rsync |
| `archive` | 压缩解压 | tar, zip, 7z |
| `security` | 安全工具 | codesign, signtool |
| `system` | 系统工具 | systemctl, sfc |
| `package` | 包管理 | apt, brew, winget |
| `cloud` | 云平台 | aws, az, gcloud |
| `mlops` | ML/AI 工具 | nvidia-smi, mlflow |

---

## 七、AI 友好设计

### 7.1 能力发现

AI 可以查询 vx 的完整能力：

```bash
$ vx capabilities --json
```

```json
{
  "version": "0.1.0",
  "platform": { "os": "macos", "arch": "arm64" },
  
  "runtimes": {
    "node": { 
      "version": "22.0.0", 
      "installed": true,
      "commands": ["node", "npm", "npx"]
    },
    "go": { 
      "version": "1.23.0", 
      "installed": true,
      "commands": ["go", "gofmt"]
    },
    "rust": {
      "version": "1.84.0",
      "installed": true,
      "commands": ["cargo", "rustc", "rustfmt", "clippy"]
    }
  },
  
  "system_tools": {
    "available": [
      { "name": "xcodebuild", "category": "build", "platform": "macos" },
      { "name": "curl", "category": "network", "platform": "universal" },
      { "name": "git", "category": "vcs", "platform": "universal" },
      { "name": "docker", "category": "container", "platform": "universal" }
    ],
    "unavailable": [
      { "name": "msbuild", "category": "build", "platform": "windows", "reason": "Windows only" }
    ]
  },
  
  "features": {
    "auto_install": true,
    "shell_mode": true,
    "project_config": true
  }
}
```

### 7.2 MCP 工具定义

```typescript
// vx 提供的 MCP 工具
const vxTools = [
  {
    name: "vx_run",
    description: "Execute any command through vx unified interface. Use this for ALL command execution.",
    inputSchema: {
      type: "object",
      properties: {
        command: { 
          type: "string", 
          description: "Command name (e.g., 'node', 'curl', 'xcodebuild')" 
        },
        args: { 
          type: "array", 
          items: { type: "string" },
          description: "Command arguments"
        },
        cwd: { 
          type: "string", 
          description: "Working directory (optional)"
        }
      },
      required: ["command"]
    }
  },
  {
    name: "vx_capabilities",
    description: "Get available tools and runtimes on this system",
    inputSchema: {
      type: "object",
      properties: {}
    }
  },
  {
    name: "vx_install",
    description: "Install a runtime or tool",
    inputSchema: {
      type: "object",
      properties: {
        name: { type: "string", description: "Runtime name (e.g., 'node', 'go')" },
        version: { type: "string", description: "Version (optional, defaults to latest)" }
      },
      required: ["name"]
    }
  }
]
```

### 7.3 AI 使用示例

```
AI: 我需要构建这个 iOS 项目

# AI 首先查询能力
$ vx capabilities --json | jq '.system_tools.available[] | select(.name=="xcodebuild")'
{
  "name": "xcodebuild",
  "category": "build",
  "platform": "macos"
}

# AI 执行构建
$ vx xcodebuild -project MyApp.xcodeproj -scheme MyApp -configuration Release build

# AI 签名
$ vx codesign --sign "Developer ID" MyApp.app
```

---

## 八、vx.toml 配置设计

与现有设计保持一致，扩展系统工具支持：

```toml
# vx.toml - 项目配置

[project]
name = "my-fullstack-app"

# 运行时版本 (已有设计)
[tools]
node = "22.0.0"
python = "3.11"
go = "1.23.0"
uv = "latest"

# 系统工具要求 (新增)
[system_tools]
required = ["docker", "kubectl", "git"]
optional = ["xcodebuild"]  # 平台特定，可选

# 行为设置 (已有设计)
[settings]
auto_install = true
cache_duration = "7d"

# 脚本定义 (已有设计)
[scripts]
dev = "npm run dev"
build = "npm run build && go build ./cmd/server"
test = "npm test && go test ./..."
deploy = "docker build -t myapp . && kubectl apply -f k8s/"

# 环境变量 (新增)
[env]
NODE_ENV = "development"
DATABASE_URL = "postgres://localhost/myapp"

# 环境特定配置 (新增)
[env.production]
NODE_ENV = "production"
DATABASE_URL = "${DATABASE_URL}"  # 从系统环境读取
```

---

## 九、CLI 命令设计

### 9.1 命令概览

```
vx - Universal Development Tool Manager

USAGE:
    vx <COMMAND> [ARGS]...
    vx <RUNTIME> [ARGS]...      # 转发到运行时
    vx <SYSTEM_TOOL> [ARGS]...  # 转发到系统工具

COMMANDS:
    # 核心命令
    install     Install a runtime
    list        List available runtimes and tools
    which       Show tool location and info
    run         Run a script defined in vx.toml
    
    # 环境命令
    shell       Enter project shell environment (manual)
    env         Manage environments
    hook        Generate shell integration script
    
    # 信息命令
    capabilities    Show available capabilities (for AI)
    help            Show help
    version         Show version

EXAMPLES:
    vx node --version           # Run node
    vx npm install              # Run npm
    vx curl https://example.com # Run system curl
    vx xcodebuild build         # Run xcodebuild with env setup
    vx shell                    # Enter project environment (manual)
    vx env list                 # List known environments
    vx run build                # Run 'build' script from vx.toml
    vx capabilities --json      # Show capabilities for AI
```

### 9.2 环境管理命令

```bash
# 查看当前环境
$ vx env
Active: my-project (auto-activated)
  Path: ~/code/my-project
  Tools:
    node: 22.0.0 (managed)
    python: 3.11 (managed)
    docker: 24.0.0 (system)

# 列出所有环境
$ vx env list
  NAME          PATH                    TOOLS
* my-project    ~/code/my-project       node@22, python@3.11
  backend       ~/code/backend          go@1.23, rust@1.84
  ml-project    ~/code/ml-project       python@3.11, cuda@12

# 切换环境
$ vx env use backend
vx: switched to 'backend'

# 临时覆盖版本
$ vx env override node@20
vx: node overridden to 20.0.0 (session only)

# 清除覆盖
$ vx env reset
vx: cleared all overrides
```

### 9.3 工具信息命令

```bash
# 查看工具信息
$ vx which node
node: ~/.vx/runtimes/node/22.0.0/bin/node
  Type: managed runtime
  Version: 22.0.0
  Provider: node

$ vx which xcodebuild
xcodebuild: /usr/bin/xcodebuild
  Type: system tool
  Category: build
  Platform: macOS
  Env: DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer

$ vx which unknown-tool
Error: 'unknown-tool' not found
  - Not a vx managed runtime
  - Not found in system PATH
  
  Did you mean: known-tool?

# 列出工具
$ vx list
Managed Runtimes:
  node     22.0.0    installed
  go       1.23.0    installed
  rust     1.84.0    installed

$ vx list --system
System Tools (available):
  xcodebuild    /usr/bin/xcodebuild     [build]
  curl          /usr/bin/curl           [network]
  git           /usr/bin/git            [vcs]
  docker        /usr/local/bin/docker   [container]
```

---

## 十、错误处理

### 10.1 友好的错误提示

```bash
# 工具不存在
$ vx unknown-command
Error: Command 'unknown-command' not found

  vx searched:
    ✗ Not a vx built-in command
    ✗ Not a registered runtime
    ✗ Not found in PATH

  Suggestions:
    - Check spelling
    - Install with: vx install <runtime>
    - Or add to PATH

# 平台不支持
$ vx xcodebuild  # on Windows
Error: 'xcodebuild' is only available on macOS

  This tool requires:
    - Platform: macOS
    - Current: Windows

  Windows alternatives:
    - msbuild (for .NET/C++ projects)

# 需要安装
$ vx node  # node not installed
Node.js is not installed. Install now? [Y/n] y
Installing node@22.0.0...
```

---

## 十一、配置文件

### 11.1 全局配置 `~/.vx/config.toml`

```toml
[defaults]
# 默认运行时版本
node = "lts"
go = "latest"

[system_tools]
# 是否启用系统工具发现
enabled = true

# 未知命令是否允许执行
allow_unknown = true

[shell]
# shell 提示符格式
prompt = "(vx:$PROJECT) "
# 自动激活
auto_activate = true

[ai]
# AI 模式：跳过确认提示
non_interactive = false
```

---

## 十二、实现计划

### Phase 1: 基础执行 (MVP)

- [ ] 命令解析和路由
- [ ] PATH 动态发现
- [ ] 基本执行转发
- [ ] `vx which` 命令
- [ ] `vx capabilities` 命令

### Phase 2: 存储与虚拟环境

- [ ] 内容寻址存储 (`~/.vx/store/`)
- [ ] 智能链接策略 (硬链接 → 软链接 → 复制)
- [ ] 虚拟环境创建 (`vx env create`)
- [ ] 环境切换与管理 (`vx env use/list/remove`)
- [ ] 存储清理 (`vx store gc`)

### Phase 3: 系统工具注册

- [ ] 平台工具注册表 (内置 TOML)
- [ ] 工具特定环境设置
- [ ] macOS 工具 (xcodebuild, codesign, swift...)
- [ ] Windows 工具 (msbuild, robocopy, signtool...)
- [ ] Linux 工具 (apt, systemctl...)
- [ ] 跨平台工具 (docker, kubectl, terraform...)

### Phase 4: 环境激活与 Shell 集成

- [ ] Shell hook 集成 (bash, zsh, fish, pwsh)
- [ ] 自动激活 (检测 vx.toml)
- [ ] PATH 优先级管理
- [ ] Shim 模式 (IDE/非交互式支持)
- [ ] 命令冲突解决

### Phase 5: DevOps/AIOps 工具

- [ ] 云平台 CLI (aws, az, gcloud)
- [ ] CI/CD 工具 (gh, act)
- [ ] 监控工具 (prometheus, grafana)
- [ ] MLOps 工具 (nvidia-smi, mlflow)

### Phase 6: AI 集成

- [ ] MCP 工具定义
- [ ] JSON 输出格式优化
- [ ] 非交互模式完善

---

## 十三、与现有设计的关系

| 现有概念 | 本 RFC 扩展 |
|----------|-------------|
| Provider | 不变，继续管理需要安装的运行时 |
| Runtime | 不变，Provider 提供的可执行工具 |
| **SystemTool** | 新增，系统已有的工具 |
| **VirtualEnvironment** | 新增，基于软链接的隔离环境 |
| **ContentStore** | 新增，内容寻址存储 (`~/.vx/store/`) |
| **EnvironmentManager** | 新增，多环境管理 |
| vx.toml | 扩展，增加 system_tools 和 env |

---

## 十四、FAQ

### Q: 为什么使用软链接而不是复制？

A: 
| 方案 | 创建速度 | 磁盘占用 | 隔离性 | 跨分区 |
|------|----------|----------|--------|--------|
| **软链接** | ~0.1s | 0 | ✅ | ✅ |
| 硬链接 | ~0.1s | 0 | ✅ | ❌ |
| 复制 | ~10s | 100% | ✅ | ✅ |

软链接是最佳平衡：
- **速度**：秒级创建环境
- **空间**：零额外占用，所有环境共享 store
- **隔离**：每个环境有独立的 bin 目录
- **兼容**：跨分区、跨文件系统都支持

### Q: Windows 上软链接有权限问题？

A: 
- Windows 10 1703+ 开发者模式下无需管理员权限
- vx 会自动检测，无权限时回退到复制模式
- 可通过配置强制使用复制：
```toml
# ~/.vx/config.toml
[storage]
link_mode = "copy"
```

### Q: vx 和直接运行命令有什么区别？

A: 
1. **环境一致性** - vx 创建干净环境，避免环境污染
2. **自动安装** - 运行时不存在时自动安装
3. **版本管理** - 项目可以锁定工具版本
4. **AI 友好** - AI 只需要知道 `vx <command>`

### Q: vx shell 和 direnv 有什么区别？

A:
- **direnv**: 只管理环境变量
- **vx**: 管理运行时版本 + 环境变量 + 系统工具

### Q: 自动激活会影响性能吗？

A: 
- Shell hook 检测 vx.toml: ~5ms
- 环境激活: ~20ms
- 总开销很小，用户无感知

### Q: 如何禁用自动激活？

A:
```toml
# ~/.vx/config.toml
[shell]
auto_activate = false
```

### Q: 多环境如何工作？

A: 每个终端会话独立，可以同时激活不同项目的环境。环境信息存储在内存中，不会相互干扰。

---

## 参考

- [Cargo 子命令](https://doc.rust-lang.org/cargo/reference/external-tools.html)
- [Git 子命令](https://git-scm.com/docs/git#_low_level_commands_plumbing)
- [Nix Shell](https://nixos.org/manual/nix/stable/command-ref/nix-shell.html)
- [direnv](https://direnv.net/)
- [mise (formerly rtx)](https://mise.jdx.dev/)
