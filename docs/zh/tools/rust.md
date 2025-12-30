# Rust

vx 支持 Rust 编程语言。

## 支持的工具

| 工具 | 描述 |
|------|------|
| `cargo` | Rust 包管理器 |
| `rustc` | Rust 编译器 |

## 安装

```bash
# 安装稳定版
vx install rust stable

# 安装特定版本
vx install rust 1.75.0
```

## 下载格式

vx 在所有平台（Windows、macOS、Linux）上都使用 `.tar.gz` 格式下载 Rust 工具链。这确保了跨操作系统的一致解压行为，避免了使用 `.msi` 等平台特定的安装程序格式。

下载 URL 格式如下：
```
https://static.rust-lang.org/dist/rust-{version}-{platform}.tar.gz
```

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

## 常见工作流

### 新建项目

```bash
vx cargo new my-project
cd my-project
vx cargo run
```

### 库项目

```bash
vx cargo new --lib my-lib
cd my-lib
vx cargo test
```

### 代码质量

```bash
# 格式化代码
vx cargo fmt

# 使用 Clippy 进行代码检查
vx cargo clippy

# 检查代码（不编译）
vx cargo check
```

## 提示

1. **生产环境使用稳定版**：除非需要 nightly 特性
2. **定期运行 clippy**：捕获常见错误
3. **使用 cargo fmt**：保持代码格式一致
4. **固定 rust-toolchain.toml**：确保团队一致性
