# macOS 构建问题修复：sevenz-rust 链接器错误

## 问题描述

在 macOS (aarch64-apple-darwin) 上构建时，`sevenz-rust` crate 会导致链接失败：

```
error: linking with `cc` failed: exit status: 1
clang: error: invalid linker name in argument '-fuse-ld=lld'
```

**根本原因**：`sevenz-rust` crate 在其构建配置中使用了 `-fuse-ld=lld` 参数，但 macOS 的 clang 不支持这个参数。

## 解决方案

将 `sevenz-rust` 改为可选依赖，通过 `extended-formats` feature 控制。在 macOS 构建时默认禁用 7z 支持。

### 修改的文件

#### 1. `crates/vx-installer/Cargo.toml`

```toml
# 将 sevenz-rust 改为可选依赖
sevenz-rust = { version = "0.6", optional = true }

[features]
default = []
extended-formats = ["sevenz-rust"]  # 7z 支持作为可选 feature
progress = ["indicatif"]
cdn-acceleration = ["turbo-cdn"]
```

#### 2. `crates/vx-installer/src/formats/mod.rs`

```rust
// 条件编译 sevenz 模块
#[cfg(feature = "extended-formats")]
pub mod sevenz;

// 在 ArchiveExtractor::new() 中条件添加 handler
#[cfg(feature = "extended-formats")]
handlers.insert(2, Box::new(sevenz::SevenZipHandler::new()));
```

#### 3. `crates/vx-runtime-archive/Cargo.toml`

```toml
sevenz-rust = { version = "0.6", optional = true }

[features]
default = []
extended-formats = ["sevenz-rust"]
```

#### 4. `crates/vx-runtime-archive/src/lib.rs`

```rust
// 条件编译 SevenZ 枚举变体
pub enum ArchiveFormat {
    TarGz,
    TarXz,
    TarZst,
    Zip,
    #[cfg(feature = "extended-formats")]
    SevenZ,
}

// 条件编译相关方法
#[cfg(feature = "extended-formats")]
fn extract_7z(&self, archive: &Path, dest: &Path) -> Result<()> {
    // ...
}
```

#### 5. `crates/vx-runtime-http/Cargo.toml`

```toml
sevenz-rust = { version = "0.6", optional = true }

[features]
default = []
cdn-acceleration = ["turbo-cdn"]
extended-formats = ["sevenz-rust"]
```

#### 6. `crates/vx-runtime-http/src/installer.rs`

```rust
Some("7z") => {
    #[cfg(feature = "extended-formats")]
    {
        sevenz_rust::decompress_file(archive, dest)
            .map_err(|e| anyhow::anyhow!("Failed to extract 7z archive: {}", e))?;
    }
    #[cfg(not(feature = "extended-formats"))]
    {
        return Err(anyhow::anyhow!(
            "7z extraction is not supported in this build. Please use a build with extended-formats feature enabled."
        ));
    }
}
```

## 使用方式

### 默认构建（无 7z 支持）

```bash
cargo build --release
```

这将在所有平台上成功构建，但不包含 7z 支持。

### 启用 7z 支持（仅在非 macOS 平台）

```bash
cargo build --release --features extended-formats
```

在 Windows 和 Linux 上可以启用此 feature 来获得 7z 支持。

### CI 配置建议

```yaml
# .github/workflows/build.yml
- name: Build (macOS)
  if: matrix.os == 'macos-latest'
  run: cargo build --release
  # macOS 不启用 extended-formats

- name: Build (Windows/Linux)
  if: matrix.os != 'macos-latest'
  run: cargo build --release --features extended-formats
  # Windows 和 Linux 启用 7z 支持
```

## 影响

1. **macOS 用户**：无法解压 .7z 格式的工具包，但可以使用其他格式（.tar.gz, .zip 等）
2. **Windows/Linux 用户**：可以选择启用 7z 支持
3. **工具提供者**：建议优先提供 .tar.gz 或 .zip 格式，而非 .7z

## 替代方案

如果未来需要在 macOS 上支持 7z，可以考虑：

1. 使用其他 7z 库（如 `sevenz` 或 `lzma-rs`）
2. 通过系统命令调用 `7z` 工具（需要用户安装）
3. 等待 `sevenz-rust` 修复 macOS 链接器问题

## 相关链接

- [sevenz-rust GitHub](https://github.com/dyz1990/sevenz-rust)
- [Rust linker configuration](https://doc.rust-lang.org/cargo/reference/config.html#targettriplelinker)
