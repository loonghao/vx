# install 命令

安装一个或多个运行时（Runtime）。

## 语法

```bash
vx install <runtime>[@version] [<runtime>[@version] ...] [--force]
```

## 说明

`vx install` 用于显式安装 vx 管理的运行时。

- 未指定版本时，vx 会解析为 `latest`。
- 支持一次安装多个运行时。
- 对于捆绑运行时，vx 会自动回退到其父运行时进行安装。

例如：

- `cargo`、`rustc` 捆绑在 `rustup` 下。
- 安装 `cargo` 时可能自动转为安装 `rustup`。

## 选项

| 选项 | 说明 |
|---|---|
| `-f`, `--force` | 即使已安装也强制重装 |

## 示例

```bash
# 安装最新版本
vx install node uv go

# 安装指定版本
vx install node@22 go@1.22

# Rust 生态（推荐）
vx install rustup
vx cargo --version
vx rustc --version

# 强制重装
vx install node@22 --force
```

## 版本说明

- 运行时版本与工具链版本可能不是同一语义。
- Rust 建议安装/配置 `rustup`，日常使用 `vx cargo` / `vx rustc`。

## 相关文档

- [`overview`](./overview) - CLI 总览
- [`list`](./list) - 查看已安装/可用运行时
- [`global`](./global) - 全局包管理
