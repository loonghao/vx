# Shell 集成

Shell 集成提供自动环境激活和命令补全。

## 设置 Shell 集成

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

## Shell 集成提供的功能

### 1. 自动 PATH 设置

你的 PATH 会被配置为找到 vx 管理的工具：

```bash
# 没有 shell 集成
/usr/bin/node  # 系统 node

# 有 shell 集成
~/.local/share/vx/envs/default/node  # vx 管理的 node
```

### 2. 基于目录的环境切换

当你 `cd` 进入包含 `vx.toml` 的目录时，环境会自动激活：

```bash
cd my-project/  # 激活项目环境
cd ~            # 返回默认环境
```

### 3. 命令补全

vx 命令和选项的 Tab 补全：

```bash
vx ins<TAB>     # 补全为 "vx install"
vx install no<TAB>  # 补全为 "vx install node"
```

## Shell 补全

单独生成补全脚本：

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

## 环境指示器

使用 shell 集成，你的提示符可以显示活动环境：

::: code-group

```bash [Bash]
PS1='$([ -n "$VX_ENV" ] && echo "[$VX_ENV] ")'"$PS1"
```

```zsh [Zsh]
PROMPT='%F{cyan}${VX_ENV:+[$VX_ENV] }%f'"$PROMPT"
```

```fish [Fish]
function fish_prompt
    if set -q VX_ENV
        echo -n "[$VX_ENV] "
    end
    # ... 提示符的其余部分
end
```

```powershell [PowerShell]
function prompt {
    if ($env:VX_ENV) {
        Write-Host "[$env:VX_ENV] " -NoNewline -ForegroundColor Cyan
    }
    # ... 提示符的其余部分
}
```

:::

## 手动环境激活

如果你更喜欢手动控制：

```bash
# 激活特定环境
eval "$(vx env activate my-env)"

# 停用（恢复默认）
eval "$(vx env activate default)"
```

## 故障排除

### Shell 集成不工作

1. 验证安装：

   ```bash
   vx shell init bash  # 应该输出 shell 脚本
   ```

2. 检查是否正确加载：

   ```bash
   source ~/.bashrc
   echo $PATH | grep vx  # 应该显示 vx 路径
   ```

3. 重启终端

### 补全不工作

1. 验证补全脚本存在：

   ```bash
   ls ~/.local/share/bash-completion/completions/vx
   ```

2. 对于 Zsh，确保调用了 `compinit`：

   ```zsh
   autoload -Uz compinit && compinit
   ```

### Shell 启动缓慢

如果 shell 启动缓慢：

1. 使用延迟加载：

   ```bash
   # Bash - 延迟加载 vx
   vx() {
       unset -f vx
       eval "$(command vx shell init bash)"
       vx "$@"
   }
   ```

2. 缓存初始化脚本：

   ```bash
   # 生成一次
   vx shell init bash > ~/.vx-init.sh

   # 加载缓存版本
   source ~/.vx-init.sh
   ```

## 高级配置

### 禁用自动切换

如果你不想自动切换环境：

```bash
export VX_AUTO_SWITCH=false
eval "$(vx shell init bash)"
```

### 自定义钩子

在环境更改时添加自定义行为：

```bash
# Bash
vx_env_changed() {
    echo "已切换到环境：$VX_ENV"
    # 自定义逻辑
}
```

## 下一步

- [CLI 参考](/zh/cli/shell) - Shell 命令参考
- [环境管理](/zh/guide/environment-management) - 管理环境
