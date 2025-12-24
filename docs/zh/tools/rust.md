# Rust

vx 支持 Rust 编程语言。

## 支持的工具

| 工具 | 描述 |
|------|------|
| `cargo` | Rust 包管理器 |
| `rustc` | Rust 编译器 |

## 使用示例

```bash
# 运行 Cargo
vx cargo --version
vx cargo build --release
vx cargo test
vx cargo run

# 运行 rustc
vx rustc --version
```

## 版本管理

```bash
# 安装稳定版
vx install rust@stable

# 安装特定版本
vx install rust@1.75.0
```

## 项目配置

```toml
[tools]
rust = "stable"

[scripts]
build = "cargo build --release"
test = "cargo test"
run = "cargo run"
```
