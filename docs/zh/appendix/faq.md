# 常见问题

## 通用

### vx 是什么？

vx 是一个通用开发工具管理器，提供零学习成本的体验。你只需在已有命令前加 `vx`，工具就会自动安装并执行。

### vx 和 asdf / mise / proto 有什么区别？

| 特性 | vx | asdf | mise | proto |
|------|-----|------|------|-------|
| 零学习成本 | ✅ | ❌ | ❌ | ❌ |
| 自动安装 | ✅ | ❌ | ✅ | ✅ |
| 48+ 内置工具 | ✅ | 插件 | ✅ | 有限 |
| 声明式配置 | ✅ | ✅ | ✅ | ✅ |
| Windows 原生支持 | ✅ | ❌ | ✅ | ✅ |
| 脚本系统 | ✅ | ❌ | ✅ | ❌ |
| 扩展系统 | ✅ | ❌ | ❌ | ❌ |
| 全局包隔离 | ✅ | ❌ | ❌ | ❌ |
| 用 Rust 编写 | ✅ | Shell | Rust | Rust |

### vx 在哪里存储数据？

默认在 `~/.vx/` 目录：

```
~/.vx/
├── store/        # 已安装的工具版本
├── cache/        # 下载缓存
├── bin/          # 全局 shims
├── envs/         # 虚拟环境
├── providers/    # 自定义 Provider
└── config/       # 配置文件
```

可通过 `VX_HOME` 环境变量覆盖。

### vx 需要管理员/root 权限吗？

不需要。vx 安装在用户目录下，所有操作都在用户权限范围内。

## 安装

### 安装脚本支持哪些平台？

- **Linux**: x86_64, aarch64
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64

### 如何卸载 vx？

```bash
# 移除二进制文件和数据
rm -rf ~/.vx
# 移除 shell 配置中的 vx 相关行
```

### 如何更新 vx？

```bash
vx self-update
```

## 工具管理

### 如何安装指定版本的工具？

```bash
vx install node@22.11.0     # 精确版本
vx install node@22          # 最新 22.x
vx install node@lts         # 最新 LTS
vx install "node@^22"       # 语义化范围
```

### 如何同时安装多个工具？

```bash
vx install node@22 python@3.12 go@1.23 uv@latest
```

### 工具在首次使用时会自动安装吗？

是的！直接运行 `vx node --version`，如果 Node.js 未安装，vx 会自动下载并安装最新稳定版。

### 如何查看已安装的工具？

```bash
vx list --installed
```

### 如何清理旧版本？

```bash
vx cache prune              # 清理过期缓存
vx uninstall node@18        # 移除特定版本
```

## 项目配置

### vx.toml 和 vx.lock 有什么区别？

- **vx.toml** — 声明式工具需求（版本范围），给人读的
- **vx.lock** — 精确锁定的版本，确保可重现环境

### 如何让团队成员使用相同的工具版本？

1. 在项目中添加 `vx.toml`
2. 运行 `vx lock` 生成锁文件
3. 将两个文件提交到版本控制
4. 团队成员运行 `vx setup` 即可

## Shell 集成

### 支持哪些 Shell？

Bash、Zsh、Fish 和 PowerShell。

### Shell 集成提供什么功能？

- 进入项目目录时自动切换工具版本
- Tab 补全
- PATH 自动配置

## CI/CD

### 如何在 GitHub Actions 中使用？

```yaml
- uses: loonghao/vx@main
  with:
    tools: node@22 python@3.12
```

或使用 `vx setup` 从 `vx.toml` 安装所有工具。

### vx 有 Docker 镜像吗？

有。`vx:latest` 和 `vx:tools-latest`（预装常用工具）。

## 性能

### vx 的启动开销大吗？

vx 使用 Rust 编写，启动时间通常在几毫秒内。

### 下载速度慢怎么办？

启用 CDN 加速：

```bash
export VX_CDN_ENABLED=true
vx install node@22
```

## 扩展

### 如何添加 vx 不支持的工具？

使用[声明式 Provider](/zh/guide/manifest-driven-providers) 创建自定义 Provider：

```toml
# ~/.vx/providers/mytool/provider.toml
[provider]
name = "mytool"

[[runtimes]]
name = "mytool"
executable = "mytool"

[runtimes.version_source]
type = "github_releases"
owner = "org"
repo = "mytool"
```

### 如何创建扩展？

参见[扩展开发指南](/zh/advanced/extension-development)。

## 更多帮助

- [故障排除](/zh/appendix/troubleshooting)
- [GitHub Issues](https://github.com/loonghao/vx/issues)
- [贡献指南](/zh/advanced/contributing)
