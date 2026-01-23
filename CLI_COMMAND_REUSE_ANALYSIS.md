# CLI 命令逻辑复用分析

## 问题总结

当前 vx CLI 中存在大量重复代码，多个命令实现了相似的功能但没有复用。这导致：
1. 代码维护困难：修改一处逻辑需要在多处修改
2. 容易引入 bug：不同实现可能有不一致的行为
3. 代码冗余：增加代码体积和维护成本

## 可复用但未复用的逻辑列表

### 1. 配置解析逻辑 ⚠️ 高优先级

**问题描述**：
多个命令都需要解析 `vx.toml` 配置文件，但实现分散在不同文件中。

**涉及命令**：
- `sync.rs` - 使用 `find_vx_config` 和 `parse_vx_config`
- `setup.rs` - 封装了 `find_vx_config` 和 `parse_vx_config`（但其他命令没有使用）
- `dev/handler.rs` - 使用自己的 `parse_vx_config` 导入
- `env/handler.rs` - 使用 `setup::parse_vx_config`
- `run.rs` - 使用 `setup::parse_vx_config`
- `services.rs` - 有自己的 `parse_vx_config` 实现（与 setup 不同）
- `bundle.rs` - 直接使用 `vx_paths::find_vx_config`
- `lock.rs` - 直接使用 `vx_paths::find_vx_config`
- `install/handler.rs` - 直接使用 `vx_paths::find_vx_config`

**重复代码示例**：

```rust
// sync.rs:39-41
let config_path = find_vx_config(&current_dir)?;
let config = parse_vx_config(&config_path)?;

// dev/handler.rs:22-23
let config_path = find_vx_config(&current_dir).map_err(|e| anyhow::anyhow!("{}", e))?;
let mut config = parse_vx_config(&config_path)?;

// services.rs:100-101
let config_path = find_vx_config(&current_dir).map_err(|e| anyhow::anyhow!("{}", e))?;
let config = parse_vx_config(&config_path)?;
```

**当前问题**：
- `setup.rs` 中有 `find_vx_config` 和 `parse_vx_config` 的封装
- 但其他命令仍然直接使用 `vx_paths::find_vx_config` 或各自的实现
- `services.rs` 中的 `parse_vx_config` 返回 `VxConfig`，而 `setup.rs` 返回 `ConfigView`

**建议方案**：
1. 统一使用 `setup::find_vx_config` 和 `setup::parse_vx_config`
2. 在 `setup.rs` 或新建 `common.rs` 中导出这些函数
3. 所有命令统一导入：
   ```rust
   use crate::commands::setup::{find_vx_config, parse_vx_config, ConfigView};
   ```

---

### 2. 工具安装逻辑 ⚠️ 高优先级（当前 bug）

**问题描述**：
`sync.rs` 和 `dev/install.rs` 都需要安装工具，但实现的细节不同，且存在 bug。

**涉及命令**：
- `sync.rs` - `install_tool` 函数（第 300 行）
- `dev/install.rs` - `check_and_install_tools` 函数（第 13 行）

**重复代码对比**：

```rust
// sync.rs:306-311 (旧代码，已修复)
let mut cmd = Command::new(exe);
cmd.args(["install", name, version]);  // ❌ 错误：两个分开的参数

// dev/install.rs:93-98
let status = Command::new(env::current_exe()?)
    .args(["install", tool, version])  // ❌ 错误：两个分开的参数
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()?;

// install/handler.rs:13-19 (期望的格式)
fn parse_tool_spec(spec: &str) -> (&str, Option<&str>) {
    if let Some((tool, version)) = spec.split_once('@') {
        (tool, Some(version))
    } else {
        (spec, None)
    }
}
```

**当前问题**：
- `install` 命令期望 `tool@version` 格式的单个参数（如 `node@20`）
- `sync` 和 `dev` 传递两个分开的参数（如 `node`, `20`）
- 导致版本号被当作工具名称，触发 "Tool '20' is not supported by vx" 错误

**已修复方案**：
```rust
// sync.rs 和 dev/install.rs: 使用 tool@version 格式
cmd.args(["install", &format!("{}@{}", tool, version)]);
```

**建议方案**：
1. ✅ 已修复：修改参数格式为 `tool@version`
2. 创建统一的安装辅助函数：
   ```rust
   // 在 common.rs 或 setup.rs 中
   pub async fn install_tool_quiet(
       tool: &str,
       version: &str,
   ) -> Result<bool> {
       let exe = env::current_exe()?;
       let status = Command::new(exe)
           .args(["install", &format!("{}@{}", tool, version)])
           .stdout(Stdio::null())
           .stderr(Stdio::null())
           .status()?;
       Ok(status.success())
   }
   ```
3. `sync.rs` 和 `dev/install.rs` 都调用这个统一的函数

---

### 3. 工具状态检查逻辑 ⚠️ 中优先级

**问题描述**：
检查工具是否已安装的逻辑在多处重复。

**涉及命令**：
- `sync.rs` - `check_tool_status` 函数（第 176 行）
- `dev/install.rs` - 状态检查逻辑内联（第 40-53 行）

**重复代码对比**：

```rust
// sync.rs:176-206
fn check_tool_status(tools: &HashMap<String, String>) -> Result<Vec<ToolStatusTuple>> {
    let path_manager = PathManager::new()?;
    let mut statuses = Vec::new();

    for (name, version) in tools {
        let (installed, path) = if version == "latest" {
            let versions = path_manager.list_store_versions(name)?;
            if let Some(latest) = versions.last() {
                let store_path = path_manager.version_store_dir(name, latest);
                (true, Some(store_path))
            } else {
                (false, None)
            }
        } else {
            let store_path = path_manager.version_store_dir(name, version);
            (store_path.exists(), Some(store_path))
        };
        // ...
    }
}

// dev/install.rs:40-53
let status = if version == "latest" {
    let versions = path_manager.list_store_versions(tool)?;
    if versions.is_empty() {
        missing_tools.push((tool.clone(), version.clone()));
        ToolStatus::NotInstalled
    } else {
        ToolStatus::Installed
    }
} else if path_manager.is_version_in_store(tool, version) {
    ToolStatus::Installed
} else {
    missing_tools.push((tool.clone(), version.clone()));
    ToolStatus::NotInstalled
};
```

**建议方案**：
1. 在 `common.rs` 或 `vx-paths` 中创建统一的状态检查函数
2. 返回统一的状态枚举
   ```rust
   #[derive(Debug, Clone, PartialEq)]
   pub enum ToolStatus {
       Installed,
       NotInstalled,
       SystemFallback,
   }
   ```

---

### 4. 进度显示逻辑 ⚠️ 低优先级

**问题描述**：
安装进度显示在多处重复实现。

**涉及命令**：
- `sync.rs` - 手动打印进度
- `dev/install.rs` - 使用 `InstallProgress`
- `install/handler.rs` - 使用 `ProgressSpinner`

**建议方案**：
1. 统一使用 `InstallProgress` 进行多工具安装
2. 单工具安装使用 `ProgressSpinner`

---

### 5. 环境变量处理逻辑

**问题描述**：
`dev` 和 `env` 命令都涉及环境变量处理，可能有重复逻辑。

**涉及命令**：
- `dev/handler.rs` - 环境变量设置
- `env/handler.rs` - 环境变量导出

**需要进一步调查**：
- 查看 `dev/handler.rs` 和 `env/handler.rs` 的实现
- 识别可以提取的公共逻辑

---

## 优先级排序

### 高优先级（影响功能和 bug）
1. ✅ **工具安装逻辑** - 已修复参数格式问题
2. ⚠️ **配置解析逻辑** - 需要统一，避免不一致

### 中优先级（影响代码质量）
3. ⚠️ **工具状态检查逻辑** - 可以减少重复代码

### 低优先级（优化改进）
4. ⚠️ **进度显示逻辑** - 统一用户体验
5. ⚠️ **环境变量处理逻辑** - 需要进一步调查

## 重构建议

### 阶段 1：统一配置解析（高优先级）
1. 在 `setup.rs` 中导出配置相关函数
2. 更新所有命令使用统一的配置解析函数
3. 删除各文件中的重复实现

### 阶段 2：创建公共模块
1. 创建 `crates/vx-cli/src/commands/common/tool_manager.rs`
2. 实现统一的工具管理函数：
   - `install_tool_quiet`
   - `check_tool_status`
   - `resolve_effective_versions`

### 阶段 3：重构现有命令
1. 更新 `sync.rs` 使用公共工具管理函数
2. 更新 `dev/install.rs` 使用公共工具管理函数
3. 更新其他命令使用统一的配置解析

---

## 已修复的问题

### ✅ 工具安装参数格式 bug
- **问题**：`sync` 和 `dev` 调用 `vx install` 时使用两个分开的参数
- **修复**：改为使用 `tool@version` 格式的单个参数
- **文件**：
  - `crates/vx-cli/src/commands/sync.rs`
  - `crates/vx-cli/src/commands/dev/install.rs`

---

## 6. vx setup 和 vx dev 概念分析 ⚠️ 高优先级

### 命令职责对比

**vx setup（一次性设置）**：
- ✅ 调用 `sync` 命令安装所有工具
- ✅ 执行生命周期钩子（pre_setup, post_setup）
- ✅ 显示下一步和可用脚本列表
- ✅ 在 CI 模式下输出工具路径
- ✅ 不会进入交互式 shell
- ✅ 适用于 CI/CD 或初始项目设置

**vx dev（交互式开发环境）**：
- ✅ 检查并安装缺失的工具
- ✅ 构建开发环境（PATH、环境变量等）
- ✅ 进入交互式 shell（或执行单个命令）
- ✅ 支持继承系统 PATH（`--inherit-system`）
- ✅ 提供隔离模式和 passenv 控制
- ✅ 适用于日常开发工作流

### 当前问题：工具安装逻辑重复

**setup 的实现**（setup.rs:139-148）：
```rust
// Delegate to sync command for tool installation
sync::handle(
    registry,
    false,       // check: false - we want to install
    force,
    dry_run,
    verbose,
    no_parallel,
    false,       // no_auto_install: false - we want auto install
).await?;
```

**dev 的实现**（dev/handler.rs:60）：
```rust
if auto_install {
    check_and_install_tools(&config.tools, args.verbose).await?;
}
```

**重复的工具安装逻辑**：

| 特性 | setup (通过 sync) | dev (通过 check_and_install_tools) |
|------|---------------------|--------------------------------|
| 配置解析 | ✅ 使用 `parse_vx_config` | ✅ 使用 `parse_vx_config` |
| 状态检查 | ✅ `check_tool_status` | ✅ 内联在 `check_and_install_tools` |
| 安装方式 | ✅ `install_tool` 调用 `vx install` | ✅ 调用 `vx install`（相同逻辑） |
| 进度显示 | ✅ `InstallProgress` 或手动输出 | ✅ `InstallProgress` |
| 并行安装 | ✅ 支持 `--no-parallel` | ✅ 固定并行 |
| 参数格式 | ✅ 使用 `tool@version` 格式 | ✅ 使用 `tool@version` 格式（已修复）|

### 代码重复位置

**sync.rs** (setup 委托的命令)：
- `install_tool` 函数（第 300 行）
- `check_tool_status` 函数（第 176 行）
- `install_sequential` 函数（第 236 行）
- `install_parallel` 函数（第 269 行）

**dev/install.rs** (dev 直接使用的模块)：
- `check_and_install_tools` 函数（第 13 行）
- 内联状态检查逻辑（第 40-53 行）

### 问题分析

1. **概念混淆**：
   - `setup` 和 `dev` 都提供"安装工具"的功能
   - 但 `dev` 被定位为"进入开发环境"，而不是"安装工具"
   - 用户可能不清楚何时使用哪个命令

2. **代码重复**：
   - 两个独立实现相同的工具安装逻辑
   - 维护成本高：bug 修复需要在两处进行
   - 行为可能不一致（如进度显示、错误处理）

3. **用户体验不一致**：
   - `vx setup` 使用 `sync` 的进度显示
   - `vx dev` 使用 `InstallProgress`
   - 错误信息格式可能不同

### 建议的统一方案

#### 方案 A：dev 直接调用 sync（推荐）

**优点**：
- ✅ 完全消除代码重复
- ✅ 确保两个命令行为一致
- ✅ 简化维护：只需维护 `sync` 的逻辑

**实现**：
```rust
// dev/handler.rs:51-62
// 替换 check_and_install_tools 调用：
if !args.no_install {
    let auto_install = config
        .settings
        .get("auto_install")
        .map(|v| v == "true")
        .unwrap_or(true);

    if auto_install {
        // 调用 sync 命令，而不是 check_and_install_tools
        let (registry, context) = (/* 获取 registry 和 context */);
        crate::commands::sync::handle(
            &registry,
            false,   // check: false
            false,   // force: false
            false,   // dry_run: false
            args.verbose,
            false,   // no_parallel: false (dev 默认并行)
            false,   // no_auto_install: false
        ).await?;
    }
}
```

#### 方案 B：提取公共工具管理器

**优点**：
- ✅ 更灵活：允许 dev 使用不同的配置
- ✅ 可以处理不同的进度显示需求

**实现**：
```rust
// crates/vx-cli/src/commands/common/tool_manager.rs

pub struct ToolManager {
    registry: Arc<ProviderRegistry>,
    context: Arc<RuntimeContext>,
}

impl ToolManager {
    pub async fn install_tools(
        &self,
        tools: &HashMap<String, String>,
        options: InstallOptions,
    ) -> Result<InstallResult> {
        // 统一的安装逻辑
        // 可配置：并行、进度显示、强制重装等
    }
}

// sync.rs 和 dev/install.rs 都使用 ToolManager
```

### 推荐方案

**阶段 1：dev 调用 sync**
1. 修改 `dev/handler.rs` 移除 `check_and_install_tools` 调用
2. 改为调用 `sync::handle()`
3. 删除 `dev/install.rs` 模块（或简化为只包含辅助函数）

**阶段 2：配置和选项映射**
1. 将 `dev` 的选项映射到 `sync` 的参数
2. 处理差异（如 `--inherit-system` 只影响 dev）

**阶段 3：清理**
1. 删除 `dev/install.rs` 中的重复逻辑
2. 简化 `sync.rs` 中的函数签名（如果需要）

---

## 待办事项

- [ ] 统一配置解析逻辑（高优先级）
- [ ] **重构：dev 调用 sync 统一工具安装**（高优先级）
- [ ] 创建公共工具管理模块（中优先级）
- [ ] 统一工具状态检查逻辑（中优先级）
- [ ] 调查环境变量处理逻辑复用（低优先级）
- [ ] 添加单元测试（所有公共函数）
