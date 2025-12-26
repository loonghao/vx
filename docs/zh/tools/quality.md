# 代码质量工具

vx 支持维护代码质量和一致性的工具。

## pre-commit

管理和维护多语言 pre-commit 钩子的框架。

```bash
vx install pre-commit latest

vx pre-commit --version
vx pre-commit install            # 安装钩子
vx pre-commit run --all-files    # 对所有文件运行
vx pre-commit autoupdate         # 更新钩子版本
vx pre-commit uninstall          # 移除钩子
```

**主要特性：**

- 多语言支持
- 自动钩子管理
- CI/CD 集成
- 丰富的钩子生态系统

**.pre-commit-config.yaml 示例：**

```yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-json

  - repo: https://github.com/psf/black
    rev: 24.1.0
    hooks:
      - id: black

  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.14
    hooks:
      - id: ruff
        args: [--fix]
```

**项目配置：**

```toml
[tools]
pre-commit = "latest"

[scripts]
lint = "pre-commit run --all-files"
lint-install = "pre-commit install"
lint-update = "pre-commit autoupdate"
```

## CI/CD 集成

```yaml
# GitHub Actions 示例
- name: 设置 vx
  uses: loonghao/vx@v0.5

- name: 安装 pre-commit
  run: vx install pre-commit latest

- name: 运行 pre-commit
  run: vx pre-commit run --all-files
```

## 最佳实践

1. **提交钩子安装**：克隆后始终运行 `pre-commit install`
2. **CI 集成**：在 CI 中运行 pre-commit 以捕获问题
3. **版本锁定**：锁定钩子版本以保证可复现性
4. **仅暂存文件**：默认行为仅对暂存文件运行

```toml
[tools]
pre-commit = "3.6"  # 锁定版本

[scripts]
# 快速检查（仅暂存文件）
lint = "pre-commit run"

# 完整检查（所有文件）
lint-all = "pre-commit run --all-files"

# 新开发者设置
setup = "pre-commit install && pre-commit install --hook-type commit-msg"
```
