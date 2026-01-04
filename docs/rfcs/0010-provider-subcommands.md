# RFC 0010: Provider 子命令架构

## 概述

本 RFC 定义了 vx 中复杂 Provider（如 MSVC）的子命令架构设计，解决多工具 Provider 的命名冲突和版本管理问题。

## 动机

某些 Provider 提供多个可执行工具，例如：

- **MSVC**: `cl.exe`, `link.exe`, `lib.exe`, `nmake.exe`, `ml64.exe`, `dumpbin.exe` 等
- **Python (uv)**: `python`, `pip`, `uvx` 等
- **Rust**: `cargo`, `rustc`, `rustup`, `rustfmt`, `clippy` 等

直接将所有工具注册为顶级别名会导致：
1. 命名冲突（如 `link` 与 Linux 系统命令冲突）
2. 不清楚工具来源
3. 版本管理困难

## 设计方案

### 1. 命名空间调用（推荐方式）

```bash
# 格式: vx <provider> <subcommand> [args...]
vx msvc cl main.cpp -o main.exe
vx msvc link main.obj
vx msvc nmake

# 带版本指定
vx msvc@14.40 cl main.cpp
vx msvc@14.29 link main.obj
```

### 2. 直接别名（无冲突的常用工具）

对于不会产生冲突的常用工具，可以注册顶级别名：

```bash
# 这些是安全的别名
vx cl main.cpp          # 等同于 vx msvc cl
vx nmake                # 等同于 vx msvc nmake

# 这些不注册别名（避免冲突）
# link, lib 等不作为顶级命令
```

### 3. Provider 实现

```rust
impl Provider for MsvcProvider {
    fn runtimes(&self) -> Vec<RuntimeInfo> {
        vec![
            // 主 Runtime - 支持子命令
            RuntimeInfo::new("msvc")
                .with_description("MSVC Build Tools")
                .with_subcommands(&[
                    SubCommand::new("cl", "C/C++ compiler"),
                    SubCommand::new("link", "Linker"),
                    SubCommand::new("lib", "Library manager"),
                    SubCommand::new("nmake", "Make utility"),
                    SubCommand::new("ml64", "MASM assembler"),
                    SubCommand::new("dumpbin", "Binary file dumper"),
                    SubCommand::new("editbin", "Binary file editor"),
                ]),
            
            // 安全的顶级别名
            RuntimeInfo::new("cl")
                .alias_of("msvc", "cl")
                .with_description("MSVC C/C++ compiler"),
            
            RuntimeInfo::new("nmake")
                .alias_of("msvc", "nmake")
                .with_description("MSVC Make utility"),
        ]
    }
}
```

## 子命令路由

### 执行流程

```
用户输入: vx msvc cl main.cpp -o main.exe
         ↓
1. 解析 Provider: msvc
2. 解析子命令: cl
3. 查找可执行文件: ~/.vx/store/msvc/14.40/VC/Tools/MSVC/14.40.33807/bin/Hostx64/x64/cl.exe
4. 设置环境变量 (INCLUDE, LIB, PATH)
5. 执行: cl.exe main.cpp -o main.exe
```

### 环境变量设置

MSVC 工具需要特定的环境变量：

```bash
# vx msvc cl 自动设置
INCLUDE=C:\Users\xxx\.vx\store\msvc\14.40\VC\Tools\MSVC\14.40.33807\include;...
LIB=C:\Users\xxx\.vx\store\msvc\14.40\VC\Tools\MSVC\14.40.33807\lib\x64;...
PATH=C:\Users\xxx\.vx\store\msvc\14.40\VC\Tools\MSVC\14.40.33807\bin\Hostx64\x64;...
```

## 版本管理

### vx.toml 配置

```toml
[tools]
msvc = "14.40"           # 指定 MSVC 版本
msvc.sdk = "10.0.22621"  # 可选：指定 Windows SDK 版本

[tools.msvc]
version = "14.40"
sdk_version = "10.0.22621"
host_arch = "x64"
target_arch = "x64"
```

### 多版本共存

```bash
# 安装多个版本
vx install msvc 14.40
vx install msvc 14.29

# 使用特定版本
vx msvc@14.40 cl main.cpp
vx msvc@14.29 cl legacy.cpp
```

## 与其他 Provider 的对比

| Provider | 子命令方式 | 直接别名 | 说明 |
|----------|-----------|---------|------|
| **msvc** | `vx msvc cl` | `vx cl` | cl, nmake 安全；link, lib 不注册 |
| **node** | `vx node npm` | `vx npm` | 所有工具都安全 |
| **rust** | `vx rust cargo` | `vx cargo` | 所有工具都安全 |
| **python** | `vx python pip` | `vx pip` | pip 可能冲突，建议用 uv |

## 实现计划

1. **Phase 1**: 在 `vx-runtime` 中添加 `SubCommand` 支持
2. **Phase 2**: 更新 MSVC Provider 实现子命令
3. **Phase 3**: 实现环境变量自动设置
4. **Phase 4**: 更新文档和示例

## 参考

- [PortableBuildTools](https://github.com/Data-Oriented-House/PortableBuildTools)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
