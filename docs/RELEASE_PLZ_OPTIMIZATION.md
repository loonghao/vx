# Release-plz 配置优化指南

## 🎯 优化目标

基于 [release-plz 官方文档](https://release-plz.ieni.dev/docs/config)，我们对 `release-plz.toml` 配置进行了全面优化，以实现：

1. **智能发布触发**：只在有意义的提交时创建发布
2. **清晰的变更日志**：结构化、易读的变更记录
3. **自动化流程**：减少手动干预，提高发布效率
4. **用户友好**：提供完整的安装指南和链接

## 📋 主要优化内容

### 1. 工作空间配置优化

#### 智能发布触发

```toml
# 只在发现 conventional commits 时创建发布
release_commits = "^(feat|fix|docs|style|refactor|perf|test|chore|build|ci)[(:]"

# 添加 PR 标签便于管理
pr_labels = ["release", "automated"]

# 自定义 PR 名称模板
pr_name = "chore: release{% if package and version %} {{ package }} v{{ version }}{% endif %}"
```

#### 依赖管理优化

```toml
# 跳过需要 registry 访问的依赖检查
dependencies_update = false

# 基于 git 历史而非 registry 状态生成发布
release_always = true
```

### 2. 变更日志配置增强

#### 提交消息预处理

```toml
commit_preprocessors = [
    # 自动链接 PR 和 issue
    { pattern = "\\(#([0-9]+)\\)", replace = "([#${1}](https://github.com/loonghao/vx/pull/${1}))" },
    # 移除签名行
    { pattern = "\\n\\nSigned-off-by: .*", replace = "" },
    # 清理合并提交消息
    { pattern = "Merge pull request #([0-9]+) from [^\\n]+\\n\\n", replace = "" },
    # 清理多余空格
    { pattern = "  +", replace = " " },
]
```

#### 增强的提交分类

```toml
commit_parsers = [
    # 使用 emoji 图标增强可读性
    { message = "^feat", group = "✨ Features" },
    { message = "^fix", group = "🐛 Bug Fixes" },
    { message = "^docs?", group = "📚 Documentation" },
    { message = "^perf", group = "⚡ Performance" },
    { message = "^refactor", group = "♻️ Refactor" },
    { message = "^style", group = "💄 Styling" },
    { message = "^test", group = "🧪 Testing" },
    { message = "^build", group = "🔧 Build System" },
    { message = "^ci", group = "👷 CI/CD" },
    { message = "^security", group = "🔒 Security" },
    { message = ".*!:", group = "💥 Breaking Changes" },

    # 跳过发布相关的提交
    { message = "^chore\\(release\\): prepare for", skip = true },
    { message = "^chore: release", skip = true },
]
```

#### 变更日志保护和排序

```toml
# 始终包含 breaking changes
protect_breaking_commits = true

# 按最新提交排序
sort_commits = "newest"
```

### 3. 包配置优化

#### 主包配置

```toml
[[package]]
name = "vx"
# 包含工作空间包的提交到主包变更日志
changelog_include = ["vx-core", "vx-cli", "vx-shim"]

# 自动发布类型检测
git_release_type = "auto"
```

### 4. GitHub 发布模板增强

#### 丰富的发布说明

```toml
git_release_body = """
## 🚀 What's New in {{ version }}

{{ changelog }}

{% if remote.contributors %}
## 👥 Contributors
Thanks to all the contributors who made this release possible:
{% for contributor in remote.contributors -%}
* @{{ contributor.username }}
{% endfor %}
{% endif %}

## 📦 Installation

### 🔧 Package Managers
- **Windows (WinGet)**: `winget install loonghao.vx`
- **Windows (Chocolatey)**: `choco install vx`
- **macOS (Homebrew)**: `brew install loonghao/vx/vx`
- **Windows (Scoop)**: `scoop bucket add vx https://github.com/loonghao/scoop-vx && scoop install vx`

### 📦 Cargo
```bash
cargo install vx
```

### 💾 Download Binary

Download the appropriate binary for your platform from the assets below.

## 🔗 Links

- **Full Changelog**: <https://github.com/loonghao/vx/compare/{{> previous_tag }}...{{ tag }}
- **Documentation**: <https://github.com/loonghao/vx#readme>
- **Issues**: <https://github.com/loonghao/vx/issues>
"""

```

## 🚀 优化效果

### 发布流程改进
1. **智能触发**：只在有 conventional commits 时创建发布，减少噪音
2. **自动分类**：提交自动分类到相应的变更日志部分
3. **链接生成**：自动为 PR 和 issue 生成链接
4. **贡献者识别**：自动识别并感谢贡献者

### 用户体验提升
1. **清晰的变更日志**：使用 emoji 图标和结构化分组
2. **完整的安装指南**：提供多种安装方式
3. **有用的链接**：直接链接到文档、问题和完整变更日志
4. **专业的发布说明**：包含所有必要信息

### 维护效率提高
1. **减少手动工作**：自动化大部分发布流程
2. **一致的格式**：标准化的变更日志和发布说明
3. **错误减少**：通过模板和自动化减少人为错误
4. **可追溯性**：完整的变更历史和链接

## 📚 参考资源

- [Release-plz 官方文档](https://release-plz.ieni.dev/docs/config)
- [Conventional Commits 规范](https://www.conventionalcommits.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Semantic Versioning](https://semver.org/)

## 🔧 使用建议

### 提交消息规范
为了充分利用这些优化，建议使用 conventional commits 格式：

```bash
# 新功能
git commit -m "feat: add new tool management feature"

# 错误修复
git commit -m "fix: resolve version detection issue"

# 文档更新
git commit -m "docs: update installation guide"

# 性能改进
git commit -m "perf: optimize package loading speed"

# 重大变更
git commit -m "feat!: change API interface (BREAKING CHANGE)"
```

### 发布流程

1. **开发阶段**：使用 conventional commits 进行提交
2. **自动检测**：release-plz 自动检测有意义的提交
3. **PR 创建**：自动创建带有标签的发布 PR
4. **发布生成**：合并后自动生成 GitHub 发布
5. **包发布**：触发 crates.io 发布流程

这套优化配置确保了 VX 项目的发布流程既专业又高效，为用户提供了清晰的变更信息和便捷的安装方式。
