# Release Guide - 使用 Release Please 自动发布

本项目使用 [Release Please](https://github.com/googleapis/release-please) 实现自动化版本管理和发布，遵循 Rust 生态系统最佳实践。

## 🎯 核心概念

### Conventional Commits

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范来自动生成版本号和变更日志：

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### 版本升级规则

- `feat:` → 升级 **minor** 版本 (0.1.0 → 0.2.0)
- `fix:` → 升级 **patch** 版本 (0.1.0 → 0.1.1)
- `feat!:` 或 `BREAKING CHANGE:` → 升级 **major** 版本 (0.1.0 → 1.0.0)

## 🚀 发布流程

### 1. 开发阶段

按照 Conventional Commits 规范提交代码：

```bash
# 新功能
git commit -m "feat: add support for Python version management"

# 修复bug
git commit -m "fix: resolve installation path issue on Windows"

# 破坏性变更
git commit -m "feat!: redesign plugin API for better extensibility"

# 带作用域的提交
git commit -m "feat(uv): add support for UV 0.6.0"
git commit -m "fix(ci): resolve build failure on macOS"
```

### 2. 自动发布流程

当代码推送到 `main` 分支时，Release Please 会：

1. **分析提交历史** - 检查自上次发布以来的所有提交
2. **计算新版本** - 根据 Conventional Commits 确定版本号
3. **创建 Release PR** - 自动创建包含以下内容的 PR：
   - 更新 `Cargo.toml` 中的版本号
   - 更新 `CHANGELOG.md`
   - 更新 `.release-please-manifest.json`

4. **合并后自动发布** - PR 合并后自动：
   - 创建 Git 标签
   - 创建 GitHub Release
   - 触发 GoReleaser 构建多平台二进制文件

### 3. 手动干预（可选）

如果需要手动调整：

```bash
# 查看 Release Please 会做什么（不实际执行）
npx release-please release-pr --dry-run

# 手动创建 Release PR
npx release-please release-pr

# 手动创建 Release
npx release-please github-release
```

## 📋 提交类型说明

| 类型 | 描述 | 版本影响 | 示例 |
|------|------|----------|------|
| `feat` | 新功能 | minor | `feat: add Docker plugin support` |
| `fix` | Bug修复 | patch | `fix: resolve memory leak in installer` |
| `perf` | 性能优化 | patch | `perf: optimize plugin loading speed` |
| `docs` | 文档更新 | patch | `docs: update installation guide` |
| `style` | 代码格式 | 无 | `style: fix clippy warnings` |
| `refactor` | 重构 | patch | `refactor: simplify plugin architecture` |
| `test` | 测试相关 | 无 | `test: add integration tests for UV plugin` |
| `chore` | 构建/工具 | 无 | `chore: update dependencies` |
| `ci` | CI配置 | 无 | `ci: add security audit to workflow` |
| `revert` | 回滚提交 | patch | `revert: "feat: add experimental feature"` |

## 🔧 配置文件说明

### `release-please-config.json`

主配置文件，定义：

- 发布类型（rust）
- 变更日志格式
- 版本升级规则
- 额外文件更新

### `.release-please-manifest.json`

版本清单文件，记录当前版本号

### `CHANGELOG.md`

自动生成的变更日志

## 🎨 最佳实践

### 1. 提交信息规范

```bash
# ✅ 好的提交信息
feat(uv): add support for virtual environments
fix(installer): handle network timeout gracefully
docs: add troubleshooting section to README

# ❌ 不好的提交信息
update code
fix bug
add feature
```

### 2. 作用域使用

常用作用域：

- `uv` - UV插件相关
- `node` - Node.js插件相关
- `go` - Go插件相关
- `rust` - Rust插件相关
- `cli` - 命令行界面
- `config` - 配置相关
- `installer` - 安装器相关
- `ci` - CI/CD相关
- `docs` - 文档相关

### 3. 破坏性变更

```bash
# 方式1：使用感叹号
feat!: redesign plugin API

# 方式2：在footer中说明
feat: redesign plugin API

BREAKING CHANGE: Plugin interface has changed, see migration guide
```

### 4. 多行提交信息

```bash
feat: add plugin marketplace support

This commit introduces a new plugin marketplace that allows users to:
- Browse available plugins
- Install plugins from remote repositories
- Manage plugin dependencies

Closes #123
```

## 🔍 监控和调试

### 查看 Release Please 状态

```bash
# 检查配置是否正确
npx release-please config-check

# 查看下一个版本号
npx release-please suggest-version

# 查看变更日志预览
npx release-please changelog
```

### 常见问题

**Q: Release Please 没有创建 PR？**
A: 检查是否有符合规范的提交，确保提交类型正确

**Q: 版本号不对？**
A: 检查 `.release-please-manifest.json` 中的当前版本

**Q: 想要跳过某个提交？**
A: 在提交信息中添加 `[skip ci]` 或使用 `chore:` 类型

## 📚 参考资源

- [Release Please 官方文档](https://github.com/googleapis/release-please)
- [Conventional Commits 规范](https://www.conventionalcommits.org/)
- [语义化版本控制](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
