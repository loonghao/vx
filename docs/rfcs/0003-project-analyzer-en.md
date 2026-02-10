# RFC 0003: Project Analyzer

## Overview

This RFC proposes the **vx-project-analyzer** crate to:

1. Analyze project dependencies and required tools
2. Detect application frameworks (Electron, Tauri, etc.)
3. Auto-sync `vx.toml` configuration
4. Ensure all dependencies are correctly installed

## Current Status (this iteration)
- Implemented: multi-language detection (Python/Node.js/Rust/Go/C++/.NET/C#), script parsing (uv run/uvx, npx, pnpm exec/run, yarn exec, bunx, python -m, dotnet), dependency state detection, missing tool detection, `vx.toml` sync suggestions, justfile parsing.
- Implemented: Electron/Tauri/React Native/Flutter framework detection with extra scripts and required tools inference.

- Future: watch mode, CI/CD integration, dependency auditing, and other advanced capabilities.

## Problem Background

### Scenario 1: New project initialization

```bash
$ git clone https://github.com/example/python-project
$ cd python-project
$ vx init
# Should automatically detect scripts and dependencies from pyproject.toml
```

### Scenario 2: Missing dependency when running scripts

```bash
$ vx run test
# Script: uv run nox -s tests
# Problem: nox is not installed
```

### Scenario 3: Sync after adding new dependency

```bash
uv add --group dev pytest  # User manually added pytest
vx sync                    # Should update vx.toml
```

### Scenario 4: Existing vx.toml needs update

```bash
# pyproject.toml added new scripts
# vx.toml should sync these changes automatically
```

## Core Design

### 1. Project analyzer architecture

```
vx-project-analyzer/
├── src/
│   ├── lib.rs
│   ├── analyzer.rs       # Core analysis engine
│   ├── script_parser.rs  # Script command parsing
│   ├── dependency.rs     # Dependency detection and management
│   ├── sync.rs           # Config sync
│   ├── installer.rs      # Dependency installation
│   ├── frameworks/       # Framework detectors
│   │   ├── mod.rs
│   │   ├── types.rs      # Framework types
│   │   ├── electron.rs   # Electron detection
│   │   └── tauri.rs      # Tauri detection
│   └── languages/        # Language analyzers
│       ├── mod.rs
│       ├── python.rs     # Python analysis
│       ├── nodejs.rs     # Node.js analysis
│       ├── rust.rs       # Rust analysis
│       └── go.rs         # Go analysis
└── tests/
```

### 2. Core data structures

```rust
/// Project analysis result
pub struct ProjectAnalysis {
    /// Project root
    pub root: PathBuf,
    /// Detected ecosystems
    pub ecosystems: Vec<Ecosystem>,
    /// Detected app frameworks (Electron, Tauri, etc.)
    pub frameworks: Vec<FrameworkInfo>,
    /// All detected dependencies
    pub dependencies: Vec<Dependency>,
    /// All detected scripts
    pub scripts: Vec<Script>,
    /// Required tools
    pub required_tools: Vec<RequiredTool>,
    /// Sync suggestions
    pub sync_actions: Vec<SyncAction>,
}

/// Framework type
pub enum ProjectFramework {
    /// Electron - JavaScript/TypeScript desktop apps
    Electron,
    /// Tauri - Rust + Web desktop apps
    Tauri,
    /// React Native - cross-platform mobile apps
    ReactNative,
    /// Flutter - cross-platform mobile/desktop apps
    Flutter,
    /// Capacitor - cross-platform mobile apps
    Capacitor,
    /// NW.js (node-webkit) - desktop apps
    NwJs,
}

/// Framework details
pub struct FrameworkInfo {
    /// Framework type
    pub framework: ProjectFramework,
    /// Framework version
    pub version: Option<String>,
    /// Config file path
    pub config_path: Option<PathBuf>,
    /// Build tool (electron-builder, tauri-cli, etc.)
    pub build_tool: Option<String>,
    /// Target platforms
    pub target_platforms: Vec<String>,
    /// Extra metadata
    pub metadata: HashMap<String, String>,
}

/// Dependency info
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub ecosystem: Ecosystem,
    pub source: DependencySource,
    pub is_dev: bool,
    pub is_installed: bool,
}

/// Dependency source
pub enum DependencySource {
    /// pyproject.toml, package.json, Cargo.toml
    ConfigFile { path: PathBuf, section: String },
    /// Detected from scripts
    Script { script_name: String, command: String },
    /// Detected from lock files
    LockFile { path: PathBuf },
}

/// Script info
pub struct Script {
    pub name: String,
    pub command: String,
    pub source: ScriptSource,
    /// Tools used by the script
    pub tools: Vec<ScriptTool>,
}

/// Tool in scripts
pub struct ScriptTool {
    pub name: String,
    pub invocation: ToolInvocation,
    pub is_available: bool,
}

/// Tool invocation method
pub enum ToolInvocation {
    /// uv run <tool>
    UvRun,
    /// uvx <tool> (temporary install)
    Uvx,
    /// npx <tool>
    Npx,
    /// python -m <module>
    PythonModule,
    /// direct
    Direct,
}

/// Required tool
pub struct RequiredTool {
    pub name: String,
    pub version: Option<String>,
    pub ecosystem: Ecosystem,
    pub reason: String,
    pub install_method: InstallMethod,
}

/// Sync action
pub enum SyncAction {
    /// Add tool to vx.toml
    AddTool { name: String, version: String },
    /// Update tool version
    UpdateTool { name: String, old_version: String, new_version: String },
    /// Add script to vx.toml
    AddScript { name: String, command: String },
    /// Update script
    UpdateScript { name: String, old_command: String, new_command: String },
    /// Install dependency
    InstallDependency { command: String, description: String },
    /// Add to project config (pyproject.toml, etc.)
    AddProjectDependency { file: PathBuf, section: String, content: String },
}
```

### 3. Language analyzer trait

```rust
/// Language/ecosystem analyzer trait
#[async_trait]
pub trait LanguageAnalyzer: Send + Sync {
    /// Detect whether the analyzer applies
    fn detect(&self, root: &Path) -> bool;

    /// Analyze dependencies
    async fn analyze_dependencies(&self, root: &Path) -> Result<Vec<Dependency>>;

    /// Analyze scripts
    async fn analyze_scripts(&self, root: &Path) -> Result<Vec<Script>>;

    /// Required tools derived from analysis
    fn required_tools(&self, deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool>;

    /// Generate install command
    fn install_command(&self, dep: &Dependency) -> Option<String>;
}
```

### 4. Python analyzer implementation

```rust
pub struct PythonAnalyzer;

impl LanguageAnalyzer for PythonAnalyzer {
    fn detect(&self, root: &Path) -> bool {
        root.join("pyproject.toml").exists()
            || root.join("setup.py").exists()
            || root.join("requirements.txt").exists()
    }

    async fn analyze_dependencies(&self, root: &Path) -> Result<Vec<Dependency>> {
        let mut deps = Vec::new();

        // Analyze pyproject.toml
        if let Ok(content) = fs::read_to_string(root.join("pyproject.toml")) {
            deps.extend(parse_pyproject_dependencies(&content)?);
        }

        // Analyze uv.lock
        if let Ok(content) = fs::read_to_string(root.join("uv.lock")) {
            deps.extend(parse_uv_lock(&content)?);
        }

        Ok(deps)
    }

    async fn analyze_scripts(&self, root: &Path) -> Result<Vec<Script>> {
        let mut scripts = Vec::new();

        // From pyproject.toml [project.scripts] and [tool.uv.scripts]
        if let Ok(content) = fs::read_to_string(root.join("pyproject.toml")) {
            scripts.extend(parse_pyproject_scripts(&content)?);
        }

        // Detect noxfile.py
        if root.join("noxfile.py").exists() {
            scripts.push(Script {
                name: "nox".to_string(),
                command: "uv run nox".to_string(),
                source: ScriptSource::Detected,
                tools: vec![ScriptTool {
                    name: "nox".to_string(),
                    invocation: ToolInvocation::UvRun,
                    is_available: false, // checked later
                }],
            });
        }

        Ok(scripts)
    }
}
```

## Core Capabilities

### 1. Project analysis (`vx analyze`)

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

🔍 Audit Findings:
  ⚠️ Missing lockfile: Project has dependencies installed but no lockfile detected. Run 'npm install', 'cargo update', or 'uv lock' to generate one.
  ℹ️ Mixed ecosystem detected: Both Node.js and Python project markers found. Ensure vx.toml specifies tools for both ecosystems if needed.
```

### 2. Config sync (`vx sync`)


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

### 3. Auto fix (`vx run` enhancements)

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

### 4. Watch mode (`vx watch`)

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

## Sync Strategy

### vx.toml sync rules

```toml
[sync]
# Whether to auto sync (default true)
enabled = true

# Source priority
sources = ["pyproject.toml", "package.json", "Cargo.toml"]

# Script sync strategy
[sync.scripts]
# Import scripts from project config
import_from_project = true
# Overwrite existing scripts
overwrite_existing = false
# Script prefix (avoid conflicts)
prefix = ""

# Tool sync strategy
[sync.tools]
# Auto-detect and add tools
auto_detect = true
# Version strategy: "exact", "minor", "major", "latest"
version_strategy = "minor"

# Dependency sync strategy
[sync.dependencies]
# Auto-install missing dependencies
auto_install = true
# Confirm before installing
confirm_install = true
```

### Sync conflict resolution

```rust
pub enum ConflictResolution {
    /// Keep value in vx.toml
    KeepLocal,
    /// Use value from project config
    UseProject,
    /// Merge (append scripts, choose latest tool version)
    Merge,
    /// Ask user
    Ask,
}
```

## Implementation Plan

### Phase 1: Core analysis engine (1 week)

- [x] Analyzer framework
- [x] Script command parser
- [x] Python analyzer
- [x] Dependency detection basics

### Phase 2: Config sync (1 week)

- [x] `vx.toml` read/write
- [x] Sync strategy (priority, overwrite policy, prefix)
- [x] Conflict resolution (KeepLocal/UseProject/Merge/Ask)
- [x] `vx sync` command enablement (SyncManager action generation/apply)

### Phase 3: Dependency installation (3 days)

- [x] Install command generation (uv/npm/cargo/go, vx tools)
- [x] Install execution & verification (SyncManager execution + vx_console confirm)
- [x] `vx run` enhancement (pre-run analysis, prompt for missing deps, interactive confirm)


### Phase 4: Multi-language support (1 week)

- [x] Node.js analyzer
- [x] Rust analyzer
- [x] Go analyzer
- [x] C++ analyzer

### Phase 5: Framework detection (in progress)

- [x] Framework detector architecture
- [x] Electron detector
- [x] Tauri detector
- [x] React Native detector (completed this iteration)
- [x] Flutter detector (completed this iteration)


### Phase 6: Advanced features (optional)

- [x] Watch mode (analyze --watch, notify-based file-change-driven re-analysis)
- [ ] CI/CD integration (analysis results + --fail-on-missing for CI gating)
- [x] Dependency audit (detect lockfiles, pinned/unpinned deps, mixed ecosystems)


## CLI Commands

```bash
# Analyze project
vx analyze [--json] [--verbose] [--watch] [--fail-on-missing]

# Sync configuration
vx sync [--dry-run] [--force] [--no-install]


# Check dependency state
vx check [--fix]

# Install all dependencies
vx install-deps [--dev] [--prod]
```

## Integration with existing commands

### vx init enhancement

```rust
pub async fn handle_init() -> Result<()> {
    // 1. Run project analysis
    let analysis = ProjectAnalyzer::new().analyze(&current_dir).await?;

    // 2. Generate vx.toml
    let config = generate_config_from_analysis(&analysis)?;

    // 3. Show detection results & suggestions
    display_analysis_results(&analysis);

    // 4. Ask whether to install missing dependencies
    if !analysis.missing_deps().is_empty() {
        if confirm("Install missing dependencies?") {
            install_missing_deps(&analysis).await?;
        }
    }

    Ok(())
}
```

### vx run enhancement

```rust
pub async fn handle_run(script_name: &str) -> Result<()> {
    let config = load_vx_config()?;
    let script = config.get_script(script_name)?;

    // Analyze script dependencies
    let analysis = analyze_script(&script);

    // Check tool availability
    for tool in &analysis.tools {
        if !tool.is_available {
            match handle_missing_tool(tool).await? {
                MissingToolAction::Install => install_tool(tool).await?,
                MissingToolAction::UseTemporary => {
                    // rewrite command to use temp install
                }
                MissingToolAction::Abort => return Err(anyhow!("Aborted")),
            }
        }
    }

    // Execute script
    execute_script(&script).await
}
```

### vx setup enhancement

```rust
pub async fn handle_setup() -> Result<()> {
    // 1. Install tools from vx.toml
    install_vx_tools().await?;

    // 2. Analyze project and install project deps
    let analysis = ProjectAnalyzer::new().analyze(&current_dir).await?;

    for action in analysis.sync_actions {
        if let SyncAction::InstallDependency { command, .. } = action {
            execute_command(&command).await?;
        }
    }

    Ok(())
}
```

## Test Plan

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

## Summary

`vx-project-analyzer` provides:

1. **Comprehensive project analysis** - multi-language, multi-ecosystem
2. **Framework detection** - Electron, Tauri, etc.
3. **Smart config sync** - keeps `vx.toml` aligned with project config
4. **Dependency management** - detect, install, verify
5. **Seamless integration** - enhances `vx init`, `vx run`, `vx setup`
6. **Extensible architecture** - easy to add new languages and frameworks

### Supported frameworks

| Framework | Detection | Features |
|-----------|-----------|----------|
| **Electron** | `electron` dependency, `electron-builder.json`, `forge.config.js` | Version detection, build tool recognition, todesktop support |
| **Tauri** | `src-tauri/` directory, `tauri.conf.json`, `@tauri-apps/cli` | v1/v2 detection, productName/identifier extraction |
| **React Native** | `react-native`/`expo` dependency, `app.json`, `android/`, `ios/` | Version detection, platform inference (android/ios/macos/windows) |
| **Flutter** | `pubspec.yaml` with `sdk: flutter`, platform folders | Platform inference (android/ios/macos/windows/linux/web), Flutter SDK detection |
| **Capacitor** | `@capacitor/core` dependency, `capacitor.config.*` | CLI version detection, platform inference (android/ios/web) |
| **NW.js** | `nw` dependency or `package.json` main HTML entry | Main-entry metadata, NW.js packaging guidance |



### Framework detection examples

```bash
# Electron project
$ vx analyze
🖥️  Detected frameworks:
    - Electron v31.3.1 (build: electron-builder)
      Config: electron-builder.json
      distribution: todesktop
      productName: ComfyUI

# Tauri project
$ vx analyze
🖥️  Detected frameworks:
    - Tauri v2.x (build: tauri-cli)
      Config: src-tauri/tauri.conf.json
      identifier: com.tauri.api
      productName: Tauri API
```
