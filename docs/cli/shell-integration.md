# vx shell-integration - Shell 集成

配置 Shell 集成功能，包括自动补全和环境初始化。

## 语法

```bash
# Shell集成命令
vx shell init [SHELL]
vx shell completions <SHELL>
```

## 描述

vx 提供了丰富的 Shell 集成功能，包括自动补全、环境初始化和透明的工具代理。

## 支持的 Shell

- `bash` - Bash Shell
- `zsh` - Zsh Shell  
- `fish` - Fish Shell
- `powershell` - PowerShell
- `cmd` - Windows Command Prompt

## Shell 初始化

### 自动检测 Shell
```bash
# 自动检测当前Shell并输出初始化脚本
vx shell init

# 在 Shell 配置文件中使用
eval "$(vx shell init)"
```

### 指定 Shell
```bash
# 为特定Shell生成初始化脚本
vx shell init bash
vx shell init zsh
vx shell init fish
vx shell init powershell
```

## 自动补全

### 安装自动补全
```bash
# Bash
vx shell completions bash > /etc/bash_completion.d/vx
# 或者用户级别
vx shell completions bash > ~/.bash_completion.d/vx

# Zsh
vx shell completions zsh > /usr/local/share/zsh/site-functions/_vx
# 或者用户级别
vx shell completions zsh > ~/.zsh/completions/_vx

# Fish
vx shell completions fish > ~/.config/fish/completions/vx.fish

# PowerShell
vx shell completions powershell | Out-String | Invoke-Expression
```

### 临时启用补全
```bash
# Bash/Zsh
source <(vx shell completions bash)  # 或 zsh

# Fish
vx shell completions fish | source

# PowerShell
vx shell completions powershell | Out-String | Invoke-Expression
```

## 配置示例

### Bash 配置 (~/.bashrc)
```bash
# 添加 vx 到 PATH
export PATH="$HOME/.vx/bin:$PATH"

# 初始化 vx Shell 集成
eval "$(vx shell init bash)"
source <(vx shell completions bash)

# 可选：设置别名
alias vnode="vx node"
alias vuv="vx uv"
alias vgo="vx go"

# 使用 vx 内置别名
alias vi="vx i"      # vx install
alias vrm="vx rm"    # vx uninstall
alias vls="vx ls"    # vx list
alias vup="vx up"    # vx update
```

### Zsh 配置 (~/.zshrc)
```bash
# 添加 vx 到 PATH
export PATH="$HOME/.vx/bin:$PATH"

# 初始化 vx Shell 集成
eval "$(vx shell-init zsh)"

# 启用自动补全
source <(vx completion zsh)

# 可选：启用 vx 提示符集成
autoload -U add-zsh-hook
add-zsh-hook precmd vx_prompt_update
```

### Fish 配置 (~/.config/fish/config.fish)
```fish
# 添加 vx 到 PATH
set -gx PATH $HOME/.vx/bin $PATH

# 初始化 vx Shell 集成
vx shell-init fish | source

# 启用自动补全
vx completion fish | source

# 可选：设置别名
alias vnode="vx node"
alias vuv="vx uv"
alias vgo="vx go"
```

### PowerShell 配置 ($PROFILE)
```powershell
# 添加 vx 到 PATH
$env:PATH = "$env:USERPROFILE\.vx\bin;$env:PATH"

# 初始化 vx Shell 集成
Invoke-Expression (vx shell-init powershell)

# 启用自动补全
vx completion powershell | Out-String | Invoke-Expression

# 可选：设置别名
Set-Alias vnode "vx node"
Set-Alias vuv "vx uv"
Set-Alias vgo "vx go"
```

## Shell 集成功能

### 环境变量设置
- `VX_HOME` - vx 主目录
- `VX_CURRENT_VENV` - 当前激活的虚拟环境
- `VX_PROJECT_ROOT` - 当前项目根目录

### 提示符集成
```bash
# 显示当前工具版本
export PS1="[vx: $(vx current-versions --short)] $PS1"

# 显示当前虚拟环境
export PS1="[$(vx venv current --name-only)] $PS1"
```

### 自动环境切换
```bash
# 进入目录时自动激活项目环境
cd() {
    builtin cd "$@"
    if [[ -f .vx.toml ]]; then
        vx sync --quiet
    fi
}
```

## 高级功能

### 钩子脚本
```bash
# ~/.vx/hooks/shell-init
#!/bin/bash
echo "VX Shell 集成已初始化"

# ~/.vx/hooks/pre-command
#!/bin/bash
# 在每个 vx 命令执行前运行
echo "执行命令: $VX_COMMAND"

# ~/.vx/hooks/post-command
#!/bin/bash
# 在每个 vx 命令执行后运行
echo "命令完成: $VX_COMMAND (退出码: $VX_EXIT_CODE)"
```

### 自定义函数
```bash
# 快速切换工具版本
vswitch() {
    local tool=$1
    local version=$2
    vx switch "$tool@$version" --session
    echo "已切换到 $tool@$version"
}

# 快速创建虚拟环境
vquick() {
    local name=$1
    shift
    vx venv create "$name" --tools "$@"
    vx venv use "$name"
}

# 显示当前工具状态
vstatus() {
    echo "当前工具版本:"
    vx version --all --current
    echo ""
    echo "当前虚拟环境:"
    vx venv current
}
```

## 自动补全功能

### 命令补全
- 主命令和子命令
- 选项和参数
- 工具名称和版本
- 虚拟环境名称
- 配置键值

### 智能补全
```bash
# 补全已安装的工具
vx remove <TAB>  # 显示已安装的工具

# 补全可用版本
vx install node@<TAB>  # 显示 node 的可用版本

# 补全虚拟环境
vx venv use <TAB>  # 显示可用的虚拟环境
```

## 故障排除

### 补全不工作
```bash
# 检查补全脚本是否正确安装
which vx
vx completion bash --check

# 重新加载 Shell 配置
source ~/.bashrc  # 或相应的配置文件
```

### 环境变量问题
```bash
# 检查环境变量
echo $VX_HOME
echo $PATH | grep vx

# 重新初始化
eval "$(vx shell-init)"
```

### 权限问题
```bash
# 检查 vx 目录权限
ls -la ~/.vx/

# 修复权限
chmod -R 755 ~/.vx/
```

## 性能优化

### 延迟加载
```bash
# 仅在需要时初始化 vx
vx() {
    if [[ -z "$VX_INITIALIZED" ]]; then
        eval "$(command vx shell-init)"
        export VX_INITIALIZED=1
    fi
    command vx "$@"
}
```

### 缓存优化
```bash
# 启用补全缓存
export VX_COMPLETION_CACHE=1
export VX_COMPLETION_CACHE_TTL=3600  # 1小时
```

## 注意事项

1. **Shell 重启**: 配置更改后需要重启 Shell 或重新加载配置
2. **权限要求**: 某些操作可能需要适当的文件权限
3. **性能影响**: Shell 集成可能轻微影响 Shell 启动时间
4. **兼容性**: 确保 Shell 版本支持所需功能

## 相关命令

- [`vx config`](./config.md) - 配置管理
- [`vx venv`](./venv.md) - 虚拟环境管理
- [`vx version`](./version.md) - 版本信息
