# shell 命令

Shell 集成和命令补全。

## 语法

```bash
vx shell <subcommand> [options]
```

## 子命令

### init

生成 shell 集成脚本。

```bash
vx shell init <shell>
```

支持的 shell：

- `bash`
- `zsh`
- `fish`
- `powershell`

### completions

生成命令补全脚本。

```bash
vx shell completions <shell>
```

## 示例

### 设置 Shell 集成

::: code-group

```bash [Bash]
# 添加到 ~/.bashrc
eval "$(vx shell init bash)"
```

```zsh [Zsh]
# 添加到 ~/.zshrc
eval "$(vx shell init zsh)"
```

```fish [Fish]
# 添加到 ~/.config/fish/config.fish
vx shell init fish | source
```

```powershell [PowerShell]
# 添加到 $PROFILE
Invoke-Expression (& vx shell init powershell | Out-String)
```

:::

### 安装命令补全

::: code-group

```bash [Bash]
vx shell completions bash > ~/.local/share/bash-completion/completions/vx
```

```zsh [Zsh]
vx shell completions zsh > ~/.zfunc/_vx
```

```fish [Fish]
vx shell completions fish > ~/.config/fish/completions/vx.fish
```

```powershell [PowerShell]
vx shell completions powershell > ~\Documents\PowerShell\Completions\vx.ps1
```

:::

## 参见

- [Shell 集成](/zh/guide/shell-integration) - Shell 集成指南
