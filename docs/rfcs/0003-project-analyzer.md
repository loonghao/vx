# RFC 0003: Project Analyzer

## 概述

本 RFC 提出 **vx-project-analyzer** crate，用于：

1. 分析项目依赖和工具需求
2. 检测应用框架（Electron、Tauri 等）
3. 自动同步 `vx.toml` 配置
4. 确保所有依赖正确安装

## 问题背景

### 场景 1: 新项目初始化

```bash
$ git clone https://github.com/example/python-project
$ cd python-project
$ vx init
# 应该自动检测 pyproject.toml 中的 scripts 和依赖
```

### 场景 2: 运行脚本时缺少依赖

```bash
$ vx run test
# 脚本: uv run nox -s tests
# 问题: nox 没有安装
```

### 场景 3: 项目添加新依赖后同步

```bash
uv add --group dev pytest  # 用户手动添加了 pytest
vx sync                     # 应该更新 vx.toml
```

### 场景 4: 已有 vx.toml 需要更新

```bash
# pyproject.toml 新增了 scripts
# vx.toml 需要自动同步这些变化
```

## 核心设计

### 1. 项目分析器架构

```
vx-project-analyzer/
├── src/
│   ├── lib.rs
│   ├── analyzer.rs       # 核心分析引擎
│   ├── script_parser.rs  # 脚本命令解析
│   ├── dependency.rs     # 依赖检测和管理
│   ├── sync.rs           # 配置同步
│   ├── installer.rs      # 依赖安装
│   ├── frameworks/       # 应用框架检测器
│   │   ├── mod.rs
│   │   ├── types.rs      # 框架类型定义
│   │   ├── electron.rs   # Electron 检测
│   │   └── tauri.rs      # Tauri 检测
│   └── languages/        # 语言特定分析器
│       ├── mod.rs
│       ├── python.rs     # Python 项目分析
│       ├── nodejs.rs     # Node.js 项目分析
│       ├── rust.rs       # Rust 项目分析
│       ├── go.rs         # Go 项目分析
│       └── dotnet/       # .NET/C# 项目分析
└── tests/
```

### 2. 核心数据结构

```rust
/// 项目分析结果
pub struct ProjectAnalysis {
    /// 项目根目录
    pub root: PathBuf,
    /// 检测到的语言/生态系统
    pub ecosystems: Vec<Ecosystem>,
    /// 检测到的应用框架 (Electron, Tauri 等)
    pub frameworks: Vec<FrameworkInfo>,
    /// 所有检测到的依赖
    pub dependencies: Vec<Dependency>,
    /// 所有检测到的脚本
    pub scripts: Vec<Script>,
    /// 需要的工具
    pub required_tools: Vec<RequiredTool>,
    /// 同步建议
    pub sync_actions: Vec<SyncAction>,
}

/// 应用框架类型
pub enum ProjectFramework {
    /// Electron - JavaScript/TypeScript 桌面应用
    Electron,
    /// Tauri - Rust + Web 技术桌面应用
    Tauri,
    /// React Native - 跨平台移动应用
    ReactNative,
    /// Flutter - 跨平台移动/桌面应用
    Flutter,
    /// Capacitor - 跨平台移动应用
    Capacitor,
    /// NW.js (node-webkit) - 桌面应用
    NwJs,
}

/// 框架详细信息
pub struct FrameworkInfo {
    /// 框架类型
    pub framework: ProjectFramework,
    /// 框架版本
    pub version: Option<String>,
    /// 配置文件路径
    pub config_path: Option<PathBuf>,
    /// 构建工具 (如 electron-builder, tauri-cli)
    pub build_tool: Option<String>,
    /// 目标平台
    pub target_platforms: Vec<String>,
    /// 额外元数据
    pub metadata: HashMap<String, String>,
}

/// 依赖信息
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub ecosystem: Ecosystem,
    pub source: DependencySource,
    pub is_dev: bool,
    pub is_installed: bool,
}

/// 依赖来源
pub enum DependencySource {
    /// pyproject.toml, package.json, Cargo.toml
    ConfigFile { path: PathBuf, section: String },
    /// 从脚本中检测到
    Script { script_name: String, command: String },
    /// 从 lock 文件检测到
    LockFile { path: PathBuf },
}

/// 脚本信息
pub struct Script {
    pub name: String,
    pub command: String,
    pub source: ScriptSource,
    /// 脚本使用的工具
    pub tools: Vec<ScriptTool>,
}

/// 脚本中使用的工具
pub struct ScriptTool {
    pub name: String,
    pub invocation: ToolInvocation,
    pub is_available: bool,
}

/// 工具调用方式
pub enum ToolInvocation {
    /// uv run <tool>
    UvRun,
    /// uvx <tool> (临时安装)
    Uvx,
    /// npx <tool>
    Npx,
    /// python -m <module>
    PythonModule,
    /// 直接调用
    Direct,
}

/// 需要的工具
pub struct RequiredTool {
    pub name: String,
    pub version: Option<String>,
    pub ecosystem: Ecosystem,
    pub reason: String,
    pub install_method: InstallMethod,
}

/// 同步动作
pub enum SyncAction {
    /// 添加工具到 vx.toml
    AddTool { name: String, version: String },
    /// 更新工具版本
    UpdateTool { name: String, old_version: String, new_version: String },
    /// 添加脚本到 vx.toml
    AddScript { name: String, command: String },
    /// 更新脚本
    UpdateScript { name: String, old_command: String, new_command: String },
    /// 安装依赖
    InstallDependency { command: String, description: String },
    /// 添加到项目配置 (pyproject.toml 等)
    AddProjectDependency { file: PathBuf, section: String, content: String },
}
```

### 3. 语言分析器接口

```rust
/// 语言/生态系统分析器 trait
#[async_trait]
pub trait LanguageAnalyzer: Send + Sync {
    /// 检测此分析器是否适用于当前项目
    fn detect(&self, root: &Path) -> bool;

    /// 分析项目依赖
    async fn analyze_dependencies(&self, root: &Path) -> Result<Vec<Dependency>>;

    /// 分析项目脚本
    async fn analyze_scripts(&self, root: &Path) -> Result<Vec<Script>>;

    /// 获取需要的工具
    fn required_tools(&self, deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool>;

    /// 生成安装命令
    fn install_command(&self, dep: &Dependency) -> Option<String>;
}
```

### 4. Python 分析器实现

```rust
pub struct PythonAnalyzer;

impl LanguageAnalyzer for PythonAnalyzer {
    fn detect(&self, root: &Path) -> bool {
        root.join("pyproject.toml").exists() ||
        root.join("setup.py").exists() ||
        root.join("requirements.txt").exists()
    }

    async fn analyze_dependencies(&self, root: &Path) -> Result<Vec<Dependency>> {
        let mut deps = Vec::new();

        // 分析 pyproject.toml
        if let Ok(content) = fs::read_to_string(root.join("pyproject.toml")) {
            deps.extend(parse_pyproject_dependencies(&content)?);
        }

        // 分析 uv.lock
        if let Ok(content) = fs::read_to_string(root.join("uv.lock")) {
            deps.extend(parse_uv_lock(&content)?);
        }

        Ok(deps)
    }

    async fn analyze_scripts(&self, root: &Path) -> Result<Vec<Script>> {
        let mut scripts = Vec::new();

        // 从 pyproject.toml [project.scripts] 和 [tool.uv.scripts]
        if let Ok(content) = fs::read_to_string(root.join("pyproject.toml")) {
            scripts.extend(parse_pyproject_scripts(&content)?);
        }

        // 检测 noxfile.py
        if root.join("noxfile.py").exists() {
            scripts.push(Script {
                name: "nox".to_string(),
                command: "uv run nox".to_string(),
                source: ScriptSource::Detected,
                tools: vec![ScriptTool {
                    name: "nox".to_string(),
                    invocation: ToolInvocation::UvRun,
                    is_available: false, // 后续检查
                }],
            });
        }

        Ok(scripts)
    }
}
```

## 核心功能

### 1. 项目分析 (`vx analyze`)

```bash
$ vx analyze
📊 Project Analysis

Ecosystems: Python, Node.js

🖥️  Frameworks:
  Electron v31.0.0 (build: electron-builder)
    Config: electron-builder.json
    productName: My App

📦 Dependencies:
  Python (pyproject.toml):
    ✅ pydantic = "^2.0"
    ✅ httpx = "^0.27"
    ⚠️  nox (dev) - not installed

  Node.js (package.json):
    ✅ typescript = "^5.0"
    ✅ eslint = "^8.0"
    ✅ electron = "^31.0.0"

📜 Scripts:
  test: uv run nox -s tests
    └─ requires: nox (Python dev dependency)
  lint: uv run ruff check .
    └─ requires: ruff (Python dev dependency)
  build: npm run build
    └─ requires: typescript (Node.js dev dependency)

🔧 Required Tools:
  ✅ uv = "latest"
  ✅ node = "20"
  ⚠️  nox - missing (add to [dependency-groups.dev])
  ✅ electron-builder - Electron application packager

💡 Suggestions:
  1. Run: uv add --group dev nox
  2. Run: uv add --group dev ruff
```

### 2. 配置同步 (`vx sync`)

```bash
$ vx sync
🔄 Syncing project configuration...

Changes detected:
  + [scripts] test = "uv run nox -s tests"    (from pyproject.toml)
  + [scripts] lint = "uv run ruff check ."    (from pyproject.toml)
  ~ [tools] python = "3.12" → "3.13"          (from pyproject.toml requires-python)

Apply changes? [Y/n] y

✅ Updated vx.toml
✅ Installing missing dependencies...
   Running: uv add --group dev nox
   Running: uv sync
✅ All dependencies installed
```

### 3. 自动修复 (`vx run` 增强)

```bash
$ vx run test
ℹ Running script 'test': uv run nox -s tests

⚠️  Missing dependency: nox

Options:
  1. Install as dev dependency: uv add --group dev nox
  2. Use temporary installation: uvx nox -s tests
  3. Skip and fail

Select [1/2/3]: 1

Installing nox...
✅ Installed nox

Running: uv run nox -s tests
...
```

### 4. 监视模式 (`vx watch`)

```bash
$ vx watch
👀 Watching for project changes...

[12:34:56] pyproject.toml changed
           + Added dependency: pytest
           Syncing vx.toml...
           ✅ Updated

[12:35:10] package.json changed
           + Added script: "format": "prettier --write ."
           Syncing vx.toml...
           ✅ Updated
```

## 配置同步策略

### vx.toml 同步规则

```toml
[sync]
# 是否自动同步 (默认 true)
enabled = true

# 同步来源优先级
sources = ["pyproject.toml", "package.json", "Cargo.toml"]

# 脚本同步策略
[sync.scripts]
# 从项目配置导入脚本
import_from_project = true
# 覆盖已存在的脚本
overwrite_existing = false
# 脚本前缀 (避免冲突)
prefix = ""

# 工具同步策略
[sync.tools]
# 自动检测并添加工具
auto_detect = true
# 版本策略: "exact", "minor", "major", "latest"
version_strategy = "minor"

# 依赖同步策略
[sync.dependencies]
# 自动安装缺失的依赖
auto_install = true
# 安装前确认
confirm_install = true
```

### 同步冲突处理

```rust
pub enum ConflictResolution {
    /// 保留 vx.toml 中的值
    KeepLocal,
    /// 使用项目配置中的值
    UseProject,
    /// 合并 (脚本追加，工具取最新版本)
    Merge,
    /// 询问用户
    Ask,
}
```

## 实现计划

### Phase 1: 核心分析引擎 (1 周)

- [ ] 项目分析器框架
- [ ] 脚本命令解析器
- [ ] Python 语言分析器
- [ ] 依赖检测基础

### Phase 2: 配置同步 (1 周)

- [ ] vx.toml 读写
- [ ] 同步策略实现
- [ ] 冲突检测和解决
- [ ] `vx sync` 命令

### Phase 3: 依赖安装 (3 天)

- [ ] 安装命令生成
- [ ] 安装执行和验证
- [ ] `vx run` 增强

### Phase 4: 多语言支持 (1 周)

- [x] Node.js 分析器
- [x] Rust 分析器
- [x] Go 分析器
- [x] C++ 分析器

### Phase 5: 框架检测 (已完成)

- [x] 框架检测器架构
- [x] Electron 检测器
- [x] Tauri 检测器
- [ ] React Native 检测器
- [ ] Flutter 检测器

### Phase 6: 高级功能 (可选)

- [ ] 监视模式
- [ ] CI/CD 集成
- [ ] 依赖审计

## CLI 命令

```bash
# 分析项目
vx analyze [--json] [--verbose]

# 同步配置
vx sync [--dry-run] [--force] [--no-install]

# 检查依赖状态
vx check [--fix]

# 安装所有依赖
vx install-deps [--dev] [--prod]
```

## 与现有命令的集成

### vx init 增强

```rust
pub async fn handle_init() -> Result<()> {
    // 1. 运行项目分析
    let analysis = ProjectAnalyzer::new().analyze(&current_dir).await?;

    // 2. 生成 vx.toml
    let config = generate_config_from_analysis(&analysis)?;

    // 3. 显示检测结果和建议
    display_analysis_results(&analysis);

    // 4. 询问是否安装缺失依赖
    if !analysis.missing_deps().is_empty() {
        if confirm("Install missing dependencies?") {
            install_missing_deps(&analysis).await?;
        }
    }

    Ok(())
}
```

### vx run 增强

```rust
pub async fn handle_run(script_name: &str) -> Result<()> {
    let config = load_vx_config()?;
    let script = config.get_script(script_name)?;

    // 分析脚本依赖
    let analysis = analyze_script(&script);

    // 检查依赖是否可用
    for tool in &analysis.tools {
        if !tool.is_available {
            match handle_missing_tool(tool).await? {
                MissingToolAction::Install => install_tool(tool).await?,
                MissingToolAction::UseTemporary => {
                    // 修改命令使用临时安装
                }
                MissingToolAction::Abort => return Err(anyhow!("Aborted")),
            }
        }
    }

    // 执行脚本
    execute_script(&script).await
}
```

### vx setup 增强

```rust
pub async fn handle_setup() -> Result<()> {
    // 1. 安装 vx.toml 中的工具
    install_vx_tools().await?;

    // 2. 分析项目并安装项目依赖
    let analysis = ProjectAnalyzer::new().analyze(&current_dir).await?;

    for action in analysis.sync_actions {
        if let SyncAction::InstallDependency { command, .. } = action {
            execute_command(&command).await?;
        }
    }

    Ok(())
}
```

## 测试计划

```rust
#[rstest]
#[case("uv run nox -s tests", vec!["nox"])]
#[case("uv run pytest && uv run ruff check .", vec!["pytest", "ruff"])]
#[case("npx eslint . && npm run build", vec!["eslint"])]
fn test_script_tool_detection(#[case] script: &str, #[case] expected: Vec<&str>) {
    let analysis = analyze_script(script);
    let tools: Vec<_> = analysis.tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(tools, expected);
}

#[tokio::test]
async fn test_python_project_analysis() {
    let temp = create_test_python_project();
    let analyzer = PythonAnalyzer;

    let deps = analyzer.analyze_dependencies(temp.path()).await.unwrap();
    assert!(deps.iter().any(|d| d.name == "pytest"));

    let scripts = analyzer.analyze_scripts(temp.path()).await.unwrap();
    assert!(scripts.iter().any(|s| s.name == "test"));
}
```

## 总结

`vx-project-analyzer` 提供：

1. **全面的项目分析** - 支持多语言、多生态系统
2. **框架检测** - 识别 Electron、Tauri 等桌面/移动应用框架
3. **智能配置同步** - 自动保持 `vx.toml` 与项目配置一致
4. **依赖管理** - 检测、安装、验证依赖
5. **无缝集成** - 增强现有 `vx init`, `vx run`, `vx setup` 命令
6. **可扩展架构** - 易于添加新语言和框架支持

### 已支持的框架

| 框架 | 检测方式 | 特性 |
|------|---------|------|
| **Electron** | `electron` 依赖, `electron-builder.json`, `forge.config.js` | 版本检测, 构建工具识别, todesktop 支持 |
| **Tauri** | `src-tauri/` 目录, `tauri.conf.json`, `@tauri-apps/cli` | v1/v2 版本检测, 产品名/标识符提取 |

### 框架检测示例

```bash
# Electron 项目
$ vx analyze
🖥️  Detected frameworks:
    - Electron v31.3.1 (build: electron-builder)
      Config: electron-builder.json
      distribution: todesktop
      productName: ComfyUI

# Tauri 项目
$ vx analyze
🖥️  Detected frameworks:
    - Tauri v2.x (build: tauri-cli)
      Config: src-tauri/tauri.conf.json
      identifier: com.tauri.api
      productName: Tauri API
```
