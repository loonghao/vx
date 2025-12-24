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

- Rust 1.70+
- Git

### 构建

```bash
cargo build
```

### 测试

```bash
cargo test
```

### 代码检查

```bash
cargo clippy
cargo fmt --check
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

## 获取帮助

- 提交 Issue
- 加入讨论

感谢你的贡献！
