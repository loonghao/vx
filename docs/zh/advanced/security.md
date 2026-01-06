# 安全

vx 包含多项安全特性，帮助保护您的开发环境免受供应链攻击和不可信代码执行的风险。

## 概述

现代开发工作流通常涉及：
- 从 URL 加载远程配置预设
- 项目级扩展包含自定义代码
- 第三方工具安装

vx 提供安全警告和验证机制，帮助您了解潜在风险。

## 远程预设验证

在 `vx.toml` 中使用远程预设时，vx 可以使用 SHA256 哈希验证下载内容的完整性。

### 基本用法

```toml
# vx.toml
[preset]
url = "https://example.com/presets/nodejs-dev.toml"
sha256 = "a1b2c3d4e5f6..."  # 可选但推荐
```

### 工作原理

1. **无 SHA256**：加载未验证的远程预设时，vx 会显示警告
2. **有 SHA256**：vx 验证下载内容是否与预期哈希匹配

### 安全警告

加载没有哈希验证的远程预设时，您会看到：

```
⚠️  警告: 远程预设 'https://example.com/preset.toml' 没有 SHA256 验证。
    建议添加 sha256 哈希以提高安全性。
```

### 生成 SHA256 哈希

为您的预设生成 SHA256 哈希：

::: code-group

```bash [Linux/macOS]
curl -fsSL https://example.com/preset.toml | sha256sum
```

```powershell [Windows]
(Invoke-WebRequest -Uri "https://example.com/preset.toml").Content | 
    Get-FileHash -Algorithm SHA256 | 
    Select-Object -ExpandProperty Hash
```

:::

### 最佳实践

1. **生产环境始终使用 SHA256**
2. **将预设 URL 固定到特定版本或提交**
3. **添加到项目前审查预设内容**
4. **使用可信来源**（官方仓库、已验证的组织）

## 扩展安全

vx 的扩展系统允许自定义功能，但项目级扩展可能带来安全风险。

### 扩展来源

扩展可以来自三个来源：

| 来源 | 位置 | 信任级别 |
|------|------|----------|
| **内置** | 随 vx 发布 | ✅ 可信 |
| **用户** | `~/.vx/extensions/` | ⚠️ 用户安装 |
| **项目** | `.vx/extensions/` | ⚠️ 可能不可信 |

### 安全警告

当 vx 发现项目级扩展时，会显示警告：

```
⚠️  警告: 扩展 'custom-tool' 从项目目录加载。
    来源: .vx/extensions/custom-tool
    项目级扩展可以执行任意代码。
    请只使用来自可信来源的扩展。
```

### 扩展信任模型

```
┌─────────────────────────────────────────────────────────┐
│                    扩展加载流程                          │
├─────────────────────────────────────────────────────────┤
│  1. 内置扩展（始终加载）                                 │
│     └── 随 vx 发布，完全可信                            │
│                                                         │
│  2. 用户扩展（~/.vx/extensions/）                       │
│     └── 用户安装，中等信任                              │
│                                                         │
│  3. 项目扩展（.vx/extensions/）                         │
│     └── 来自仓库，需要审查                              │
│     └── ⚠️ 加载时显示警告                               │
└─────────────────────────────────────────────────────────┘
```

### 审查项目扩展

使用带有自定义扩展的项目前：

1. **检查 `.vx/extensions/` 目录** 是否有意外文件
2. **审查扩展代码** 是否有可疑行为
3. **验证扩展来源** 是否与预期一致
4. **运行不可信项目时考虑沙箱**

### 禁用项目扩展

不加载项目级扩展运行 vx：

```bash
# 设置环境变量
VX_DISABLE_PROJECT_EXTENSIONS=1 vx node --version
```

## 可观测性

vx 包含安全相关操作的结构化日志。

### 追踪 Span

关键操作使用追踪 span 进行检测：

```rust
// Span 结构示例
vx_execute {
    runtime: "node",
    version: "20.10.0",
    args_count: 3
}
```

### 启用调试日志

```bash
# 启用调试输出
VX_LOG=debug vx node --version

# 启用追踪输出（最详细）
VX_LOG=trace vx node --version
```

### 缓存日志

解析缓存操作使用结构化字段记录：

```
DEBUG runtime="node" cache_hit=true "Resolution cache hit"
DEBUG runtime="go" cache_hit=false "Resolution cache miss"
```

## CI/CD 安全

### GitHub Actions

在 CI/CD 流水线中使用 vx：

```yaml
- name: Setup vx
  uses: loonghao/vx@v1

- name: 安装工具并验证
  run: |
    # vx 会警告未验证的预设
    vx setup
```

### 内联测试检测

vx 强制执行测试规范，要求测试放在单独的 `tests/` 目录中。CI 流水线会检查内联测试：

```yaml
- name: 检查内联测试
  run: |
    # 警告应该迁移的内联测试
    ./scripts/check-inline-tests.sh
```

## 安全检查清单

### 项目维护者

- [ ] 为远程预设使用 SHA256 验证
- [ ] 审查所有项目级扩展
- [ ] 在 `vx.toml` 中固定工具版本
- [ ] 记录扩展要求

### 贡献者

- [ ] 不要添加不可信的预设
- [ ] 提交前审查扩展代码
- [ ] 遵循测试规范（不使用内联测试）
- [ ] 为安全事件使用结构化日志

### 用户

- [ ] 运行 `vx setup` 前审查 `vx.toml`
- [ ] 检查新项目中的 `.vx/extensions/`
- [ ] 调查问题时启用调试日志
- [ ] 向维护者报告安全问题

## 报告安全问题

如果您发现 vx 中的安全漏洞：

1. **不要** 创建公开的 GitHub issue
2. 通过邮件向维护者报告安全问题
3. 包含详细的复现步骤
4. 在公开披露前给予修复时间

## 未来改进

计划中的安全增强：

- **扩展签名**：扩展作者的加密验证
- **预设缓存**：带完整性验证的本地缓存
- **审计日志**：安全事件的完整审计跟踪
- **沙箱模式**：不可信代码的隔离执行环境
