# dev 命令

进入开发环境，使用项目配置的所有工具。

## 语法

```bash
vx dev [OPTIONS]
vx dev -c <COMMAND>
vx dev --export [--format <FORMAT>]
```

## 描述

`vx dev` 命令是使用 vx 管理工具的主要方式。它提供三种操作模式：

1. **交互式 Shell 模式**（默认）：启动一个新的 shell，所有项目工具都可在 PATH 中使用
2. **命令模式**（`-c`）：在开发环境中运行单个命令
3. **导出模式**（`--export`）：输出用于环境激活的 shell 脚本

## 选项

| 选项 | 说明 |
|------|------|
| `--shell <SHELL>` | 使用的 shell（未指定时自动检测） |
| `-c`, `--command <CMD>` | 运行命令而不是启动 shell |
| `--no-install` | 不安装缺失的工具 |
| `-v`, `--verbose` | 显示详细输出 |
| `-e`, `--export` | 导出环境变量用于 shell 激活 |
| `-f`, `--format <FORMAT>` | `--export` 的输出格式：shell、powershell、batch、github |

## 使用场景

### 场景 1：交互式开发

进入开发 shell，所有工具可用：

```bash
vx dev
```

输出：

```
✓ Entering vx development environment
ℹ Tools: node, uv, go

💡 Type 'exit' to leave the dev environment.

(vx) $ node --version
v20.10.0
(vx) $ exit
ℹ Left vx development environment
```

**适用场景**：日常开发、探索代码库、运行临时命令。

### 场景 2：运行单个命令

在开发环境中执行命令，无需进入 shell：

```bash
vx dev -c "npm run build"
vx dev -c "node scripts/deploy.js"
vx dev -c "go test ./..."
```

**适用场景**：CI/CD 流水线、脚本、一次性任务。

### 场景 3：Shell 激活（导出模式）

导出环境变量以在当前 shell 中激活工具：

**Bash/Zsh:**

```bash
eval "$(vx dev --export)"
```

**Fish:**

```fish
vx dev --export | source
```

**PowerShell:**

```powershell
Invoke-Expression (vx dev --export --format powershell)
```

**Windows CMD:**

```batch
vx dev --export --format batch > activate.bat && activate.bat
```

**适用场景**：IDE 集成、shell 配置文件、自定义脚本。

### 场景 4：CI/CD 集成

在 GitHub Actions 中使用导出模式：

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: 安装 vx
        run: curl -fsSL https://get.vx.dev | bash

      - name: 安装工具
        run: vx setup

      - name: 激活环境
        run: |
          eval "$(vx dev --export --format github)"
          # 工具现在可用
          node --version
          npm run build
```

**适用场景**：GitHub Actions、GitLab CI、Jenkins 等。

## 导出格式

| 格式 | 说明 | 默认环境 |
|------|------|----------|
| `shell` | Bash/Zsh 兼容 | Unix |
| `powershell` | PowerShell 兼容 | Windows (PowerShell) |
| `batch` | Windows CMD 批处理 | Windows (CMD) |
| `github` | GitHub Actions 格式 | GitHub Actions |

### 输出示例

对于配置了 `node` 和 `uv` 的项目：

**Shell 格式:**

```bash
export PATH="/home/user/.vx/bin:/home/user/.vx/store/node/20.0.0/bin:/home/user/.vx/store/uv/0.5.14:$PATH"
```

**PowerShell 格式:**

```powershell
$env:PATH = "C:\Users\user\.vx\bin;C:\Users\user\.vx\store\node\20.0.0;C:\Users\user\.vx\store\uv\0.5.14;$env:PATH"
```

**GitHub Actions 格式:**

```bash
echo "/home/runner/.vx/bin" >> $GITHUB_PATH
echo "/home/runner/.vx/store/node/20.0.0/bin" >> $GITHUB_PATH
echo "/home/runner/.vx/store/uv/0.5.14" >> $GITHUB_PATH
export PATH="/home/runner/.vx/bin:/home/runner/.vx/store/node/20.0.0/bin:/home/runner/.vx/store/uv/0.5.14:$PATH"
```

## 环境设置

进入开发环境时（交互式或命令模式）：

1. **PATH 更新**：添加 `~/.vx/store/` 中的项目工具版本
2. **VX_DEV=1**：表示处于活动环境
3. **VX_PROJECT_ROOT**：设置为项目目录
4. **自定义环境变量**：来自 `vx.toml` 的 `[env]` 部分
5. **自动安装缺失工具**（除非使用 `--no-install`）

## 配置

开发环境由 `vx.toml` 配置：

```toml
[tools]
node = "20"
uv = "latest"
go = "1.21"

[env]
NODE_ENV = "development"
DEBUG = "true"

[settings]
auto_install = true
```

## vx dev 与 vx run 的比较

| 特性 | `vx dev` | `vx run` |
|------|----------|----------|
| 用途 | 开发环境 | 运行定义的脚本 |
| 范围 | 所有工具在 PATH 中 | 特定脚本 |
| 交互式 | 是（shell 模式） | 否 |
| 脚本 | 任意命令 | 仅 `vx.toml` 中定义的 |

**使用 `vx dev`**：

- 交互式开发
- 临时命令
- Shell 激活

**使用 `vx run`**：

- 预定义的项目脚本
- 一致的任务执行
- 团队工作流

## 提示

1. **添加到 shell 配置文件**：在新终端中自动激活：

   ```bash
   # ~/.bashrc 或 ~/.zshrc
   if [ -f "vx.toml" ]; then
     eval "$(vx dev --export)"
   fi
   ```

2. **IDE 集成**：配置 IDE 的终端在启动时运行 `eval "$(vx dev --export)"`。

3. **检查环境**：运行 `echo $PATH` 验证工具路径已包含。

4. **指定 shell**：如果自动检测失败，使用 `--shell`：

   ```bash
   vx dev --shell zsh
   vx dev --shell fish
   ```

## 参见

- [setup](setup) - 安装项目工具
- [run](run) - 运行定义的脚本
- [env](env) - 环境管理
