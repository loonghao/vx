# Pre-commit Hooks

vx 使用 [prek](https://prek.j178.dev/)（一个基于 Rust 的 pre-commit 替代工具）在每次提交前强制执行代码质量检查。本文档介绍 `.pre-commit-config.yaml` 中配置的 hooks 及其作用。

## 概述

Pre-commit hooks 在执行 `git commit` 时自动运行。如果任何 hook 失败，提交将被阻止，直到问题解决。这能在问题到达 CI 或影响其他开发者之前及早发现。

```bash
# 安装 hooks（一次性设置）
vx prek install

# 手动在所有文件上运行所有 hooks
vx prek run --all-files

# 运行特定 hook
vx prek run --hook-id cargo-hakari
```

## 已配置的 Hooks

### 1. 拼写检查（`typos`）

检查源代码和文档中的常见拼写错误。

```yaml
- repo: https://github.com/crate-ci/typos
  rev: v1.43.4
  hooks:
    - id: typos
```

### 2. Rust 代码格式化（`cargo-fmt`）

确保所有 Rust 代码使用 `rustfmt` 格式化。

```yaml
- id: cargo-fmt
  entry: vx cargo fmt --all --
  types: [rust]
```

### 3. Rust 代码检查（`cargo-clippy`）

运行 Clippy 并将所有警告视为错误。

```yaml
- id: cargo-clippy
  entry: vx cargo clippy --workspace -- -D warnings
  types: [rust]
```

### 4. 测试编译检查（`cargo-check-tests`）

编译所有测试代码，捕获仅在测试文件中出现的错误（如 `E0061` 参数数量错误）。

```yaml
- id: cargo-check-tests
  entry: vx cargo check --workspace --tests
  types: [rust]
```

### 5. YAML/JSON 格式化（`prettier`）

使用 Prettier 格式化 YAML 和 JSON 文件。

```yaml
- id: prettier
  entry: vx npx prettier --write --ignore-unknown
  types_or: [yaml, json]
```

### 6. Workspace-Hack 自动修复（`cargo-hakari-fix`）⭐ 新增

依赖变更时自动重新生成 `workspace-hack` crate 并暂存修改。

```yaml
- id: cargo-hakari-fix
  name: cargo hakari generate (auto-fix)
  entry: bash -c 'vx cargo hakari generate && vx cargo hakari manage-deps && git add crates/workspace-hack/Cargo.toml && vx cargo hakari generate --diff'
  language: system
  files: Cargo\.(toml|lock)$
  pass_filenames: false
```

**为什么重要：** vx 使用 [cargo-hakari](https://docs.rs/cargo-hakari) 通过统一 workspace 中的 feature flags 来优化构建时间。当你在任何 `Cargo.toml` 中添加或更新依赖时，`workspace-hack` crate 必须重新生成。以前这是一个容易遗忘的手动步骤；现在 hook 会自动处理。

**工作原理：**
- 在 `Cargo.toml` 或 `Cargo.lock` 发生变更时触发
- 运行 `cargo hakari generate` 和 `cargo hakari manage-deps` 重新生成 workspace-hack
- 通过 `git add` 暂存更新后的 `crates/workspace-hack/Cargo.toml`
- 使用 `cargo hakari generate --diff` 验证没有剩余差异
- 重新生成的文件会自动包含在你的提交中 — 无需手动干预

### 7. Justfile 重复 Recipe 检测（`justfile-no-duplicate-recipes`）⭐ 新增

检测 `justfile` 中的重复 recipe 定义。

```yaml
- id: justfile-no-duplicate-recipes
  name: justfile no duplicate recipes
  entry: vx just --list
  language: system
  files: ^justfile$
  pass_filenames: false
```

**为什么重要：** `just` 命令运行器不会静默忽略重复的 recipe 定义 — 它会以如下错误退出：

```
error: Recipe `test-pkgs` first defined on line 74 is redefined on line 155
   ——▶ justfile:155:1
    │
155 │ test-pkgs PKGS:
    │ ^^^^^^^^^
Error: Process completed with exit code 1.
```

这个错误会导致所有 `just` 命令失败，破坏整个开发工作流和 CI 流水线。

**工作原理：**
- 仅在 `justfile` 被修改时触发
- 运行 `just --list`，它会解析整个 justfile，遇到重复 recipe 时立即失败
- 在提交时捕获问题，防止其进入 CI

**失败时的修复方法：**

```bash
# 查找重复的 recipe 名称
grep -n "^[a-zA-Z_-]*:" justfile | sort | uniq -d

# 或使用 just 显示错误位置
vx just --list
```

### 8. 通用安全检查

来自 `pre-commit-hooks` 的标准检查：

| Hook | 描述 |
|------|------|
| `check-merge-conflict` | 防止提交未解决的合并冲突标记 |
| `check-added-large-files` | 阻止超过 500 KB 的文件 |
| `end-of-file-fixer` | 确保文件以换行符结尾 |
| `check-toml` | 验证 TOML 语法 |
| `trailing-whitespace` | 删除行尾空白 |

## 安装设置

### 安装 prek

```bash
# 通过 vx 安装 prek（如需要会自动安装）
vx prek install
```

### 验证安装

```bash
# 检查 hooks 是否已安装
ls .git/hooks/pre-commit

# 在所有文件上运行所有 hooks 以验证一切正常
vx prek run --all-files
```

## 跳过 Hooks（仅限紧急情况）

在极少数情况下需要不运行 hooks 直接提交：

```bash
# 跳过所有 hooks（谨慎使用！）
git commit --no-verify -m "emergency fix"
```

::: warning
跳过 hooks 应该是最后手段。CI 流水线运行相同的检查，所以在本地跳过 hooks 只是将失败推迟到 CI。
:::

## 故障排除

### 添加依赖后 `cargo-hakari-fix` 失败

Hook 应该能自动修复大多数情况。如果仍然失败：

```bash
# 手动重新生成 workspace-hack
vx cargo hakari generate
vx cargo hakari manage-deps

# 验证现在已经干净
vx cargo hakari generate --diff

# 或使用 justfile 快捷命令
vx just hakari-fix
```

### `justfile-no-duplicate-recipes` 失败

```bash
# 显示带行号的错误
vx just --list

# 搜索重复项
grep -n "^recipe-name:" justfile
```

### Hook 运行缓慢

`cargo-clippy` 和 `cargo-check-tests` hooks 需要编译 Rust 代码，首次运行可能较慢。后续运行会使用增量编译缓存，速度会快很多。

### 重置所有 hooks

```bash
# 卸载并重新安装
vx prek uninstall
vx prek install
```

## 进阶用法

### 在特定文件上运行 hooks

```bash
# 在特定文件上运行所有 hooks
vx prek run --files src/main.rs

# 在特定文件上运行特定 hook
vx prek run --hook-id cargo-fmt --files src/lib.rs src/main.rs
```

### 在 CI 中运行 hooks

CI 流水线通过 `vx prek run --all-files` 运行相同的 hooks。这确保了：

1. 本地开发和 CI 使用完全相同的检查
2. 不会出现"在我机器上能运行"的格式化或 lint 问题
3. workspace-hack 始终保持同步

### 添加新 hook

要添加新的 pre-commit hook，编辑 `.pre-commit-config.yaml`：

```yaml
- repo: local
  hooks:
    - id: my-new-check
      name: 我的新检查描述
      entry: vx cargo my-check
      language: system
      types: [rust]
      pass_filenames: false
```

然后运行 `vx prek run --all-files` 验证新 hook 正常工作。
