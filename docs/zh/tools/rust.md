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

## Rust 版本解析优先级

Rust 是唯一一个 vx **不是**唯一权威的生态。仓库一旦明确声明了工具链，vx 就把命令
直接交给 rustup：不安装、不把自己的 store 加进 `PATH`、也绝不执行 `rustup default`
（那会改写整台机器上其它项目的默认工具链）。

优先级从高到低：

| 优先级 | 信号 | 行为 |
|---|---|---|
| 1 | `rust-toolchain.toml` / `rust-toolchain` | 交给 rustup。从当前工作目录向上查找。 |
| 2 | 环境变量 `RUSTUP_TOOLCHAIN` | 交给 rustup。 |
| 3 | `vx.toml` 中 `rust = "rustup-managed"` | 显式交给 rustup。 |
| 4 | `vx.toml` 中 `rust = "<version>"` | vx 把该工具链装进自己的 store。 |

说明：

- 同时存在 (1) 和 (2) 时以 (2) 为准 —— 这是 rustup 自身的优先级，vx 上报的工具链
  必须与实际运行的工具链一致。
- `vx.toml` 里的数字 pin 与实际生效工具链不一致时，vx 会向 stderr 输出告警，且
  `vx check` 返回非 0。仓库的 toolchain 文件依然优先；告警只是为了让失效的 pin 不被
  悄悄忽略。
- 删掉 `rust` pin，或写成 `rust = "rustup-managed"`，即可显式记录「rustup 管理」并
  消除告警。
- `vx cargo@1.90.0` 这种显式指定版本是调用方在指名工具链，一律走 vx，不受 toolchain
  文件影响。

```toml
# 已经用 rustup 固定工具链的仓库，推荐这样写
[tools]
rust = "rustup-managed"
```

## Rust 生态包语法

```bash
# 按需运行 Rust 生态包
vx cargo:ripgrep::rg --version
vx cargo:fd-find::fd .
```

