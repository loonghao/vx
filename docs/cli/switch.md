# vx switch - 切换工具版本

临时或永久切换工具的默认版本。

## 语法

```bash
vx switch [OPTIONS] <TOOL@VERSION>
```

## 描述

`vx switch` 命令用于切换工具的默认版本。可以临时切换当前会话的版本，或设置为全局默认版本。

## 选项

- `--global` - 设置为全局默认版本
- `--session` - 仅在当前会话中切换（默认）
- `--project` - 在当前项目中切换
- `--temporary` - 临时切换，退出后恢复
- `-v, --verbose` - 显示详细输出

## 参数

- `TOOL@VERSION` - 要切换到的工具和版本

## 示例

### 临时切换版本
```bash
# 在当前会话中切换到 Node.js 20.10.0
vx switch node@20.10.0

# 临时切换，退出后自动恢复
vx switch node@20.10.0 --temporary
```

### 全局切换
```bash
# 设置 Node.js 20.10.0 为全局默认
vx switch node@20.10.0 --global

# 设置 Python 3.11 为全局默认
vx switch python@3.11 --global
```

### 项目级切换
```bash
# 在当前项目中切换版本
vx switch node@18.17.0 --project

# 这会更新 .vx.toml 文件
```

## 工作流程

1. **验证版本**: 检查指定版本是否已安装
2. **备份当前设置**: 保存当前版本设置
3. **应用切换**: 根据选项应用版本切换
4. **更新配置**: 更新相应的配置文件
5. **验证切换**: 确认切换是否成功

## 切换范围

### 会话级切换（默认）
```bash
vx switch node@20.10.0
# 仅在当前终端会话中生效
```

### 全局切换
```bash
vx switch node@20.10.0 --global
# 影响所有新的终端会话
# 更新 ~/.vx/config/global.toml
```

### 项目级切换
```bash
vx switch node@18.17.0 --project
# 仅在当前项目中生效
# 更新 .vx.toml 文件
```

## 输出示例

```bash
$ vx switch node@20.10.0
切换 node 版本: 18.17.0 → 20.10.0
当前会话中 node 版本已切换到 20.10.0

$ node --version
v20.10.0
```

### 全局切换示例
```bash
$ vx switch node@20.10.0 --global
设置 node 全局默认版本: 18.17.0 → 20.10.0
全局配置已更新: ~/.vx/config/global.toml

所有新的终端会话将使用 node@20.10.0
```

### 项目切换示例
```bash
$ vx switch node@18.17.0 --project
更新项目配置: .vx.toml
设置项目 node 版本: 20.10.0 → 18.17.0

项目配置已更新，运行 'vx sync' 应用更改
```

## 版本优先级

vx 按以下优先级选择工具版本：

1. **临时切换** - 最高优先级
2. **项目配置** - `.vx.toml` 文件
3. **会话切换** - 当前会话设置
4. **全局默认** - 全局配置文件
5. **系统安装** - 系统 PATH 中的版本

## 查看当前版本

```bash
# 查看当前使用的版本
vx which node

# 查看所有工具的当前版本
vx version --all

# 查看版本来源
vx version node --source
```

## 恢复版本

```bash
# 恢复到之前的版本
vx switch node@18.17.0

# 清除会话级切换
vx switch --clear-session

# 恢复全局默认
vx switch node --reset-global
```

## 注意事项

1. **版本必须已安装**: 只能切换到已安装的版本
2. **项目配置**: 项目级切换会修改 `.vx.toml` 文件
3. **会话隔离**: 会话级切换不影响其他终端
4. **优先级**: 了解版本选择的优先级顺序

## 相关命令

- [`vx install`](./install.md) - 安装工具版本
- [`vx list`](./list.md) - 列出已安装版本
- [`vx version`](./version.md) - 查看版本信息
- [`vx which`](./which.md) - 查看工具路径
