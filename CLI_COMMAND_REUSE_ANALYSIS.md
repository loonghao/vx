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

## 待办事项

- [ ] 统一配置解析逻辑（高优先级）
- [ ] 创建公共工具管理模块（中优先级）
- [ ] 统一工具状态检查逻辑（中优先级）
- [ ] 调查环境变量处理逻辑复用（低优先级）
- [ ] 添加单元测试（所有公共函数）
