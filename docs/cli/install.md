# vx install - 安装工具

手动安装指定版本的工具。

## 语法

```bash
vx install <tool>[@version] [options]
vx install <tool1>[@version1] <tool2>[@version2] ... [options]
```

## 描述

`vx install` 命令用于手动安装指定版本的工具。如果不指定版本，将安装最新稳定版本。

## 参数

- `<tool>` - 要安装的工具名称
- `[@version]` - 可选的版本号，支持多种格式：
  - `latest` - 最新稳定版本
  - `1.2.3` - 精确版本号
  - `^1.2.0` - 兼容版本范围
  - `~1.2.0` - 补丁版本范围

## 选项

- `--force` - 强制重新安装，即使已存在
- `--prerelease` - 包含预发布版本
- `--dry-run` - 仅显示将要执行的操作，不实际安装
- `--verbose` - 显示详细的安装过程

## 示例

### 基本安装
```bash
# 安装最新版本
vx install node
vx install uv
vx install go

# 安装特定版本
vx install node@18.17.0
vx install uv@0.1.0
vx install go@1.21.6
```

### 批量安装
```bash
# 安装多个工具
vx install node@18.17.0 uv@latest go@1.21.6

# 从项目配置安装
vx install --from-config
```

### 高级选项
```bash
# 强制重新安装
vx install node@18.17.0 --force

# 包含预发布版本
vx install node@latest --prerelease

# 预览安装操作
vx install node@18.17.0 --dry-run
```

## 版本格式

### 精确版本
```bash
vx install node@18.17.0          # 必须是这个版本
vx install uv@0.1.0             # 必须是这个版本
```

### 语义化版本范围
```bash
vx install node@^18.0.0          # 兼容 18.x.x
vx install go@~1.21.0            # 兼容 1.21.x
vx install uv@>=0.1.0            # 大于等于 0.1.0
```

### 特殊版本标识
```bash
vx install node@latest           # 最新稳定版本
vx install node@lts              # 最新LTS版本（如果支持）
vx install python@3              # 最新的3.x版本
```

## 安装位置

工具将被安装到以下位置：
```
~/.vx/tools/<tool>/<version>/
```

例如：
```
~/.vx/tools/node/18.17.0/
~/.vx/tools/uv/0.1.0/
~/.vx/tools/go/1.21.6/
```

## 安装过程

1. **版本解析** - 解析版本要求，获取可用版本列表
2. **版本选择** - 选择符合要求的最佳版本
3. **下载** - 从官方源或镜像下载工具包
4. **解压** - 解压到临时目录
5. **安装** - 移动到最终安装位置
6. **验证** - 验证安装是否成功
7. **清理** - 清理临时文件

## 配置

### 安装源配置
```toml
# ~/.vx/config.toml
[registries]
node = "https://nodejs.org/dist/"
python = "https://www.python.org/ftp/python/"
go = "https://golang.org/dl/"

[mirrors]
# 使用镜像源加速下载
node = "https://npmmirror.com/mirrors/node/"
python = "https://npmmirror.com/mirrors/python/"
```

### 安装选项
```toml
[install]
timeout = 300                    # 下载超时时间（秒）
retry_count = 3                  # 重试次数
verify_checksum = true           # 验证校验和
parallel_downloads = 4           # 并行下载数
```

## 故障排除

### 安装失败
```bash
# 检查网络连接
vx --verbose install node@18.17.0

# 清理缓存重试
vx cleanup --cache-only
vx install node@18.17.0 --force

# 使用镜像源
vx config set mirrors.node "https://npmmirror.com/mirrors/node/"
vx install node@18.17.0
```

### 版本不存在
```bash
# 列出可用版本
vx list node

# 搜索版本
vx search node --version 18
```

### 权限问题
```bash
# 检查安装目录权限
ls -la ~/.vx/tools/

# 手动创建目录
mkdir -p ~/.vx/tools/
chmod 755 ~/.vx/tools/
```

## 相关命令

- [list](./list.md) - 列出可用工具和版本
- [remove](./remove.md) - 移除已安装的工具
- [update](./update.md) - 更新工具到最新版本
- [sync](./sync.md) - 同步项目所需工具
