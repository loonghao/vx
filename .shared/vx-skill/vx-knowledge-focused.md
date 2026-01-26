# VX 核心知识库（聚焦版）

> **设计理念**：聚焦核心价值，避免大而全

## 🎯 VX 是什么？

vx 是一个**轻量级的工具执行层**，提供统一的命令接口，让 AI 和开发者可以无缝使用各种开发工具。

**核心价值：**
- ✅ 统一的命令接口：`vx <tool> [args]`
- ✅ 自动版本管理：每个项目独立的工具版本
- ✅ 零配置使用：无需手动安装和配置工具
- ✅ AI 友好：AI 助手可以直接使用，无需环境设置

**不是：**
- ❌ 不是所有工具的官方实现
- ❌ 不是大而全的工具管理器
- ❌ 不是试图统一所有工具环境

## 💡 核心概念

### 1. 工具执行层

vx 只做一件事：**提供统一的工具执行接口**

```bash
# 统一接口
vx node --version
vx python script.py
vx go build
vx cargo test
```

**好处：**
- AI 助手只需要知道 `vx <tool>` 这个模式
- 无需关心工具安装、版本、平台差异
- 一致的执行环境

### 2. 项目级版本管理

每个项目可以有独立的工具版本：

```toml
# vx.toml
[tools]
node = "20"      # 项目使用 Node.js 20
python = "3.12"  # 项目使用 Python 3.12
```

**好处：**
- 团队协作：新成员 `vx setup` 即可
- 多项目共存：不同项目不同版本
- 可复现环境：版本锁定

### 3. 社区驱动

**官方维护核心工具（5-10个）：**
- Node.js, Python, Go, Rust, Docker

**社区贡献其他工具：**
- 通过工具注册表添加
- 社区维护和更新
- 官方审核质量

## 📋 常用命令

### 基础使用

```bash
# 执行工具（自动安装）
vx node --version
vx python script.py

# 项目初始化
vx init              # 创建 vx.toml
vx setup             # 安装项目工具
vx dev               # 进入开发环境
```

### 工具管理

```bash
# 安装工具
vx install node@20

# 查看工具
vx list
vx which node
vx versions node
```

### 项目脚本

```bash
# 运行项目脚本
vx run dev
vx run test
vx run build
```

## 🎯 常见场景

### 场景 1: 新项目设置

```bash
# 1. 初始化项目
vx init

# 2. 编辑 vx.toml，添加工具
# [tools]
# node = "20"
# python = "3.12"

# 3. 安装工具
vx setup

# 4. 开始开发
vx dev
```

### 场景 2: AI 助手使用

```bash
# AI 可以直接使用，无需环境设置
vx npx create-react-app my-app
vx uvx ruff check .
vx cargo build --release
```

### 场景 3: 团队协作

```bash
# 新成员加入
git clone <repo>
cd <project>
vx setup  # 一键设置环境
vx dev    # 开始开发
```

## 🔧 配置示例

### 基础配置

```toml
# vx.toml
[tools]
node = "20"
python = "3.12"

[scripts]
dev = "npm run dev"
test = "npm test"
```

### 多语言项目

```toml
[tools]
node = "20"
python = "3.12"
go = "1.21"
docker = "latest"

[scripts]
frontend = "cd frontend && npm run dev"
backend = "cd backend && go run main.go"
```

## ⚠️ 故障排查

### 工具未找到

```bash
# 检查工具是否安装
vx which <tool>

# 安装工具
vx install <tool>

# 检查项目配置
cat vx.toml
```

### 版本问题

```bash
# 查看当前版本
vx which <tool>

# 切换版本
vx switch <tool>@<version>

# 查看可用版本
vx versions <tool>
```

### 环境问题

```bash
# 使用系统工具
vx --use-system-path <tool>

# 检查 shell 集成
vx shell init
```

## 📚 更多资源

- **官方文档**：https://github.com/loonghao/vx
- **工具注册表**：查看社区贡献的工具
- **问题反馈**：https://github.com/loonghao/vx/issues

## 🎯 设计哲学

**聚焦核心：**
- 只做工具执行层
- 不试图统一所有工具
- 让社区贡献工具支持

**简单易用：**
- 零学习成本
- 一致的接口
- 清晰的文档

**社区驱动：**
- 官方维护核心
- 社区贡献扩展
- 共同建设生态
