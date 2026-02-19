# 贡献指南

感谢你有兴趣为 vx 做出贡献！

## 开始之前

### 前提条件

- Rust 1.80+
- Git

### 克隆与构建

```bash
git clone https://github.com/loonghao/vx.git
cd vx
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

### 运行测试

```bash
cargo test
```

### 运行 Clippy

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### 格式化代码

```bash
cargo fmt
```

## 开发工作流

### 1. 创建分支

```bash
git checkout -b feature/my-feature
```

### 2. 安装 Pre-commit Hooks

vx 使用 [prek](https://prek.j178.dev/) 进行 pre-commit 检查。克隆仓库后执行一次安装：

```bash
vx prek install
```

这会安装在每次提交前自动检查代码的 hooks。详情请参阅 [Pre-commit Hooks](pre-commit-hooks)。

### 3. 进行修改

- 编写代码
- 添加测试
- 更新文档

### 4. 本地测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 带输出运行
cargo test -- --nocapture
```

### 5. 检查代码质量

```bash
# 格式化
cargo fmt

# 代码检查
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 检查文档
cargo doc --all-features --no-deps

# 手动运行所有 pre-commit hooks
vx prek run --all-files
```

### 6. 保持 workspace-hack 同步

在任何 `Cargo.toml` 中添加或更新依赖后，需要重新生成 workspace-hack：

```bash
just hakari-generate
# 或手动执行：
cargo hakari generate
cargo hakari manage-deps
```

pre-commit hook 会在你忘记时自动捕获这个问题。

### 7. 提交 PR

- 推送你的分支
- 创建 Pull Request
- 填写 PR 模板

## 代码规范

### Rust 风格

- 遵循 Rust 标准约定
- 使用 `rustfmt` 格式化
- 解决所有 Clippy 警告
- 为公开 API 添加文档

### 测试

- 将测试放在 `tests/` 目录中
- 使用 `rstest` 进行参数化测试
- 追求良好的测试覆盖率

### 文档

- 为公开函数和类型添加文档
- 在文档注释中包含示例
- 根据需要更新用户文档

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

## 添加新 Provider

1. 在 `crates/vx-providers/` 中创建 crate
2. 实现 `Provider` trait
3. 添加测试
4. 在 `vx-cli/src/registry.rs` 中注册
5. 更新文档

详情请参阅 [Plugin Development](plugin-development)。

## 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/)：

```
feat: 添加新功能
fix: 修复问题
docs: 更新文档
test: 添加测试
refactor: 代码重构
```

## Pull Request 流程

1. 确保 CI 通过
2. 更新文档
3. 为新功能添加测试
4. 请求代码审查
5. 处理反馈意见

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

## 依赖限制

部分依赖存在版本限制，无法自动升级：

### bincode（v1.3 → v3.x 被阻止）

`bincode` crate 被锁定在 v1.3，原因如下：

- **msvc-kit**（用于 Windows 上 MSVC 构建工具的安装）依赖 `bincode ^1.3`
- `bincode v3` 具有完全不同的 API（该 crate 已被重构）
- 在 `msvc-kit` 发布支持 `bincode v3` 的版本之前，我们无法升级

**解决方案**：Renovate 已配置为跳过 `bincode` 的主版本更新。详情请参阅 [PR #378](https://github.com/loonghao/vx/pull/378)。

**后续处理**：监控 `msvc-kit` 的新版本以获取 `bincode v3` 支持。一旦可用：
1. 将 `msvc-kit` 更新到新版本
2. 移除 Renovate 中关于 `bincode` 的规则
3. 将代码迁移到 `bincode v3` API

## 报告问题

报告 Bug 时请提供：

1. 检查现有 Issues
2. 包含 vx 版本（`vx --version`）
3. 包含操作系统和 Shell 信息
4. 提供复现步骤
5. 包含错误信息

## 功能请求

1. 检查现有 Issues/Discussions
2. 描述使用场景
3. 如有可能，提出解决方案

## 社区

- [GitHub Issues](https://github.com/loonghao/vx/issues)
- [GitHub Discussions](https://github.com/loonghao/vx/discussions)

## 许可证

通过贡献，你同意你的贡献将以 MIT 许可证授权。
