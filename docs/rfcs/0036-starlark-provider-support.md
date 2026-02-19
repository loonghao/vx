# RFC 0036: Starlark Provider Support

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2026-02-19
> **目标版本**: v0.14.0

## 摘要

引入 [Starlark](https://github.com/bazelbuild/starlark) 语言作为 Provider 的脚本配置语言，与现有的 TOML 格式并存。Starlark 是一种 Python 方言，被 Bazel、Buck2 等构建系统广泛使用，具有表达能力强、安全沙箱、易于嵌入等优点。

本 RFC 设计：
1. **混合格式支持** - 同时支持 `provider.toml` 和 `provider.star`
2. **Starlark API 设计** - 为 Provider 开发提供安全的脚本 API
3. **沙箱安全模型** - 限制文件系统、网络访问，确保安全性
4. **MSVC Provider 迁移示例** - 展示复杂 Provider 的 Starlark 实现

## 动机

### 当前 TOML 的局限性

经过对 62 个现有 Provider 的分析，以下场景 TOML 无法优雅处理：

| 场景 | TOML 表达能力 | 实际例子 |
|------|--------------|----------|
| 动态 URL 构建 | ❌ 无逻辑 | MSVC 需要根据架构/组件动态选择下载包 |
| 多步骤安装流程 | ❌ 无流程控制 | vcpkg 需要 git clone + sparse checkout |
| 复杂检测逻辑 | ❌ 无条件组合 | winget 需要检查 where + registry + env |
| 环境变量构建 | ❌ 无字符串操作 | MSVC 需要构建复杂的 INCLUDE/LIB/PATH |
| 组件存在性检查 | ❌ 无文件操作 | MSVC 需要检查 Spectre/MFC 组件是否存在 |

### Provider 复杂度分析

```
┌─────────────────────────────────────────────────────────────────┐
│                    Provider 代码行数分布                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  MSVC      ████████████████████████████████████████  1077 行    │
│  vcpkg     ███████████████████████████████████        809 行    │
│  ffmpeg    ███████████████████████████                 413 行    │
│  winget    ████████████████████                        215 行    │
│  brew      ██████████████                              139 行    │
│  docker    ████████████                                111 行    │
│  node      ████████████████████████                    ~200 行   │
│  go        ████████████████████                        ~150 行   │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│  简单 Provider (<200 行): TOML 足够          ~70%              │
│  复杂 Provider (>200 行): 需要脚本能力       ~30%              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 为什么选择 Starlark

| 特性 | Starlark | Lua | JavaScript | Python |
|------|----------|-----|------------|--------|
| 学习曲线 | ⭐⭐⭐⭐ (类 Python) | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| 安全沙箱 | ⭐⭐⭐⭐⭐ 内置 | ⭐⭐⭐ | ⭐⭐ | ⭐⭐ |
| 表达能力 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 嵌入难度 | ⭐⭐⭐⭐⭐ 简单 | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| 生态成熟度 | ⭐⭐⭐⭐⭐ Bazel/Buck2 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Rust 实现 | ⭐⭐⭐⭐⭐ starlark-rust | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |

**选择 Starlark 的理由：**
1. **Buck2 同款** - Meta 的 Buck2 使用 starlark-rust，经过大规模生产验证
2. **Python 语法** - 对开发者友好，学习成本低
3. **内置沙箱** - 无 I/O、无全局状态、无副作用，天然安全
4. **Rust 原生支持** - `starlark-rust` crate 提供完整的 Rust 实现

## 设计

### 1. 混合格式架构

#### 1.1 文件选择优先级

```
~/.vx/providers/myprovider/
├── provider.star    # 优先级 1: Starlark 脚本
├── provider.toml    # 优先级 2: TOML 配置
└── README.md
```

**加载逻辑：**

```rust
impl ProviderLoader {
    fn load_provider(path: &Path) -> Result<Box<dyn Provider>> {
        let star_path = path.join("provider.star");
        let toml_path = path.join("provider.toml");

        if star_path.exists() {
            // 优先使用 Starlark
            self.load_starlark_provider(&star_path)
        } else if toml_path.exists() {
            // 回退到 TOML
            self.load_toml_provider(&toml_path)
        } else {
            Err(anyhow!("No provider.star or provider.toml found"))
        }
    }
}
```

#### 1.2 格式对比

| 特性 | provider.toml | provider.star |
|------|--------------|---------------|
| 声明能力 | 静态配置 | 完全可编程 |
| 学习成本 | 极低 | 中等 (类 Python) |
| 适用场景 | 简单 Provider | 复杂 Provider |
| 安全性 | 无风险 | 需要沙箱 |
| 调试支持 | 无需调试 | 需要 debug 工具 |

### 2. Starlark Provider API

#### 2.1 核心 API 设计

```python
# provider.star - Starlark Provider API

# ============== 元数据 ==============

def name() -> str:
    """Provider 名称"""
    return "msvc"

def description() -> str:
    """Provider 描述"""
    return "MSVC Build Tools - Microsoft Visual C++ compiler"

def version() -> str:
    """Provider API 版本"""
    return "1.0"

def ecosystem() -> str:
    """生态系统: nodejs, python, rust, go, system, custom"""
    return "system"

def aliases() -> list[str]:
    """Runtime 别名"""
    return ["cl", "nmake"]

def supported_platforms() -> list[dict]:
    """支持的平台列表"""
    return [
        {"os": "windows", "arch": "x64"},
        {"os": "windows", "arch": "arm64"},
    ]

# ============== 版本管理 ==============

def fetch_versions(ctx: Context) -> list[dict]:
    """
    获取可用版本列表

    Args:
        ctx: 执行上下文，包含平台信息、HTTP 客户端等

    Returns:
        版本信息列表，每个版本是一个字典：
        {"version": "14.42", "lts": True, "prerelease": False}
    """
    # 可以从 GitHub API 获取
    releases = ctx.http.get_json(
        "https://api.github.com/repos/microsoft/vcpkg-tool/releases"
    )

    versions = []
    for release in releases:
        if not release.get("draft"):
            versions.append({
                "version": release["tag_name"].lstrip("v"),
                "lts": not release.get("prerelease"),
                "prerelease": release.get("prerelease", False),
            })

    return versions

# ============== 下载 URL ==============

def download_url(ctx: Context, version: str) -> str | None:
    """
    构建下载 URL

    Args:
        ctx: 执行上下文
        version: 目标版本

    Returns:
        下载 URL，如果平台不支持则返回 None
    """
    if ctx.platform.os != "windows":
        return None

    # 根据 CPU 架构选择不同的包
    arch = ctx.platform.arch  # "x64" or "arm64"

    return f"https://github.com/microsoft/vcpkg-tool/releases/download/v{version}/vcpkg-{arch}.zip"

# ============== 安装流程 ==============

def install(ctx: Context, version: str) -> dict:
    """
    安装指定版本

    Args:
        ctx: 执行上下文
        version: 目标版本

    Returns:
        安装结果：
        {"success": True, "path": "/path/to/executable"}
        或
        {"success": False, "error": "错误信息"}
    """
    install_path = ctx.paths.install_dir("msvc", version)

    # 检查是否已安装
    if ctx.fs.exists(install_path) and ctx.fs.exists(install_path + "/cl.exe"):
        return {"success": True, "path": install_path, "already_installed": True}

    # 创建安装目录
    ctx.fs.mkdir(install_path)

    # 多步骤安装
    steps = [
        lambda: download_msvc(ctx, version, install_path),
        lambda: extract_packages(ctx, install_path),
        lambda: deploy_msbuild_bridge(ctx, install_path),
        lambda: save_install_info(ctx, version, install_path),
    ]

    for i, step in enumerate(steps):
        ctx.progress(f"Step {i+1}/{len(steps)}...")
        result = step()
        if not result.get("success"):
            return result

    return {"success": True, "path": install_path}

# ============== 系统检测 ==============

def detect_system_installation(ctx: Context) -> list[dict]:
    """
    检测系统已安装的版本

    Returns:
        检测结果列表，按优先级排序
    """
    results = []

    # 方式 1: 使用 where 命令 (Windows)
    if ctx.platform.os == "windows":
        where_result = ctx.execute("where", ["cl.exe"])
        if where_result.success:
            for path in where_result.stdout.strip().split("\n"):
                if ctx.fs.exists(path):
                    version = detect_cl_version(ctx, path)
                    results.append({
                        "type": "system",
                        "path": path,
                        "version": version,
                        "priority": 100,
                    })

    # 方式 2: 检查 Visual Studio 安装
    vs_paths = [
        "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community",
        "C:\\Program Files\\Microsoft Visual Studio\\2022\\Professional",
        "C:\\Program Files\\Microsoft Visual Studio\\2022\\Enterprise",
    ]

    for vs_path in vs_paths:
        if ctx.fs.exists(vs_path):
            cl_exes = ctx.fs.glob(f"{vs_path}/VC/Tools/MSVC/*/bin/Host*/cl.exe")
            if cl_exes:
                results.append({
                    "type": "visual_studio",
                    "path": cl_exes[0],
                    "version": extract_version_from_path(cl_exes[0]),
                    "priority": 90,
                })

    # 方式 3: 检查环境变量
    if ctx.env.get("VCINSTALLDIR"):
        vc_dir = ctx.env["VCINSTALLDIR"]
        cl_exe = f"{vc_dir}/Tools/MSVC/*/bin/Host*/cl.exe"
        matches = ctx.fs.glob(cl_exe)
        if matches:
            results.append({
                "type": "env",
                "path": matches[0],
                "version": extract_version_from_path(matches[0]),
                "priority": 80,
            })

    return sorted(results, key=lambda x: x["priority"], reverse=True)

# ============== 环境变量 ==============

def prepare_environment(ctx: Context, version: str) -> dict[str, str]:
    """
    准备执行环境变量

    Args:
        ctx: 执行上下文
        version: 目标版本

    Returns:
        环境变量字典
    """
    env = {}
    install_path = ctx.paths.install_dir("msvc", version)

    # 查找 MSVC 版本目录
    tools_dirs = ctx.fs.glob(f"{install_path}/VC/Tools/MSVC/*")
    if not tools_dirs:
        return env

    msvc_version = ctx.fs.basename(tools_dirs[0])
    arch = ctx.platform.arch

    # 构建 INCLUDE
    include_paths = [
        f"{install_path}/VC/Tools/MSVC/{msvc_version}/include",
    ]

    # 添加 Windows SDK 头文件
    sdk_version = detect_windows_sdk_version(ctx)
    if sdk_version:
        for sdk_root in ["C:\\Program Files (x86)\\Windows Kits\\10", "C:\\Program Files\\Windows Kits\\10"]:
            inc_base = f"{sdk_root}/Include/{sdk_version}"
            for subdir in ["ucrt", "shared", "um", "winrt"]:
                path = f"{inc_base}/{subdir}"
                if ctx.fs.exists(path):
                    include_paths.append(path)

    env["INCLUDE"] = ";".join(include_paths)

    # 构建 LIB
    lib_paths = [
        f"{install_path}/VC/Tools/MSVC/{msvc_version}/lib/{arch}",
    ]

    if sdk_version:
        for sdk_root in ["C:\\Program Files (x86)\\Windows Kits\\10", "C:\\Program Files\\Windows Kits\\10"]:
            lib_base = f"{sdk_root}/Lib/{sdk_version}"
            for subdir in ["ucrt", "um"]:
                path = f"{lib_base}/{subdir}/{arch}"
                if ctx.fs.exists(path):
                    lib_paths.append(path)

    env["LIB"] = ";".join(lib_paths)

    # 设置 VS 发现变量
    env["VCINSTALLDIR"] = f"{install_path}/VC\\"
    env["VSCMD_VER"] = "17.0"
    env["GYP_MSVS_VERSION"] = "2022"

    return env

# ============== 验证 ==============

def verify_installation(ctx: Context, version: str) -> dict:
    """
    验证安装

    Returns:
        {"valid": True, "executable": "/path/to/cl.exe"}
        或
        {"valid": False, "errors": ["..."], "suggestions": ["..."]}
    """
    install_path = ctx.paths.install_dir("msvc", version)
    arch = ctx.platform.arch

    # 查找 cl.exe
    expected_paths = [
        f"{install_path}/VC/Tools/MSVC/{version}/bin/Host{arch}/{arch}/cl.exe",
        f"{install_path}/bin/Host{arch}/{arch}/cl.exe",
        f"{install_path}/cl.exe",
    ]

    for path in expected_paths:
        if ctx.fs.exists(path):
            return {"valid": True, "executable": path}

    # 尝试在安装目录中搜索
    cl_exes = ctx.fs.glob(f"{install_path}/**/cl.exe")
    if cl_exes:
        return {"valid": True, "executable": cl_exes[0]}

    return {
        "valid": False,
        "errors": [f"MSVC compiler (cl.exe) not found in {install_path}"],
        "suggestions": [
            "Try reinstalling: vx install msvc",
            "Ensure the installation completed successfully",
        ]
    }

# ============== 组件管理 (MSVC 特有) ==============

def check_missing_components(ctx: Context, version: str, components: list[str]) -> list[str]:
    """
    检查缺失的 MSVC 组件

    Args:
        ctx: 执行上下文
        version: MSVC 版本
        components: 请求的组件列表 (如 ["spectre", "mfc", "atl"])

    Returns:
        缺失的组件列表
    """
    install_path = ctx.paths.install_dir("msvc", version)
    arch = ctx.platform.arch
    missing = []

    # 查找 MSVC 版本目录
    tools_dirs = ctx.fs.glob(f"{install_path}/VC/Tools/MSVC/*")
    if not tools_dirs:
        return components  # 所有组件都缺失

    msvc_dir = tools_dirs[0]

    for component in components:
        if component == "spectre":
            # Spectre libs: VC/Tools/MSVC/{ver}/lib/{arch}/spectre/
            spectre_dir = f"{msvc_dir}/lib/{arch}/spectre"
            if not ctx.fs.exists(spectre_dir) or not ctx.fs.list_dir(spectre_dir):
                missing.append("spectre")

        elif component in ["mfc", "atl"]:
            # MFC/ATL: VC/Tools/MSVC/{ver}/atlmfc/
            atlmfc_dir = f"{msvc_dir}/atlmfc/include"
            if not ctx.fs.exists(atlmfc_dir):
                missing.append(component)

        elif component == "asan":
            # ASAN: VC/Tools/MSVC/{ver}/lib/{arch}/clang_rt.asan*.lib
            lib_dir = f"{msvc_dir}/lib/{arch}"
            asan_libs = ctx.fs.glob(f"{lib_dir}/clang_rt.asan*.lib")
            if not asan_libs:
                missing.append("asan")

    return missing
```

#### 2.2 Context API

```python
# Context 对象提供给 Starlark 脚本使用

class Context:
    """执行上下文，注入所有外部依赖"""

    # ========== 平台信息 ==========
    platform: Platform     # 当前平台信息
    env: dict[str, str]    # 环境变量

    # ========== 路径管理 ==========
    paths: PathProvider    # 路径管理器

    # ========== 文件系统 (沙箱限制) ==========
    fs: FileSystem         # 文件系统操作

    # ========== HTTP 客户端 (沙箱限制) ==========
    http: HttpClient       # HTTP 请求

    # ========== 命令执行 (沙箱限制) ==========
    execute: Callable      # 执行外部命令

    # ========== 进度报告 ==========
    progress: Callable     # 报告进度
    log: Callable          # 日志输出


class Platform:
    """平台信息"""
    os: str          # "windows", "macos", "linux"
    arch: str        # "x64", "arm64", "x86"

    def exe_name(self, name: str) -> str:
        """添加平台特定的可执行文件后缀"""
        if self.os == "windows":
            return f"{name}.exe"
        return name


class PathProvider:
    """路径管理器"""

    def vx_home(self) -> str:
        """vx 主目录 (~/.vx)"""
        ...

    def store_dir(self) -> str:
        """全局存储目录 (~/.vx/store)"""
        ...

    def install_dir(self, name: str, version: str) -> str:
        """指定 runtime 的安装目录"""
        ...

    def cache_dir(self) -> str:
        """缓存目录"""
        ...


class FileSystem:
    """文件系统操作 (沙箱限制)"""

    # 只允许访问安装目录和缓存目录

    def exists(self, path: str) -> bool:
        """检查文件/目录是否存在"""
        ...

    def mkdir(self, path: str) -> None:
        """创建目录"""
        ...

    def remove(self, path: str) -> None:
        """删除文件/目录"""
        ...

    def list_dir(self, path: str) -> list[str]:
        """列出目录内容"""
        ...

    def glob(self, pattern: str) -> list[str]:
        """Glob 模式匹配"""
        ...

    def read(self, path: str) -> str:
        """读取文件内容"""
        ...

    def write(self, path: str, content: str) -> None:
        """写入文件"""
        ...

    def copy(self, src: str, dst: str) -> None:
        """复制文件"""
        ...

    def rename(self, src: str, dst: str) -> None:
        """重命名/移动文件"""
        ...

    def basename(self, path: str) -> str:
        """获取文件名"""
        ...

    def dirname(self, path: str) -> str:
        """获取目录名"""
        ...

    def join(self, *paths: str) -> str:
        """连接路径"""
        ...


class HttpClient:
    """HTTP 客户端 (沙箱限制)"""

    # 只允许访问预定义的白名单域名

    def get(self, url: str) -> str:
        """GET 请求，返回响应体"""
        ...

    def get_json(self, url: str) -> dict:
        """GET 请求，解析 JSON 响应"""
        ...

    def download(self, url: str, path: str) -> None:
        """下载文件到指定路径"""
        ...
```

### 3. 沙箱安全模型

#### 3.1 Starlark 内置安全特性

Starlark 语言本身的设计就考虑了安全性：

```python
# ❌ Starlark 不支持的操作
import os          # SyntaxError: import not allowed
open("/etc/passwd")  # NameError: open not defined
eval("code")       # SyntaxError: eval not allowed
exec("code")       # SyntaxError: exec not allowed
os.system("rm -rf")  # NameError: os not defined

# ❌ 无副作用
x = [1, 2, 3]
x.append(4)  # Error: list is frozen (immutable)
```

**内置限制：**
- 无 `import` 语句
- 无文件 I/O（除非通过注入的 API）
- 无网络访问（除非通过注入的 API）
- 无全局状态
- 无副作用（所有数据结构默认不可变）
- 无无限循环（可配置超时）

#### 3.2 vx 沙箱增强

```rust
// crates/vx-starlark/src/sandbox.rs

/// Starlark 沙箱配置
pub struct SandboxConfig {
    /// 文件系统访问白名单
    pub fs_allowed_paths: Vec<PathBuf>,

    /// HTTP 请求域名白名单
    pub http_allowed_hosts: Vec<String>,

    /// 执行超时时间
    pub execution_timeout: Duration,

    /// 内存限制
    pub memory_limit: usize,

    /// 是否允许执行外部命令
    pub allow_command_execution: bool,

    /// 允许执行的命令白名单
    pub allowed_commands: Vec<String>,
}

impl SandboxConfig {
    /// 默认安全配置
    pub fn secure() -> Self {
        Self {
            fs_allowed_paths: vec![],
            http_allowed_hosts: vec![
                "api.github.com",
                "github.com",
                "nodejs.org",
                "go.dev",
                "pypi.org",
                "static.rust-lang.org",
            ],
            execution_timeout: Duration::from_secs(60),
            memory_limit: 100 * 1024 * 1024, // 100MB
            allow_command_execution: false,
            allowed_commands: vec![],
        }
    }

    /// MSVC Provider 配置（需要更宽松的权限）
    pub fn for_msvc() -> Self {
        Self {
            fs_allowed_paths: vec![
                // 允许访问 vx 安装目录
                dirs::home_dir().unwrap().join(".vx"),
                // 允许访问 Windows SDK
                PathBuf::from(r"C:\Program Files (x86)\Windows Kits"),
                PathBuf::from(r"C:\Program Files\Windows Kits"),
                // 允许访问 Visual Studio
                PathBuf::from(r"C:\Program Files\Microsoft Visual Studio"),
            ],
            http_allowed_hosts: vec![
                "api.github.com",
                "github.com",
                "aka.ms",  // Microsoft 短链接
            ],
            execution_timeout: Duration::from_secs(300), // 5 分钟
            memory_limit: 500 * 1024 * 1024, // 500MB
            allow_command_execution: true,
            allowed_commands: vec![
                "where",
                "powershell",
                "git",
            ],
        }
    }
}
```

#### 3.3 文件系统沙箱

```rust
// crates/vx-starlark/src/filesystem.rs

/// 沙箱文件系统
pub struct SandboxFileSystem {
    /// 允许访问的路径前缀
    allowed_prefixes: Vec<PathBuf>,

    /// 实际的文件系统操作
    inner: RealFileSystem,
}

impl SandboxFileSystem {
    /// 检查路径是否在白名单内
    fn check_path(&self, path: &Path) -> Result<()> {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        for prefix in &self.allowed_prefixes {
            if canonical.starts_with(prefix) {
                return Ok(());
            }
        }

        Err(anyhow!(
            "Access denied: {} is not in allowed paths. Allowed: {:?}",
            path.display(),
            self.allowed_prefixes
        ))
    }

    pub fn exists(&self, path: &str) -> Result<bool> {
        let path = Path::new(path);
        self.check_path(path)?;
        self.inner.exists(path)
    }

    pub fn read(&self, path: &str) -> Result<String> {
        let path = Path::new(path);
        self.check_path(path)?;
        self.inner.read(path)
    }

    // ... 其他方法都会先调用 check_path
}
```

#### 3.4 HTTP 沙箱

```rust
// crates/vx-starlark/src/http.rs

/// 沙箱 HTTP 客户端
pub struct SandboxHttpClient {
    /// 允许访问的域名
    allowed_hosts: Vec<String>,

    /// 实际的 HTTP 客户端
    inner: reqwest::Client,
}

impl SandboxHttpClient {
    /// 检查 URL 是否在白名单内
    fn check_url(&self, url: &str) -> Result<()> {
        let parsed = Url::parse(url)?;
        let host = parsed.host_str().ok_or_else(|| anyhow!("Invalid URL"))?;

        for allowed in &self.allowed_hosts {
            if host == allowed || host.ends_with(&format!(".{allowed}")) {
                return Ok(());
            }
        }

        Err(anyhow!(
            "HTTP access denied: {} is not in allowed hosts. Allowed: {:?}",
            host,
            self.allowed_hosts
        ))
    }

    pub async fn get(&self, url: &str) -> Result<String> {
        self.check_url(url)?;
        let response = self.inner.get(url).send().await?;
        let body = response.text().await?;
        Ok(body)
    }
}
```

### 4. MSVC Provider 完整示例

#### 4.1 provider.star

```python
# crates/vx-providers/msvc/provider.star
# MSVC Build Tools Starlark Provider

def name() -> str:
    return "msvc"

def description() -> str:
    return "MSVC Build Tools - Microsoft Visual C++ compiler and tools"

def version() -> str:
    return "1.0"

def ecosystem() -> str:
    return "system"

def aliases() -> list[str]:
    return ["cl", "nmake"]

def supported_platforms() -> list[dict]:
    return [{"os": "windows", "arch": "x64"}, {"os": "windows", "arch": "arm64"}]


# ============== 版本管理 ==============

def fetch_versions(ctx) -> list[dict]:
    """返回已知的 MSVC 版本"""
    return [
        {"version": "14.42", "lts": True},
        {"version": "14.41", "lts": True},
        {"version": "14.40", "lts": True},
        {"version": "14.39", "lts": True},
        {"version": "14.38", "lts": True},
        {"version": "14.37", "lts": True},
        {"version": "14.36", "lts": True},
        {"version": "14.35", "lts": True},
        {"version": "14.34", "lts": True},
        {"version": "14.29", "lts": False},
    ]


# ============== 系统检测 ==============

def detect_system_installation(ctx) -> list[dict]:
    """检测系统已安装的 Visual Studio"""
    results = []

    # VS 2022 路径
    vs_editions = ["Community", "Professional", "Enterprise"]
    vs_root = "C:\\Program Files\\Microsoft Visual Studio\\2022"

    for edition in vs_editions:
        vs_path = f"{vs_root}\\{edition}"
        if ctx.fs.exists(vs_path):
            # 查找 cl.exe
            cl_exes = ctx.fs.glob(f"{vs_path}\\VC\\Tools\\MSVC\\*\\bin\\Host*\\cl.exe")
            if cl_exes:
                version = _extract_version_from_path(cl_exes[0])
                results.append({
                    "type": "visual_studio_2022",
                    "path": cl_exes[0],
                    "version": version,
                    "edition": edition,
                    "priority": 100,
                })

    # 使用 where 命令
    result = ctx.execute("where", ["cl.exe"])
    if result.success:
        for path in result.stdout.strip().split("\n"):
            path = path.strip()
            if ctx.fs.exists(path) and path not in [r["path"] for r in results]:
                version = _detect_cl_version(ctx, path)
                results.append({
                    "type": "path",
                    "path": path,
                    "version": version,
                    "priority": 90,
                })

    return sorted(results, key=lambda x: x["priority"], reverse=True)


# ============== 安装流程 ==============

def install(ctx, version) -> dict:
    """安装 MSVC Build Tools"""
    install_path = ctx.paths.install_dir("msvc", version)

    # 解析请求的组件
    components = _parse_components(ctx)

    # 检查是否已安装
    if ctx.fs.exists(install_path):
        verification = verify_installation(ctx, version)
        if verification["valid"]:
            # 检查缺失的组件
            missing = check_missing_components(ctx, version, components)
            if not missing:
                return {"success": True, "path": install_path, "already_installed": True}

            # 检查是否已经尝试安装过组件
            marker = f"{install_path}/.component-install-attempted"
            if ctx.fs.exists(marker):
                ctx.log("warn", f"Components {missing} were requested but unavailable. Skipping re-installation.")
                return {"success": True, "path": install_path, "already_installed": True}

            # 清理旧的 marker 文件以强制重新提取
            ctx.fs.remove(f"{install_path}/.msvc-kit-extracted")

    # 创建安装目录
    ctx.fs.mkdir(install_path)

    # 使用 msvc-kit 安装
    ctx.progress("Downloading MSVC packages...")
    result = _install_with_msvc_kit(ctx, version, install_path, components)

    if not result["success"]:
        return result

    # 部署 MSBuild bridge
    ctx.progress("Deploying MSBuild bridge...")
    _deploy_msbuild_bridge(ctx, install_path)

    # 标记组件安装尝试
    ctx.fs.write(f"{install_path}/.component-install-attempted", version)

    return {"success": True, "path": install_path}


# ============== 环境变量 ==============

def prepare_environment(ctx, version) -> dict[str, str]:
    """准备 MSVC 编译环境"""
    env = {}
    install_path = ctx.paths.install_dir("msvc", version)

    # 查找 MSVC 版本目录
    tools_dirs = ctx.fs.glob(f"{install_path}\\VC\\Tools\\MSVC\\*")
    if not tools_dirs:
        return env

    msvc_version = ctx.fs.basename(tools_dirs[0])
    arch = ctx.platform.arch

    # 构建 INCLUDE
    include_paths = _build_include_paths(ctx, install_path, msvc_version, arch)
    if include_paths:
        env["INCLUDE"] = ";".join(include_paths)

    # 构建 LIB
    lib_paths = _build_lib_paths(ctx, install_path, msvc_version, arch)
    if lib_paths:
        env["LIB"] = ";".join(lib_paths)

    # 设置 VS 发现变量
    vc_dir = f"{install_path}\\VC"
    if ctx.fs.exists(vc_dir):
        env["VCINSTALLDIR"] = f"{vc_dir}\\"
        env["VCToolsInstallDir"] = f"{vc_dir}\\Tools\\MSVC\\{msvc_version}\\"
        env["VSCMD_VER"] = "17.0"
        env["GYP_MSVS_VERSION"] = "2022"

    # Windows SDK 版本
    sdk_version = _detect_windows_sdk_version(ctx)
    if sdk_version:
        env["WindowsSDKVersion"] = f"{sdk_version}\\"

    # vcpkg 集成
    _integrate_vcpkg(ctx, env, arch)

    return env


# ============== 验证 ==============

def verify_installation(ctx, version) -> dict:
    """验证 MSVC 安装"""
    install_path = ctx.paths.install_dir("msvc", version)
    arch = ctx.platform.arch

    # 查找 cl.exe
    expected_paths = [
        f"{install_path}\\VC\\Tools\\MSVC\\{version}\\bin\\Host{arch}\\{arch}\\cl.exe",
        f"{install_path}\\bin\\Host{arch}\\{arch}\\cl.exe",
        f"{install_path}\\cl.exe",
    ]

    for path in expected_paths:
        if ctx.fs.exists(path):
            return {"valid": True, "executable": path}

    # 搜索
    cl_exes = ctx.fs.glob(f"{install_path}\\**\\cl.exe")
    if cl_exes:
        return {"valid": True, "executable": cl_exes[0]}

    return {
        "valid": False,
        "errors": [f"MSVC compiler (cl.exe) not found in {install_path}"],
        "suggestions": ["Try reinstalling: vx install msvc"]
    }


# ============== 组件管理 ==============

def check_missing_components(ctx, version, components) -> list[str]:
    """检查缺失的组件"""
    install_path = ctx.paths.install_dir("msvc", version)
    arch = ctx.platform.arch
    missing = []

    tools_dirs = ctx.fs.glob(f"{install_path}\\VC\\Tools\\MSVC\\*")
    if not tools_dirs:
        return components

    msvc_dir = tools_dirs[0]

    for component in components:
        if component == "spectre":
            spectre_dir = f"{msvc_dir}\\lib\\{arch}\\spectre"
            if not ctx.fs.exists(spectre_dir) or not ctx.fs.list_dir(spectre_dir):
                missing.append("spectre")

        elif component in ["mfc", "atl"]:
            atlmfc_dir = f"{msvc_dir}\\atlmfc\\include"
            if not ctx.fs.exists(atlmfc_dir):
                missing.append(component)

        elif component == "asan":
            lib_dir = f"{msvc_dir}\\lib\\{arch}"
            asan_libs = ctx.fs.glob(f"{lib_dir}\\clang_rt.asan*.lib")
            if not asan_libs:
                missing.append("asan")

    return missing


# ============== 内部函数 ==============

def _parse_components(ctx) -> list[str]:
    """从配置解析请求的组件"""
    # 从 install_options 获取
    components_str = ctx.install_options.get("VX_MSVC_COMPONENTS", "")
    if not components_str:
        # 从环境变量获取
        components_str = ctx.env.get("VX_MSVC_COMPONENTS", "")

    if not components_str:
        return []

    return [c.strip() for c in components_str.split(",") if c.strip()]


def _extract_version_from_path(path) -> str:
    """从路径提取版本号"""
    # path like: .../VC/Tools/MSVC/14.42.34433/bin/...
    parts = path.replace("/", "\\").split("\\")
    for i, part in enumerate(parts):
        if part == "MSVC" and i + 1 < len(parts):
            return parts[i + 1].split(".")[0:2]  # 返回 14.42
    return "unknown"


def _detect_cl_version(ctx, cl_path) -> str:
    """通过执行 cl.exe 检测版本"""
    result = ctx.execute(cl_path, ["-Bv"])
    if result.success:
        # 解析版本号
        import re
        match = re.search(r"(\d+\.\d+\.\d+)", result.stderr)
        if match:
            return match.group(1)
    return "unknown"


def _detect_windows_sdk_version(ctx) -> str | None:
    """检测 Windows SDK 版本"""
    sdk_roots = [
        "C:\\Program Files (x86)\\Windows Kits\\10\\Include",
        "C:\\Program Files\\Windows Kits\\10\\Include",
    ]

    for sdk_root in sdk_roots:
        if ctx.fs.exists(sdk_root):
            versions = ctx.fs.list_dir(sdk_root)
            versions = [v for v in versions if v.startswith("10.0.")]
            if versions:
                return sorted(versions)[-1]

    return None


def _build_include_paths(ctx, install_path, msvc_version, arch) -> list[str]:
    """构建 INCLUDE 路径"""
    paths = []

    # MSVC 头文件
    msvc_inc = f"{install_path}\\VC\\Tools\\MSVC\\{msvc_version}\\include"
    if ctx.fs.exists(msvc_inc):
        paths.append(msvc_inc)

    # Windows SDK 头文件
    sdk_version = _detect_windows_sdk_version(ctx)
    if sdk_version:
        for sdk_root in ["C:\\Program Files (x86)\\Windows Kits\\10", "C:\\Program Files\\Windows Kits\\10"]:
            inc_base = f"{sdk_root}\\Include\\{sdk_version}"
            for subdir in ["ucrt", "shared", "um", "winrt"]:
                path = f"{inc_base}\\{subdir}"
                if ctx.fs.exists(path):
                    paths.append(path)

    return paths


def _build_lib_paths(ctx, install_path, msvc_version, arch) -> list[str]:
    """构建 LIB 路径"""
    paths = []

    # MSVC 库
    msvc_lib = f"{install_path}\\VC\\Tools\\MSVC\\{msvc_version}\\lib\\{arch}"
    if ctx.fs.exists(msvc_lib):
        paths.append(msvc_lib)

    # Windows SDK 库
    sdk_version = _detect_windows_sdk_version(ctx)
    if sdk_version:
        for sdk_root in ["C:\\Program Files (x86)\\Windows Kits\\10", "C:\\Program Files\\Windows Kits\\10"]:
            lib_base = f"{sdk_root}\\Lib\\{sdk_version}"
            for subdir in ["ucrt", "um"]:
                path = f"{lib_base}\\{subdir}\\{arch}"
                if ctx.fs.exists(path):
                    paths.append(path)

    return paths


def _deploy_msbuild_bridge(ctx, install_path) -> None:
    """部署 MSBuild bridge"""
    target = f"{install_path}\\MSBuild\\Current\\Bin\\MSBuild.exe"
    ctx.fs.mkdir(ctx.fs.dirname(target))
    # 调用 vx-bridge 部署
    # 这里简化处理，实际需要调用 Rust 的 vx_bridge 模块


def _install_with_msvc_kit(ctx, version, install_path, components) -> dict:
    """使用 msvc-kit 安装"""
    # 这里需要调用 Rust 的 msvc-kit 库
    # Starlark 无法直接调用，需要通过 ctx 提供的 API
    return ctx.install_msvc_kit(version, install_path, components)


def _integrate_vcpkg(ctx, env, arch) -> None:
    """集成 vcpkg 环境"""
    vcpkg_dir = ctx.paths.install_dir("vcpkg", "latest")
    if not ctx.fs.exists(vcpkg_dir):
        return

    triplet = f"{arch}-windows"
    installed_dir = f"{vcpkg_dir}\\installed\\{triplet}"

    if ctx.fs.exists(installed_dir):
        # 添加 include
        include_dir = f"{installed_dir}\\include"
        if ctx.fs.exists(include_dir):
            if env.get("INCLUDE"):
                env["INCLUDE"] = f"{include_dir};{env['INCLUDE']}"
            else:
                env["INCLUDE"] = include_dir

        # 添加 lib
        lib_dir = f"{installed_dir}\\lib"
        if ctx.fs.exists(lib_dir):
            if env.get("LIB"):
                env["LIB"] = f"{lib_dir};{env['LIB']}"
            else:
                env["LIB"] = lib_dir

    env["VCPKG_ROOT"] = vcpkg_dir
    env["VCPKG_DEFAULT_TRIPLET"] = triplet
```

### 5. Rust 实现

#### 5.1 Cargo.toml

```toml
# crates/vx-starlark/Cargo.toml

[package]
name = "vx-starlark"
version = "0.1.0"
edition = "2021"

[dependencies]
# Starlark runtime
starlark = { version = "0.13", features = ["trace"] }

# vx crates
vx-runtime = { path = "../vx-runtime" }
vx-paths = { path = "../vx-paths" }

# Async
tokio = { workspace = true }
async-trait = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Utilities
anyhow = { workspace = true }
tracing = { workspace = true }
```

#### 5.2 核心 Trait

```rust
// crates/vx-starlark/src/lib.rs

pub mod sandbox;
pub mod context;
pub mod provider;
pub mod api;

pub use provider::StarlarkProvider;
pub use context::StarlarkContext;
pub use sandbox::SandboxConfig;

/// Starlark Provider 加载器
pub struct StarlarkLoader {
    sandbox_config: SandboxConfig,
}

impl StarlarkLoader {
    pub fn new(sandbox_config: SandboxConfig) -> Self {
        Self { sandbox_config }
    }

    /// 加载 provider.star 文件
    pub fn load(&self, path: &Path) -> Result<StarlarkProvider> {
        let source = std::fs::read_to_string(path)?;

        // 创建 Starlark 环境
        let mut env = starlark::environment::Environment::new();

        // 注册 vx API
        self.register_api(&mut env)?;

        // 解析并执行
        let ast = starlark::syntax::parse(path.to_string_lossy(), source)?;
        let module = ast.eval(&env)?;

        // 提取函数
        StarlarkProvider::from_module(module)
    }

    fn register_api(&self, env: &mut Environment) -> Result<()> {
        // 注册 Context API
        env.add_function("ctx", self.create_context_function()?);

        // 注册辅助函数
        env.add_function("semver_compare", api::semver_compare)?;
        env.add_function("regex_match", api::regex_match)?;

        Ok(())
    }
}
```

#### 5.3 StarlarkProvider

```rust
// crates/vx-starlark/src/provider.rs

use starlark::values::Value;
use std::path::Path;

/// Starlark Provider 实现
pub struct StarlarkProvider {
    /// Provider 名称
    name: String,

    /// 解析后的 Starlark 模块
    module: starlark::environment::Module,

    /// 沙箱配置
    sandbox: SandboxConfig,
}

impl StarlarkProvider {
    /// 从 Starlark 模块创建 Provider
    pub fn from_module(module: starlark::environment::Module) -> Result<Self> {
        let name = module
            .get("name")
            .and_then(|v| v.unpack_str())
            .ok_or_else(|| anyhow!("name() function not found in provider.star"))?
            .to_string();

        Ok(Self {
            name,
            module,
            sandbox: SandboxConfig::secure(),
        })
    }

    /// 调用 Starlark 函数
    fn call_function(&self, name: &str, args: &[Value]) -> Result<Value> {
        let func = self.module
            .get(name)
            .ok_or_else(|| anyhow!("Function '{}' not found", name))?;

        func.call(args, &self.module)
    }
}

#[async_trait]
impl Runtime for StarlarkProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        self.call_function("description", &[])
            .and_then(|v| v.unpack_str().map(|s| s as &str))
            .unwrap_or("")
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let ctx_value = StarlarkContext::from_runtime_context(ctx)?;

        let result = self.call_function("fetch_versions", &[ctx_value])?;

        // 将 Starlark 列表转换为 Rust Vec<VersionInfo>
        parse_version_list(&result)
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let ctx_value = StarlarkContext::from_runtime_context(ctx)?;
        let version_value = Value::new(version);

        let result = self.call_function("install", &[ctx_value, version_value])?;

        // 解析安装结果
        parse_install_result(&result)
    }

    async fn prepare_environment(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>> {
        let ctx_value = StarlarkContext::from_runtime_context(ctx)?;
        let version_value = Value::new(version);

        let result = self.call_function("prepare_environment", &[ctx_value, version_value])?;

        // 解析环境变量字典
        parse_env_dict(&result)
    }

    // ... 其他 Runtime trait 方法
}
```

## 实现计划

### Phase 1: 基础设施 (Week 1-2)

- [ ] 创建 `vx-starlark` crate
- [ ] 集成 `starlark-rust` 依赖
- [ ] 实现基础沙箱配置
- [ ] 实现 Context API 注入
- [ ] 编写单元测试

### Phase 2: Provider 迁移 (Week 3-4)

- [ ] 实现 `StarlarkProvider` trait
- [ ] 实现 `StarlarkLoader`
- [ ] 迁移 MSVC provider 到 Starlark
- [ ] 添加混合格式支持
- [ ] 编写集成测试

### Phase 3: API 完善 (Week 5-6)

- [ ] 完善 FileSystem API
- [ ] 完善 HttpClient API
- [ ] 添加辅助函数库
- [ ] 添加调试工具 (`--debug-provider`)
- [ ] 编写文档

### Phase 4: 生态迁移 (Week 7-8)

- [ ] 迁移 vcpkg provider
- [ ] 迁移 winget provider
- [ ] 迁移 brew provider
- [ ] 更新用户文档
- [ ] 发布 v0.14.0

## 向后兼容性

1. **TOML 格式完全保留** - 所有现有 provider.toml 继续工作
2. **优先级明确** - provider.star > provider.toml
3. **迁移路径清晰** - 可渐进式迁移，无需一次性全部转换
4. **API 版本化** - provider.star 中的 `version()` 函数支持未来扩展

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Starlark 学习曲线 | 中 | 提供详细文档和示例，TOML 仍可用 |
| 沙箱绕过 | 高 | 严格审计 API，限制权限，编写安全测试 |
| 性能开销 | 低 | Starlark 执行很快，主要时间在 I/O |
| 维护复杂度 | 中 | 混合格式增加测试负担，需要 CI 覆盖 |

## 参考资料

- [Starlark Language Specification](https://github.com/bazelbuild/starlark/blob/master/spec.md)
- [starlark-rust](https://github.com/facebook/starlark-rust)
- [Buck2 Starlark API](https://buck2.build/docs/concepts/starlark/)
- [Bazel Starlark Rules](https://bazel.build/extending/rules)
