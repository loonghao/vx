# vx execute - 工具执行

直接执行工具，提供完全透明的使用体验。

## 语法

```bash
vx <tool> [args...]
```

## 描述

VX 的核心功能是透明代理工具执行。用户只需要 `vx <tool>` 就能自动处理一切：

- 自动检测项目配置（`.vx.toml`）
- 自动安装缺失工具
- 自动使用正确版本
- 透明代理到正确版本执行

## 示例

### 基本使用
```bash
# 完全透明的使用体验
vx node --version                    # 自动使用项目配置的版本
vx uv pip install requests          # 自动安装并使用配置的uv版本
vx go build                         # 自动使用项目配置的go版本
```

### 自动安装功能
```bash
# 自动安装并运行（✨ 新功能）
vx uv pip install requests          # 如果uv未安装，自动下载并使用
vx python -c "print('Hello')"       # 如果python未安装，自动安装最新版本
vx npm install                      # 如果npm未安装，自动安装最新版本
```

### 自动安装过程示例
```bash
$ vx node --version
# 🔍 检测到工具 'node' 未安装
# 📦 正在获取最新版本信息...
# ⬇️  正在下载 Node.js v20.10.0...
# 📁 正在安装到 ~/.vx/tools/node/20.10.0/...
# ✅ 安装完成！
# v20.10.0
```

## 执行流程

1. 用户运行 `vx node --version`
2. 查找项目配置文件（`.vx.toml`）
3. 解析版本需求（`18.17.0` 或 `latest`）
4. 检查工具是否已安装：
   - ✅ 已安装：直接使用
   - ❌ 未安装：触发自动安装流程
5. **自动安装流程**：
   - 检查自动安装是否启用（默认启用）
   - 获取工具的可用版本列表
   - 选择最新稳定版本（跳过预发布版本）
   - 下载并安装到 `~/.vx/tools/<tool>/<version>/`
   - 验证安装是否成功
6. 透明代理到正确版本执行

## 自动安装配置

### 全局配置
```toml
# ~/.vx/config.toml
[auto_install]
enabled = true                    # 启用自动安装
include_prerelease = false        # 是否包含预发布版本
timeout = 300                     # 安装超时时间（秒）
confirm_before_install = false    # 安装前是否需要确认
```

### 项目配置
```toml
# .vx.toml
[auto_install]
enabled = true                    # 项目级别开关
```

## 选项

### 全局选项
- `--use-system-path` - 使用系统PATH查找工具而非vx管理的版本
- `--verbose` - 启用详细输出和日志

## 支持的工具

VX 支持多种开发工具：

- **Node.js 生态**: node, npm, npx, yarn, pnpm
- **Python 生态**: python, pip, uv
- **Go 生态**: go, gofmt, goimports
- **Rust 生态**: rust, cargo, rustc
- **其他**: 通过插件系统扩展

## 故障排除

### 工具安装失败
```bash
# 检查网络连接
vx --verbose install node@18.17.0

# 清理缓存重试
vx cleanup --cache-only
vx install node@18.17.0 --force

# 使用系统PATH作为后备
vx --use-system-path node --version
```

### 版本问题
```bash
# 显示工具实际路径
vx which node

# 显示工具版本信息
vx version node
```

## 相关命令

- [install](./install.md) - 手动安装工具
- [list](./list.md) - 列出可用工具
- [config](./config.md) - 配置管理
- [sync](./sync.md) - 项目同步
