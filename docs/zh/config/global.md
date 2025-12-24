# 全局配置

全局配置文件位于 `~/.config/vx/config.toml`。

## 配置文件位置

| 平台 | 路径 |
|------|------|
| Linux | `~/.config/vx/config.toml` |
| macOS | `~/.config/vx/config.toml` |
| Windows | `%APPDATA%\vx\config.toml` |

## 配置示例

```toml
[defaults]
auto_install = true
parallel_install = true
cache_duration = "7d"

[tools]
node = "lts"
python = "3.11"
```

## 配置选项

### [defaults]

| 选项 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `auto_install` | bool | true | 自动安装缺失的工具 |
| `parallel_install` | bool | true | 并行安装工具 |
| `cache_duration` | string | "7d" | 缓存持续时间 |

### [tools]

工具的默认版本。

```toml
[tools]
node = "lts"
python = "3.11"
go = "1.21"
```

## 管理配置

```bash
# 显示配置
vx config show

# 设置值
vx config set defaults.auto_install true

# 获取值
vx config get defaults.auto_install

# 重置
vx config reset

# 编辑
vx config edit
```
