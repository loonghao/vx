# RFC 0005: VX 测试框架 (vx-test)

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2025-12-29
> **目标版本**: v0.6.0

## 摘要

本 RFC 提议创建一个统一的测试框架 crate `vx-test`，参考 pytest 的设计理念，为 vx 项目提供：

- 类似 pytest conftest.py 的 fixtures 共享机制
- 统一的测试环境管理
- 丰富的断言宏和辅助工具
- 支持单元测试、E2E 测试、冒烟测试和覆盖测试的标记系统

## 动机

### 当前状态分析

目前 vx 项目的测试代码存在以下问题：

1. **代码重复**：多个测试文件中存在相同的辅助函数
   - `E2ETestEnv` 在 `tests/e2e_workflow_tests.rs` 中定义
   - `common/mod.rs` 在 `crates/vx-cli/tests/` 中定义
   - 各种 `vx_binary()`, `run_vx()` 函数重复实现

2. **配置文件名不一致**：
   - 部分测试检查 `vx.toml`（旧格式）
   - 部分测试检查 `vx.toml`（新格式）
   - 每次格式变更需要修改多处

3. **缺乏统一的测试分类**：
   - 无法方便地只运行单元测试或 E2E 测试
   - 无法标记冒烟测试用于快速验证

4. **Fixtures 不可复用**：
   - 每个测试模块需要重新定义 fixtures
   - 无法像 pytest 那样继承和组合 fixtures

### 行业趋势对比

| 框架 | 语言 | 特点 | 可借鉴 |
|------|------|------|--------|
| **pytest** | Python | conftest.py fixtures, markers, parametrize | fixtures 继承、markers 系统 |
| **Jest** | JavaScript | describe/it, beforeEach/afterEach, mocks | 测试组织结构 |
| **rstest** | Rust | #[fixture], #[case] parametrize | 已在使用，可扩展 |
| **nextest** | Rust | 并行测试、过滤、重试 | 测试运行策略 |
| **insta** | Rust | 快照测试 | 配置文件快照 |

### 需求分析

1. **统一的 Fixtures 系统**
   - 类似 conftest.py 的共享 fixtures
   - 支持 scope（function, module, session）
   - 支持 fixtures 依赖注入

2. **测试分类标记**
   - `#[unit]` - 单元测试
   - `#[e2e]` - 端到端测试
   - `#[smoke]` - 冒烟测试
   - `#[slow]` - 慢速测试
   - `#[network]` - 需要网络的测试

3. **丰富的断言宏**
   - `assert_vx_success!` - 验证命令成功
   - `assert_config_exists!` - 验证配置文件存在
   - `assert_tool_installed!` - 验证工具已安装
   - `assert_output_contains!` - 验证输出包含文本

4. **测试环境管理**
   - 自动创建/清理临时目录
   - 隔离的 VX_HOME 环境
   - Mock 系统支持

## 设计方案

### 完整 Crate 结构

```
crates/vx-test/
├── Cargo.toml
└── src/
    ├── lib.rs              # 公开 API 导出
    ├── fixtures/           # Fixtures 系统
    │   ├── mod.rs
    │   ├── env.rs          # 环境 fixtures (TempDir, VxHome)
    │   ├── registry.rs     # Registry fixtures
    │   ├── config.rs       # 配置 fixtures
    │   └── project.rs      # 项目 fixtures
    ├── assertions/         # 断言宏
    │   ├── mod.rs
    │   ├── command.rs      # 命令相关断言
    │   ├── file.rs         # 文件相关断言
    │   └── output.rs       # 输出相关断言
    ├── markers/            # 测试标记
    │   ├── mod.rs
    │   └── attributes.rs   # 属性宏定义
    ├── env/                # 测试环境
    │   ├── mod.rs
    │   ├── test_env.rs     # E2ETestEnv
    │   ├── mock_env.rs     # MockEnv
    │   └── isolated.rs     # 隔离环境
    ├── runners/            # 测试运行器
    │   ├── mod.rs
    │   ├── command.rs      # 命令运行
    │   └── process.rs      # 进程管理
    ├── constants.rs        # 共享常量
    └── helpers.rs          # 通用辅助函数
```

### 详细设计

#### 1. Fixtures 系统

**1.1 环境 Fixtures**

```rust
// crates/vx-test/src/fixtures/env.rs

use rstest::fixture;
use tempfile::TempDir;
use std::path::PathBuf;

/// 临时目录 fixture，测试结束后自动清理
#[fixture]
pub fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp dir")
}

/// 隔离的 VX_HOME 环境
#[fixture]
pub fn vx_home() -> VxHomeFixture {
    VxHomeFixture::new()
}

pub struct VxHomeFixture {
    dir: TempDir,
    original_home: Option<String>,
}

impl VxHomeFixture {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("Failed to create VX_HOME");
        let original_home = std::env::var("VX_HOME").ok();
        std::env::set_var("VX_HOME", dir.path());
        Self { dir, original_home }
    }

    pub fn path(&self) -> &std::path::Path {
        self.dir.path()
    }
}

impl Drop for VxHomeFixture {
    fn drop(&mut self) {
        if let Some(ref home) = self.original_home {
            std::env::set_var("VX_HOME", home);
        } else {
            std::env::remove_var("VX_HOME");
        }
    }
}
```

**1.2 Registry Fixtures**

```rust
// crates/vx-test/src/fixtures/registry.rs

use rstest::fixture;
use vx_runtime::ProviderRegistry;

/// 完整的 Provider Registry
#[fixture]
pub fn full_registry() -> ProviderRegistry {
    vx_cli::create_registry()
}

/// 空的 Registry（用于单元测试）
#[fixture]
pub fn empty_registry() -> ProviderRegistry {
    ProviderRegistry::new()
}

/// 带特定 Provider 的 Registry
pub fn registry_with_providers(providers: &[&str]) -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    for provider in providers {
        // 注册指定的 providers
    }
    registry
}
```

**1.3 配置 Fixtures**

```rust
// crates/vx-test/src/fixtures/config.rs

use rstest::fixture;
use std::path::Path;

/// 配置文件名常量
pub const CONFIG_FILE_NAME: &str = "vx.toml";
pub const CONFIG_FILE_NAME_LEGACY: &str = "vx.toml";

/// 检查配置文件是否存在（兼容新旧格式）
pub fn config_exists(dir: &Path) -> bool {
    dir.join(CONFIG_FILE_NAME).exists() || dir.join(CONFIG_FILE_NAME_LEGACY).exists()
}

/// 获取配置文件路径（优先新格式）
pub fn config_path(dir: &Path) -> Option<std::path::PathBuf> {
    let new_path = dir.join(CONFIG_FILE_NAME);
    let legacy_path = dir.join(CONFIG_FILE_NAME_LEGACY);

    if new_path.exists() {
        Some(new_path)
    } else if legacy_path.exists() {
        Some(legacy_path)
    } else {
        None
    }
}

/// 最小配置
#[fixture]
pub fn minimal_config() -> &'static str {
    r#"
[project]
name = "test-project"
"#
}

/// 完整配置
#[fixture]
pub fn full_config() -> &'static str {
    r#"
min_version = "0.6.0"

[project]
name = "test-project"
version = "1.0.0"

[tools]
node = "20"
uv = "latest"

[scripts]
test = "echo test"
build = "echo build"
"#
}
```

**1.4 项目 Fixtures**

```rust
// crates/vx-test/src/fixtures/project.rs

use rstest::fixture;
use tempfile::TempDir;
use std::fs;

/// Python 项目 fixture
#[fixture]
pub fn python_project() -> ProjectFixture {
    ProjectFixture::python()
}

/// Node.js 项目 fixture
#[fixture]
pub fn node_project() -> ProjectFixture {
    ProjectFixture::node()
}

/// Rust 项目 fixture
#[fixture]
pub fn rust_project() -> ProjectFixture {
    ProjectFixture::rust()
}

pub struct ProjectFixture {
    dir: TempDir,
    project_type: ProjectType,
}

pub enum ProjectType {
    Python,
    Node,
    Rust,
    Go,
    Cpp,
    Empty,
}

impl ProjectFixture {
    pub fn python() -> Self {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("pyproject.toml"),
            r#"
[project]
name = "test"
version = "0.1.0"
requires-python = ">=3.10"

[tool.uv.scripts]
test = "pytest"
"#,
        ).unwrap();
        Self { dir, project_type: ProjectType::Python }
    }

    pub fn node() -> Self {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"name": "test", "version": "1.0.0", "scripts": {"test": "jest"}}"#,
        ).unwrap();
        Self { dir, project_type: ProjectType::Node }
    }

    pub fn rust() -> Self {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
        ).unwrap();
        Self { dir, project_type: ProjectType::Rust }
    }

    pub fn path(&self) -> &std::path::Path {
        self.dir.path()
    }

    pub fn project_type(&self) -> &ProjectType {
        &self.project_type
    }
}
```

#### 2. 测试环境 (E2ETestEnv)

```rust
// crates/vx-test/src/env/test_env.rs

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

/// E2E 测试环境，提供隔离的 VX_HOME 和工作目录
pub struct E2ETestEnv {
    home: TempDir,
    workdir: TempDir,
}

impl E2ETestEnv {
    /// 创建新的测试环境
    pub fn new() -> Self {
        Self {
            home: TempDir::new().expect("Failed to create temp dir for home"),
            workdir: TempDir::new().expect("Failed to create temp dir for workdir"),
        }
    }

    /// 在指定工作目录创建测试环境
    pub fn with_workdir(workdir: TempDir) -> Self {
        Self {
            home: TempDir::new().expect("Failed to create temp dir for home"),
            workdir,
        }
    }

    /// 获取 VX_HOME 路径
    pub fn home(&self) -> &Path {
        self.home.path()
    }

    /// 获取工作目录路径
    pub fn workdir(&self) -> &Path {
        self.workdir.path()
    }

    /// 运行 vx 命令
    pub fn run(&self, args: &[&str]) -> Output {
        Command::new(crate::runners::vx_binary())
            .args(args)
            .env("VX_HOME", self.home.path())
            .env("VX_PROJECT_ROOT", self.workdir.path())
            .current_dir(self.workdir.path())
            .output()
            .expect("Failed to execute vx command")
    }

    /// 运行 vx 命令并期望成功
    pub fn run_success(&self, args: &[&str]) -> String {
        let output = self.run(args);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            panic!(
                "Command failed: vx {}\nstdout: {}\nstderr: {}",
                args.join(" "),
                stdout,
                stderr
            );
        }
        stdout
    }

    /// 运行 vx 命令并期望失败
    pub fn run_failure(&self, args: &[&str]) -> String {
        let output = self.run(args);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if output.status.success() {
            panic!(
                "Command should have failed: vx {}",
                args.join(" ")
            );
        }
        stderr
    }

    /// 创建配置文件（使用新格式 vx.toml）
    pub fn create_config(&self, content: &str) {
        let config_path = self.workdir.path().join("vx.toml");
        fs::write(&config_path, content).expect("Failed to create vx.toml");
    }

    /// 创建旧格式配置文件（vx.toml）
    pub fn create_legacy_config(&self, content: &str) {
        let config_path = self.workdir.path().join("vx.toml");
        fs::write(&config_path, content).expect("Failed to create vx.toml");
    }

    /// 创建文件
    pub fn create_file(&self, name: &str, content: &str) {
        let path = self.workdir.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&path, content).expect("Failed to create file");
    }

    /// 读取文件
    pub fn read_file(&self, name: &str) -> String {
        let path = self.workdir.path().join(name);
        fs::read_to_string(&path).unwrap_or_default()
    }

    /// 检查文件是否存在
    pub fn file_exists(&self, name: &str) -> bool {
        self.workdir.path().join(name).exists()
    }

    /// 检查配置文件是否存在（兼容新旧格式）
    pub fn config_exists(&self) -> bool {
        self.file_exists("vx.toml") || self.file_exists("vx.toml")
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> Option<PathBuf> {
        crate::fixtures::config::config_path(self.workdir.path())
    }

    /// 创建目录
    pub fn create_dir(&self, name: &str) {
        let path = self.workdir.path().join(name);
        fs::create_dir_all(&path).expect("Failed to create directory");
    }
}

impl Default for E2ETestEnv {
    fn default() -> Self {
        Self::new()
    }
}
```

#### 3. 断言宏

```rust
// crates/vx-test/src/assertions/mod.rs

/// 断言 vx 命令执行成功
#[macro_export]
macro_rules! assert_vx_success {
    ($output:expr) => {
        assert!(
            $output.status.success(),
            "vx command should succeed\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&$output.stdout),
            String::from_utf8_lossy(&$output.stderr)
        );
    };
    ($output:expr, $msg:expr) => {
        assert!(
            $output.status.success(),
            "{}\nstdout: {}\nstderr: {}",
            $msg,
            String::from_utf8_lossy(&$output.stdout),
            String::from_utf8_lossy(&$output.stderr)
        );
    };
}

/// 断言 vx 命令执行失败
#[macro_export]
macro_rules! assert_vx_failure {
    ($output:expr) => {
        assert!(
            !$output.status.success(),
            "vx command should fail\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&$output.stdout),
            String::from_utf8_lossy(&$output.stderr)
        );
    };
}

/// 断言配置文件存在（兼容新旧格式）
#[macro_export]
macro_rules! assert_config_exists {
    ($env:expr) => {
        assert!(
            $env.config_exists(),
            "Config file should exist (vx.toml or vx.toml)"
        );
    };
    ($path:expr) => {
        assert!(
            $crate::fixtures::config::config_exists($path),
            "Config file should exist at {:?}",
            $path
        );
    };
}

/// 断言输出包含指定文本
#[macro_export]
macro_rules! assert_output_contains {
    ($output:expr, $text:expr) => {
        let stdout = String::from_utf8_lossy(&$output.stdout);
        let stderr = String::from_utf8_lossy(&$output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        assert!(
            combined.contains($text),
            "Output should contain '{}'\nActual:\nstdout: {}\nstderr: {}",
            $text,
            stdout,
            stderr
        );
    };
}

/// 断言标准输出包含指定文本
#[macro_export]
macro_rules! assert_stdout_contains {
    ($output:expr, $text:expr) => {
        let stdout = String::from_utf8_lossy(&$output.stdout);
        assert!(
            stdout.contains($text),
            "stdout should contain '{}'\nActual: {}",
            $text,
            stdout
        );
    };
}

/// 断言标准错误包含指定文本
#[macro_export]
macro_rules! assert_stderr_contains {
    ($output:expr, $text:expr) => {
        let stderr = String::from_utf8_lossy(&$output.stderr);
        assert!(
            stderr.contains($text),
            "stderr should contain '{}'\nActual: {}",
            $text,
            stderr
        );
    };
}

/// 断言文件存在
#[macro_export]
macro_rules! assert_file_exists {
    ($path:expr) => {
        assert!($path.exists(), "File should exist: {:?}", $path);
    };
    ($env:expr, $name:expr) => {
        assert!(
            $env.file_exists($name),
            "File should exist: {}",
            $name
        );
    };
}

/// 断言文件不存在
#[macro_export]
macro_rules! assert_file_not_exists {
    ($path:expr) => {
        assert!(!$path.exists(), "File should not exist: {:?}", $path);
    };
    ($env:expr, $name:expr) => {
        assert!(
            !$env.file_exists($name),
            "File should not exist: {}",
            $name
        );
    };
}

/// 断言文件内容包含指定文本
#[macro_export]
macro_rules! assert_file_contains {
    ($path:expr, $text:expr) => {
        let content = std::fs::read_to_string($path)
            .expect(&format!("Failed to read file: {:?}", $path));
        assert!(
            content.contains($text),
            "File {:?} should contain '{}'\nActual: {}",
            $path,
            $text,
            content
        );
    };
    ($env:expr, $name:expr, $text:expr) => {
        let content = $env.read_file($name);
        assert!(
            content.contains($text),
            "File {} should contain '{}'\nActual: {}",
            $name,
            $text,
            content
        );
    };
}
```

#### 4. 跳过条件宏

```rust
// crates/vx-test/src/markers/mod.rs

/// 跳过测试如果 vx 二进制不可用
#[macro_export]
macro_rules! skip_if_no_vx {
    () => {
        if !$crate::runners::vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }
    };
}

/// 跳过测试如果网络测试被禁用
#[macro_export]
macro_rules! skip_if_no_network {
    () => {
        if !$crate::helpers::network_tests_enabled() {
            eprintln!("Skipping: network tests disabled");
            return;
        }
    };
}

/// 跳过测试如果指定工具未安装
#[macro_export]
macro_rules! skip_if_tool_not_installed {
    ($tool:expr) => {
        if !$crate::helpers::tool_installed($tool) {
            eprintln!("Skipping: {} not installed", $tool);
            return;
        }
    };
}

/// 跳过测试如果在 CI 环境
#[macro_export]
macro_rules! skip_if_ci {
    () => {
        if std::env::var("CI").map(|v| v == "true").unwrap_or(false) {
            eprintln!("Skipping: running in CI environment");
            return;
        }
    };
}

/// 跳过测试如果不在 CI 环境
#[macro_export]
macro_rules! skip_if_not_ci {
    () => {
        if !std::env::var("CI").map(|v| v == "true").unwrap_or(false) {
            eprintln!("Skipping: not running in CI environment");
            return;
        }
    };
}
```

#### 5. 命令运行器

```rust
// crates/vx-test/src/runners/command.rs

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 获取 vx 二进制名称
pub fn binary_name() -> &'static str {
    if cfg!(windows) {
        "vx.exe"
    } else {
        "vx"
    }
}

/// 获取 vx 二进制路径
pub fn vx_binary() -> PathBuf {
    // 1. 检查 VX_BINARY 环境变量
    if let Ok(path) = std::env::var("VX_BINARY") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return p;
        }
    }

    let cargo_target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target"));

    // 2. 检查 release 构建
    let release_binary = cargo_target.join("release").join(binary_name());
    if release_binary.exists() {
        return release_binary;
    }

    // 3. 检查 debug 构建
    let debug_binary = cargo_target.join("debug").join(binary_name());
    if debug_binary.exists() {
        return debug_binary;
    }

    // 4. 回退到系统 PATH
    PathBuf::from(binary_name())
}

/// 检查 vx 二进制是否可用
pub fn vx_available() -> bool {
    vx_binary().exists() || Command::new("vx").arg("--version").output().is_ok()
}

/// 运行 vx 命令
pub fn run_vx(args: &[&str]) -> std::io::Result<Output> {
    Command::new(vx_binary()).args(args).output()
}

/// 在指定目录运行 vx 命令
pub fn run_vx_in_dir(dir: &Path, args: &[&str]) -> std::io::Result<Output> {
    Command::new(vx_binary())
        .args(args)
        .current_dir(dir)
        .output()
}

/// 运行 vx 命令并设置环境变量
pub fn run_vx_with_env(args: &[&str], env: &[(&str, &str)]) -> std::io::Result<Output> {
    let mut cmd = Command::new(vx_binary());
    cmd.args(args);
    for (key, value) in env {
        cmd.env(key, value);
    }
    cmd.output()
}
```

#### 6. 辅助函数

```rust
// crates/vx-test/src/helpers.rs

use std::process::Output;

/// 检查网络测试是否启用
pub fn network_tests_enabled() -> bool {
    // 显式启用
    if std::env::var("VX_NETWORK_TESTS")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }

    // CI 环境
    let has_token = std::env::var("GITHUB_TOKEN").is_ok() || std::env::var("GH_TOKEN").is_ok();
    let is_ci = std::env::var("CI").map(|v| v == "true").unwrap_or(false);

    has_token || is_ci
}

/// 检查工具是否已安装
pub fn tool_installed(tool: &str) -> bool {
    if !crate::runners::vx_available() {
        return false;
    }
    crate::runners::run_vx(&["which", tool])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 获取标准输出字符串
pub fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// 获取标准错误字符串
pub fn stderr_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// 获取组合输出
pub fn combined_output(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        stdout_str(output),
        stderr_str(output)
    )
}

/// 检查输出是否成功
pub fn is_success(output: &Output) -> bool {
    output.status.success()
}

/// 获取退出码
pub fn exit_code(output: &Output) -> Option<i32> {
    output.status.code()
}
```

#### 7. 常量定义

```rust
// crates/vx-test/src/constants.rs

/// 配置文件名（新格式）
pub const CONFIG_FILE_NAME: &str = "vx.toml";

/// 配置文件名（旧格式）
pub const CONFIG_FILE_NAME_LEGACY: &str = "vx.toml";

/// 支持的运行时工具
pub const SUPPORTED_RUNTIMES: &[&str] = &["node", "go", "cargo", "uv", "bun"];

/// 支持的包管理器
pub const SUPPORTED_PACKAGE_MANAGERS: &[&str] = &["npm", "pnpm", "yarn"];

/// 测试超时时间（秒）
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// 网络测试超时时间（秒）
pub const NETWORK_TIMEOUT_SECS: u64 = 60;
```

### 使用示例

#### 基本使用

```rust
// tests/e2e_workflow_tests.rs
use vx_test::prelude::*;

#[test]
fn test_workflow_init() {
    skip_if_no_vx!();

    let env = E2ETestEnv::new();
    let output = env.run(&["init"]);

    assert_vx_success!(output);
    assert_config_exists!(env);
}

#[rstest]
#[tokio::test]
async fn test_with_registry(full_registry: ProviderRegistry) {
    let runtime = full_registry.get_runtime("node");
    assert!(runtime.is_some());
}
```

#### 使用项目 Fixtures

```rust
use vx_test::prelude::*;

#[rstest]
#[tokio::test]
async fn test_python_project_detection(python_project: ProjectFixture) {
    let analyzer = ProjectAnalyzer::new(Default::default());
    let analysis = analyzer.analyze(python_project.path()).await.unwrap();

    assert!(analysis.ecosystems.contains(&Ecosystem::Python));
}
```

#### 参数化测试

```rust
use vx_test::prelude::*;

#[rstest]
#[case("node", "20")]
#[case("go", "1.21")]
#[case("uv", "latest")]
fn test_tool_version_parsing(#[case] tool: &str, #[case] version: &str) {
    skip_if_no_vx!();

    let env = E2ETestEnv::new();
    env.create_config(&format!(r#"
[tools]
{} = "{}"
"#, tool, version));

    assert_config_exists!(env);
}
```

### 公开 API

```rust
// crates/vx-test/src/lib.rs

pub mod assertions;
pub mod constants;
pub mod env;
pub mod fixtures;
pub mod helpers;
pub mod markers;
pub mod runners;

/// Prelude 模块，方便导入常用项
pub mod prelude {
    // Fixtures
    pub use crate::fixtures::config::{config_exists, config_path};
    pub use crate::fixtures::env::{temp_dir, vx_home, VxHomeFixture};
    pub use crate::fixtures::project::{node_project, python_project, rust_project, ProjectFixture};
    pub use crate::fixtures::registry::{empty_registry, full_registry};

    // 测试环境
    pub use crate::env::test_env::E2ETestEnv;

    // 运行器
    pub use crate::runners::{run_vx, run_vx_in_dir, run_vx_with_env, vx_available, vx_binary};

    // 辅助函数
    pub use crate::helpers::{
        combined_output, exit_code, is_success, network_tests_enabled, stderr_str, stdout_str,
        tool_installed,
    };

    // 常量
    pub use crate::constants::*;

    // 断言宏
    pub use crate::{
        assert_config_exists, assert_file_contains, assert_file_exists, assert_file_not_exists,
        assert_output_contains, assert_stderr_contains, assert_stdout_contains, assert_vx_failure,
        assert_vx_success,
    };

    // 跳过宏
    pub use crate::{
        skip_if_ci, skip_if_no_network, skip_if_no_vx, skip_if_not_ci, skip_if_tool_not_installed,
    };

    // Re-export rstest
    pub use rstest::{fixture, rstest};
}
```

### Cargo.toml

```toml
[package]
name = "vx-test"
version = "0.6.0"
edition = "2021"
description = "Testing framework for vx project"
license = "MIT"
publish = false

[dependencies]
# 测试框架
rstest = "0.24"
tempfile = "3"

# vx 内部依赖
vx-paths = { path = "../vx-paths" }
vx-runtime = { path = "../vx-runtime" }
vx-cli = { path = "../vx-cli" }

# 可选依赖
tokio = { workspace = true, optional = true }
insta = { version = "1", optional = true }

[features]
default = []
async = ["tokio"]
snapshot = ["insta"]
full = ["async", "snapshot"]
```

## 向后兼容性

### 兼容策略

1. **渐进迁移**：现有测试可以逐步迁移到新框架
2. **保留旧代码**：`common/mod.rs` 可以暂时保留
3. **重导出**：`vx-test` 可以重导出 `rstest` 等依赖

### 迁移路径

```bash
# 阶段 1: 添加 vx-test 依赖
# Cargo.toml
[dev-dependencies]
vx-test = { path = "../crates/vx-test" }

# 阶段 2: 逐步替换导入
# 旧代码
use crate::common::{E2ETestEnv, run_vx};

# 新代码
use vx_test::prelude::*;

# 阶段 3: 使用新断言宏
# 旧代码
assert!(output.status.success());

# 新代码
assert_vx_success!(output);
```

## 实现计划

### Phase 1: 核心框架 (v0.6.0)

- [ ] 创建 `vx-test` crate 基础结构
- [ ] 实现 `E2ETestEnv`
- [ ] 实现基础断言宏
- [ ] 实现跳过条件宏
- [ ] 实现命令运行器
- [ ] 添加配置文件常量

### Phase 2: Fixtures 系统 (v0.6.1)

- [ ] 实现环境 fixtures
- [ ] 实现 registry fixtures
- [ ] 实现配置 fixtures
- [ ] 实现项目 fixtures
- [ ] 文档和示例

### Phase 3: 高级功能 (v0.7.0)

- [ ] 快照测试支持 (insta)
- [ ] 测试覆盖率集成
- [ ] 并行测试优化
- [ ] 测试报告生成

### Phase 4: 迁移现有测试 (v0.7.x)

- [ ] 迁移 `tests/` 目录测试
- [ ] 迁移 `crates/vx-cli/tests/` 测试
- [ ] 迁移其他 crate 测试
- [ ] 删除旧的 `common/mod.rs`

## 参考资料

- [pytest documentation](https://docs.pytest.org/)
- [rstest crate](https://docs.rs/rstest/)
- [insta snapshot testing](https://insta.rs/)
- [nextest](https://nexte.st/)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2025-12-29 | Draft | 初始草案 |
