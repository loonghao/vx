# 贡献指南

感谢你有兴趣为 vx 做出贡献！

## 开始之前

1. Fork 仓库
2. 克隆你的 fork
3. 创建功能分支

```bash
git clone https://github.com/YOUR_USERNAME/vx.git
cd vx
git checkout -b feature/my-feature
```

## 开发环境

### 前提条件

- Rust 1.80+
- Git

### 构建

```bash
cargo build
```

### 跨平台构建说明

vx 使用 **rustls**（纯 Rust TLS 实现）而不是 OpenSSL，这带来了以下好处：

- **无系统依赖**：可以直接交叉编译到 musl 目标
- **更小的二进制文件**：无需打包 OpenSSL
- **行为一致**：所有平台使用相同的 TLS 实现

HTTP 客户端（`reqwest`）配置了：
- `rustls-tls`：纯 Rust TLS 后端
- `rustls-tls-native-roots`：使用系统证书存储作为信任根

这意味着你可以在不安装 OpenSSL 的情况下构建静态 musl 二进制文件：

```bash
# 构建 Linux musl（静态二进制）
cross build --release --target x86_64-unknown-linux-musl

# 构建 ARM64 musl
cross build --release --target aarch64-unknown-linux-musl
```

### 测试

```bash
cargo test
```

### 代码检查

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
```

## 项目结构

```
vx/
├── crates/
│   ├── vx-cli/         # CLI 应用
│   ├── vx-core/        # 核心类型和 traits
│   ├── vx-paths/       # 路径管理
│   ├── vx-resolver/    # 版本解析
│   ├── vx-runtime/     # 运行时管理
│   └── vx-providers/   # 工具提供者
├── book/               # 文档 (mdBook)
├── tests/              # 集成测试
└── examples/           # 示例配置
```

## 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/)：

```
feat: 添加新功能
fix: 修复问题
docs: 更新文档
refactor: 代码重构
test: 添加测试
chore: 杂项更改
```

## Pull Request

1. 确保所有测试通过
2. 更新相关文档
3. 描述你的更改
4. 链接相关 issue

## 代码风格

- 遵循 Rust 标准风格
- 使用 `cargo fmt` 格式化
- 使用 `cargo clippy` 检查

## CI 流水线

CI 流水线采用 **crate 级别的变更检测** 优化，以最小化构建时间：

### 工作原理

1. **变更检测**：CI 自动检测哪些 crate 发生了变更
2. **依赖分析**：理解 crate 之间的依赖关系图
3. **定向测试**：只测试受影响的 crate

### Crate 依赖层次

```
┌─────────────────────────────────────────────────────────────┐
│                      vx-cli (应用层)                         │
├─────────────────────────────────────────────────────────────┤
│  vx-resolver │ vx-extension │ vx-project-analyzer │ ...     │
├─────────────────────────────────────────────────────────────┤
│                    vx-runtime (基础设施层)                    │
├─────────────────────────────────────────────────────────────┤
│              vx-core │ vx-paths (基础层)                      │
└─────────────────────────────────────────────────────────────┘
```

### 影响规则

| 变更的 Crate | 受影响的 Crate |
|-------------|---------------|
| `vx-core` | 所有依赖它的 crate（runtime、resolver、extension 等） |
| `vx-paths` | runtime、resolver、env、setup、migration、args、extension、cli |
| `vx-runtime` | resolver、extension、cli、所有 providers |
| `vx-config` | project-analyzer、cli |
| Provider crates | 仅变更的 provider 和 cli |
| `vx-cli` | 仅 cli 自身 |

### CI 任务

| 任务 | 触发条件 | 描述 |
|-----|---------|------|
| `test-targeted` | 特定 crate 变更 | 仅测试受影响的 crate |
| `test-full` | 核心 crate 变更或 CI 配置变更 | 完整 workspace 测试 |
| `code-quality` | 任何 Rust 代码变更 | 格式化和 Clippy 检查 |
| `dogfood` | 任何 Rust 代码变更 | 使用真实工具的集成测试 |
| `cross-build` | 仅 main 分支 | ARM/musl 交叉编译 |
| `coverage` | 仅 main 分支 | 代码覆盖率报告 |

### 强制完整 CI

要忽略变更检测运行所有测试：

1. 进入 Actions 标签页
2. 选择 "CI" 工作流
3. 点击 "Run workflow"
4. 勾选 "Force full CI run"

## 获取帮助

- 提交 Issue
- 加入讨论

感谢你的贡献！
