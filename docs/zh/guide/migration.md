# 迁移指南

本指南帮助你将 `vx.toml` 配置从旧版本迁移到最新格式。

## 迁移框架

vx 包含内置的迁移框架 (`vx-migration`)，可自动处理配置升级。该框架支持：

- **自动版本检测** - 检测当前配置格式
- **插件化迁移** - 可扩展的迁移系统
- **Dry-run 模式** - 应用前预览更改
- **回滚支持** - 需要时可恢复更改
- **历史记录** - 跟踪所有迁移操作

### 快速迁移命令

```bash
# 检查需要哪些迁移
vx migrate --check

# 预览更改（dry-run）
vx migrate --dry-run

# 执行迁移
vx migrate

# 带备份执行
vx migrate --backup

# 回滚到之前版本
vx migrate --rollback v1.0.0
```

### 内置迁移

| 迁移 ID | 描述 | 版本范围 |
|---------|------|----------|
| `file-rename` | 将 `vx.toml` 重命名为 `vx.toml` | 任意 |
| `config-v1-to-v2` | 将 `[tools]` 转换为 `[runtimes]` | 1.x → 2.0 |

## 版本历史

| 版本 | 发布 | 主要变更 |
|------|------|----------|
| v0.5.x | 当前 | 基本工具、脚本、环境变量 |
| v0.6.0 | 即将发布 | 钩子、服务、依赖管理 |
| v0.7.0 | 计划中 | AI 集成、文档生成 |
| v0.8.0 | 计划中 | 团队协作、远程开发 |

## 快速迁移

```bash
# 检查当前配置兼容性
vx config check

# 自动迁移到最新格式
vx config migrate --to v2

# 验证结果
vx config validate
```

## 从 v0.5.x 迁移到 v0.6.0

### 步骤 1：添加版本要求

添加 `min_version` 确保兼容性：

```toml
# 之前
[project]
name = "my-project"

# 之后
min_version = "0.6.0"

[project]
name = "my-project"
```

### 步骤 2：迁移带依赖的脚本

如果脚本之间有依赖关系，使用新的 `depends` 字段：

```toml
# 之前（手动排序）
[scripts]
build = "npm run build"
deploy = "npm run build && npm run deploy"

# 之后（显式依赖）
[scripts]
build = "npm run build"

[scripts.deploy]
command = "npm run deploy"
depends = ["build"]
```

### 步骤 3：添加生命周期钩子

将设置命令移到钩子中：

```toml
# 之前（在 README 或手动步骤中）
# 1. 运行 npm install
# 2. 运行 db:migrate
# 3. 运行 seed

# 之后（自动化）
[hooks]
post_setup = ["npm install", "vx run db:migrate", "vx run seed"]
```

### 步骤 4：定义服务

如果之前单独使用 docker-compose，可以集成进来：

```toml
# 之前（docker-compose.yml）
# services:
#   db:
#     image: postgres:16
#     ports:
#       - "5432:5432"

# 之后（vx.toml）
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
healthcheck = "pg_isready"
```

## 向后兼容性

vx 保持向后兼容：

1. **所有 v0.5.x 配置可用**：基本使用无需更改
2. **新字段可选**：按需逐步添加
3. **警告而非错误**：未知字段生成警告
4. **优雅降级**：缺失功能回退到默认值

## 验证

迁移后，验证你的配置：

```bash
# 检查错误
vx config validate

# 显示解析后的配置
vx config show

# 测试设置（不做更改）
vx setup --dry-run
```

## 常见迁移问题

### 问题：脚本未按顺序运行

**问题**：脚本相互依赖但运行顺序错误。

**解决方案**：使用 `depends` 字段：

```toml
[scripts.deploy]
command = "npm run deploy"
depends = ["build", "test"]
```

### 问题：环境变量未加载

**问题**：必需的环境变量未被验证。

**解决方案**：移到 `[env.required]`：

```toml
[env.required]
API_KEY = "外部服务的 API 密钥"
DATABASE_URL = "PostgreSQL 连接字符串"
```

## 获取帮助

- [配置参考](/zh/config/vx-toml) - 完整字段参考
- [最佳实践](/zh/guide/best-practices) - 推荐模式
- [GitHub Issues](https://github.com/loonghao/vx/issues) - 报告问题
