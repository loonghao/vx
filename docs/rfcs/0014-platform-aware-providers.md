# RFC 0014: Platform-Aware Provider System

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2026-01-07
> **目标版本**: v0.10.0
> **依赖**: RFC 0012 (Provider Manifest), RFC 0013 (Manifest-Driven Registration)

## 摘要

本 RFC 提出增强 `provider.toml` 清单系统，支持声明式的平台约束，使 vx 能够：
1. 在 `vx list` 等命令中显示平台兼容性信息
2. 在不支持的平台上提供友好的错误提示
3. （可选）在编译时根据目标平台过滤 Provider

## 动机

### 当前问题

1. **用户困惑**：用户在 Linux 上执行 `vx msvc --version` 会得到不明确的错误
2. **信息缺失**：`vx list` 不显示哪些工具是平台特定的
3. **浪费资源**：所有平台都编译所有 Provider，即使某些永远不会用到

### 目标

- 在 `provider.toml` 中声明平台支持
- 运行时检测平台兼容性并提供清晰反馈
- 在 CLI 输出中显示平台信息
- （可选）条件编译优化二进制大小

## 设计

### 1. provider.toml 平台声明

#### 1.1 Provider 级别平台约束

```toml
# msvc/provider.toml
[provider]
name = "msvc"
description = "Microsoft Visual C++ Compiler"

# Provider 级别的平台约束 - 整个 Provider 只在这些平台可用
[provider.platforms]
os = ["windows"]

[[runtimes]]
name = "cl"
executable = "cl"
```

#### 1.2 Runtime 级别平台约束

```toml
# xcode/provider.toml
[provider]
name = "xcode"
description = "Apple Xcode Command Line Tools"

[[runtimes]]
name = "xcodebuild"
executable = "xcodebuild"

# Runtime 级别的平台约束
[runtimes.platforms]
os = ["macos"]

[[runtimes]]
name = "xcrun"
executable = "xcrun"

[runtimes.platforms]
os = ["macos"]
```

#### 1.3 完整平台规格

```toml
[provider.platforms]
# 支持的操作系统: windows, macos, linux
os = ["windows", "linux"]

# 支持的架构: x86_64, aarch64, x86
arch = ["x86_64", "aarch64"]

# 排除特定组合
exclude = [
    { os = "linux", arch = "x86" }
]
```

### 2. 数据结构

```rust
// vx-manifest/src/platform.rs

/// 平台约束定义
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformConstraint {
    /// 支持的操作系统
    #[serde(default)]
    pub os: Vec<Os>,
    
    /// 支持的架构
    #[serde(default)]
    pub arch: Vec<Arch>,
    
    /// 排除的平台组合
    #[serde(default)]
    pub exclude: Vec<PlatformExclusion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformExclusion {
    pub os: Option<Os>,
    pub arch: Option<Arch>,
}

impl PlatformConstraint {
    /// 检查当前平台是否满足约束
    pub fn is_current_platform_supported(&self) -> bool {
        let current_os = Os::current();
        let current_arch = Arch::current();
        
        // 如果没有指定约束，默认支持所有平台
        if self.os.is_empty() && self.arch.is_empty() {
            return true;
        }
        
        // 检查 OS
        if !self.os.is_empty() && !self.os.contains(&current_os) {
            return false;
        }
        
        // 检查架构
        if !self.arch.is_empty() && !self.arch.contains(&current_arch) {
            return false;
        }
        
        // 检查排除列表
        for exclusion in &self.exclude {
            let os_match = exclusion.os.map_or(true, |os| os == current_os);
            let arch_match = exclusion.arch.map_or(true, |arch| arch == current_arch);
            if os_match && arch_match {
                return false;
            }
        }
        
        true
    }
    
    /// 生成人类可读的平台描述
    pub fn description(&self) -> Option<String> {
        if self.os.is_empty() && self.arch.is_empty() {
            return None;
        }
        
        let os_str = if self.os.len() == 1 {
            match self.os[0] {
                Os::Windows => "Windows only",
                Os::MacOS => "macOS only",
                Os::Linux => "Linux only",
            }
        } else if !self.os.is_empty() {
            let names: Vec<_> = self.os.iter().map(|o| o.as_str()).collect();
            return Some(format!("{} only", names.join("/")));
        } else {
            return None;
        };
        
        Some(os_str.to_string())
    }
}
```

### 3. CLI 集成

#### 3.1 `vx list` 输出

```
$ vx list

Available runtimes:

  Node.js Ecosystem:
    node          Node.js JavaScript runtime
    npm           Node.js package manager
    npx           Node.js package runner

  Python Ecosystem:
    python        Python programming language
    pip           Python package installer
    uv            Fast Python package manager

  Platform-Specific:
    msvc          Microsoft Visual C++ Compiler (Windows only)
    xcodebuild    Xcode build tool (macOS only)
    choco         Chocolatey package manager (Windows only)

  Build Tools:
    cmake         Cross-platform build system
    ninja         Small build system with a focus on speed
```

#### 3.2 不支持平台的错误提示

```
$ vx msvc --version  # 在 Linux 上执行

Error: 'msvc' is not available on Linux

  msvc (Microsoft Visual C++ Compiler) is only available on Windows.

  Alternative tools for Linux:
    - gcc: GNU Compiler Collection
    - clang: LLVM C/C++ compiler

  To see all available runtimes: vx list
```

### 4. 实现方案

#### Phase 1: 运行时检测 (v0.10.0)

1. **扩展 `provider.toml` 解析**
   - 添加 `PlatformConstraint` 到 `ProviderMeta` 和 `RuntimeDef`
   - 实现 `is_current_platform_supported()` 方法

2. **更新 `vx list`**
   - 按平台分组显示 runtime
   - 添加平台标签 `(Windows only)` 等

3. **改进错误提示**
   - 检测平台不匹配时提供清晰错误
   - 建议替代工具

#### Phase 2: 条件编译 (v0.11.0, 可选)

1. **修改 `build.rs`**
   - 读取 `provider.toml` 的平台约束
   - 根据 `target_os` 过滤不兼容的 Provider

2. **条件特性**
   ```toml
   # Cargo.toml
   [features]
   default = ["all-providers"]
   all-providers = ["windows-providers", "macos-providers", "linux-providers"]
   windows-providers = ["vx-provider-msvc", "vx-provider-choco"]
   macos-providers = ["vx-provider-xcode"]
   ```

3. **二进制大小优化**
   - Windows 二进制不包含 xcode provider
   - Linux 二进制不包含 msvc/choco provider

### 5. 示例 Provider 配置

#### msvc (Windows only)

```toml
[provider]
name = "msvc"
description = "Microsoft Visual C++ Compiler"
homepage = "https://visualstudio.microsoft.com"

[provider.platforms]
os = ["windows"]

[[runtimes]]
name = "cl"
description = "MSVC C/C++ Compiler"
executable = "cl"

[[runtimes]]
name = "link"
description = "MSVC Linker"
executable = "link"

[[runtimes]]
name = "nmake"
description = "Microsoft Program Maintenance Utility"
executable = "nmake"
```

#### xcode (macOS only)

```toml
[provider]
name = "xcode"
description = "Apple Xcode Command Line Tools"
homepage = "https://developer.apple.com/xcode/"

[provider.platforms]
os = ["macos"]

[[runtimes]]
name = "xcodebuild"
description = "Build Xcode projects"
executable = "xcodebuild"

[[runtimes]]
name = "xcrun"
description = "Run Xcode developer tools"
executable = "xcrun"

[[runtimes]]
name = "swift"
description = "Swift programming language"
executable = "swift"
aliases = ["swiftc"]
```

#### cmake (跨平台)

```toml
[provider]
name = "cmake"
description = "Cross-platform build system"
homepage = "https://cmake.org"

# 不指定 platforms = 默认支持所有平台

[[runtimes]]
name = "cmake"
description = "CMake build system generator"
executable = "cmake"

[[runtimes]]
name = "ctest"
description = "CMake test driver"
executable = "ctest"

[[runtimes]]
name = "cpack"
description = "CMake packaging tool"
executable = "cpack"
```

### 6. API 设计

```rust
// vx-runtime/src/registry.rs

impl ProviderRegistry {
    /// 获取当前平台支持的所有 runtime
    pub fn supported_runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        self.providers()
            .iter()
            .filter(|p| p.is_platform_supported())
            .flat_map(|p| p.runtimes())
            .filter(|r| r.is_platform_supported())
            .collect()
    }
    
    /// 获取 runtime，如果平台不支持返回特定错误
    pub fn get_runtime_checked(&self, name: &str) -> Result<Arc<dyn Runtime>, PlatformError> {
        if let Some(runtime) = self.get_runtime(name) {
            if runtime.is_platform_supported() {
                Ok(runtime)
            } else {
                Err(PlatformError::UnsupportedPlatform {
                    runtime: name.to_string(),
                    constraint: runtime.platform_constraint(),
                    current: Platform::current(),
                })
            }
        } else {
            Err(PlatformError::NotFound(name.to_string()))
        }
    }
}

// Provider trait 扩展
pub trait Provider: Send + Sync {
    // ... existing methods ...
    
    /// 获取平台约束
    fn platform_constraint(&self) -> Option<&PlatformConstraint> {
        None // 默认无约束
    }
    
    /// 检查当前平台是否支持
    fn is_platform_supported(&self) -> bool {
        self.platform_constraint()
            .map_or(true, |c| c.is_current_platform_supported())
    }
}
```

### 7. 迁移计划

#### 需要添加平台约束的 Provider

| Provider | 平台约束 | 说明 |
|----------|----------|------|
| msvc | Windows only | Microsoft Visual C++ |
| choco | Windows only | Chocolatey 包管理器 |
| xcode | macOS only | Apple 开发工具 (待添加) |
| brew | macOS only | Homebrew (待添加) |

#### 跨平台 Provider (无需修改)

- node, go, rust, python, uv, java, cmake, ninja, docker, kubectl, terraform 等

## 替代方案

### 方案 A: 仅运行时检测 (推荐)

- 优点：实现简单，向后兼容
- 缺点：二进制包含所有 Provider 代码

### 方案 B: 编译时过滤

- 优点：更小的二进制
- 缺点：实现复杂，需要修改 CI 构建流程

### 方案 C: 动态加载

- 优点：最灵活，按需加载
- 缺点：实现最复杂，需要插件系统支持

**建议**：Phase 1 采用方案 A，Phase 2 根据需求决定是否实现方案 B。

## 安全考虑

- 平台检测使用 Rust 标准库，无安全风险
- 条件编译不影响运行时安全性

## 测试计划

1. **单元测试**
   - `PlatformConstraint::is_current_platform_supported()` 各种组合
   - 平台描述生成

2. **集成测试**
   - `vx list` 在不同平台的输出
   - 平台不支持时的错误提示

3. **CI 测试**
   - Windows/macOS/Linux 矩阵测试
   - 验证平台特定 Provider 行为

## 参考

- [RFC 0012: Provider Manifest](./0012-provider-manifest.md)
- [RFC 0013: Manifest-Driven Registration](./0013-manifest-driven-registration.md)
- [Rust 条件编译](https://doc.rust-lang.org/reference/conditional-compilation.html)
