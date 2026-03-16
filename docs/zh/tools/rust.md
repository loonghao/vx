# Rust

vx 通过 `rustup` 及其捆绑运行时（`cargo`、`rustc`）支持 Rust 开发。

## 运行时关系

| 运行时 | 角色 | 推荐用法 |
|---|---|---|
| `rustup` | 工具链管理/安装 | `vx rustup ...` |
| `cargo` | 构建、测试、依赖管理 | `vx cargo ...` |
| `rustc` | Rust 编译器 | `vx rustc ...` |

> `vx rust` 当前是 `vx rustc` 的别名。为避免歧义，文档与脚本建议显式使用 `vx rustc`。

## 安装

```bash
# 推荐：安装 rustup 运行时
vx install rustup
vx install rustup@latest
```

## 日常命令

### Cargo

```bash
vx cargo --version
vx cargo build --release
vx cargo test
vx cargo run
```

### Rustc

```bash
vx rustc --version
vx rustc main.rs -o main
```

### Rustup（工具链管理）

```bash
vx rustup --version
vx rustup toolchain list
vx rustup target add x86_64-unknown-linux-musl
```

## vx.toml 推荐写法

```toml
[tools]
rustup = "latest"

[scripts]
build = "cargo build --release"
test = "cargo test"
lint = "cargo clippy -- -D warnings"
format = "cargo fmt"
```

## 版本说明（重要）

`rustup` 版本和 `rustc` 版本不是一回事：

- `rustup = "1.93.1"` 通常无效（这是 Rust 编译器版本，不是 rustup 发布版本）。
- 如果你要固定某个编译器工具链，请通过 `vx rustup toolchain ...` 管理。

## Rust 生态包语法

```bash
# 按需运行 Rust 生态包
vx cargo:ripgrep::rg --version
vx cargo:fd-find::fd .
```

