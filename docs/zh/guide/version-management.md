# 版本管理

vx 提供强大的版本管理功能，让你可以指定工具的精确版本，并自动处理工具之间的依赖约束。

## 版本语法

使用 `@` 符号在运行任何工具时指定版本：

```bash
# 指定主版本
vx node@20 --version

# 精确版本
vx node@20.10.0 --version

# 最新版本
vx node@latest --version

# Python 指定版本
vx python@3.10 --version
vx python@3.12.8 script.py
```

### 支持的版本格式

| 格式 | 示例 | 说明 |
|------|------|------|
| `major` | `node@20` | 主版本的最新版本 |
| `major.minor` | `python@3.10` | 特定次版本的最新补丁 |
| `major.minor.patch` | `node@20.10.0` | 精确版本 |
| `latest` | `uv@latest` | 最新稳定版本 |

### 各工具示例

```bash
# Node.js 生态
vx node@20 --version
vx npm@10 install
vx npx@20 create-react-app my-app

# Python 生态
vx python@3.10 --version
vx python@3.12.8 script.py
vx uv@0.4 pip install requests

# Go
vx go@1.21 version
vx go@1.22.5 build

# Yarn 指定版本
vx yarn@1.22.22 install
```

## 依赖版本约束

某些工具依赖于其他运行时。vx 会自动管理这些依赖并确保版本兼容性。

### 工作原理

当你运行一个有依赖的工具时，vx 会：

1. **检查依赖版本** - 验证已安装的依赖是否满足版本要求
2. **检测不兼容** - 识别当前版本是否超出允许范围
3. **自动安装兼容版本** - 如果需要，安装兼容的依赖版本
4. **配置环境** - 设置 PATH 以使用兼容版本

### 示例：Yarn 和 Node.js

Yarn 1.x 需要 Node.js 12-22。如果你安装了 Node.js 23+，vx 会自动使用兼容版本：

```bash
# 你安装了 Node.js 23，但 yarn 需要 Node.js ≤22
vx yarn@1.22.22 install

# vx 检测到不兼容并：
# 1. 查找或安装 Node.js 20（推荐版本）
# 2. 使用兼容的 Node.js 运行 yarn
# 3. 你的命令成功执行！
```

### 各工具的依赖约束

| 工具 | 依赖 | 最低版本 | 最高版本 | 推荐版本 |
|------|------|----------|----------|----------|
| yarn | node | 12.0.0 | 22.99.99 | 20 |
| npm | node | 14.0.0 | - | - |
| npx | node | 14.0.0 | - | - |
| pnpm | node | 16.0.0 | - | - |

::: tip 为什么需要版本约束？
某些工具与较新的运行时版本存在兼容性问题。例如：
- Yarn 1.x 在 Node.js 23+ 上存在原生模块编译问题
- 某些 npm 包需要特定的 Node.js 版本
- Python 包可能需要特定的 Python 版本

vx 会自动处理这些约束，让你无需担心兼容性问题。
:::

## 实际示例

### 使用特定版本进行 Web 开发

```bash
# 使用 Node.js 20 创建 React 应用
vx node@20 npx create-react-app my-app
cd my-app

# 使用 yarn，自动管理 Node.js 版本
vx yarn@1.22.22 install
vx yarn@1.22.22 start
```

### Python 开发

```bash
# 指定使用 Python 3.10
vx python@3.10 --version

# 使用 Python 3.12 运行脚本
vx python@3.12 script.py

# 使用 Python 3.11 以确保兼容性
vx python@3.11 -m pytest
```

### 多版本测试

```bash
# 使用不同 Node.js 版本测试代码
vx node@18 npm test
vx node@20 npm test
vx node@22 npm test

# 使用不同 Python 版本测试
vx python@3.10 -m pytest
vx python@3.11 -m pytest
vx python@3.12 -m pytest
```

### CI/CD 流水线

```yaml
# GitHub Actions 示例
jobs:
  test:
    strategy:
      matrix:
        node: [18, 20, 22]
        python: ['3.10', '3.11', '3.12']
    steps:
      - uses: actions/checkout@v4
      - name: Setup vx
        uses: loonghao/vx@v1
      - name: Test Node.js
        run: vx node@${{ matrix.node }} npm test
      - name: Test Python
        run: vx python@${{ matrix.python }} -m pytest
```

## 故障排除

### 版本未找到

如果特定版本不可用：

```bash
# 列出可用版本
vx versions node
vx versions python

# 先安装特定版本
vx install node@20.10.0
```

### 依赖冲突

如果看到依赖警告：

```bash
# 检查已安装的版本
vx list --status

# 警告显示 vx 正在做什么：
# "Dependency node version 23.0.0 is incompatible with yarn (requires: max=22.99.99)"
# "Installing compatible version: node@20"
```

### 强制使用系统版本

绕过 vx 的版本管理：

```bash
# 使用系统安装的工具
vx --use-system-path node --version
```

## 最佳实践

1. **在项目中固定版本** - 使用 `vx.toml` 确保团队一致性：

   ```toml
   [tools]
   node = "20"
   python = "3.11"
   yarn = "1.22.22"
   ```

2. **使用推荐版本** - 当工具有依赖时，使用推荐版本以获得最佳兼容性

3. **跨版本测试** - 使用版本语法在升级前测试兼容性

4. **让 vx 管理依赖** - 除非必要，不要手动覆盖依赖版本

## 下一步

- [项目环境](/zh/guide/project-environments) - 配置项目特定的工具版本
- [配置](/zh/guide/configuration) - 了解 `vx.toml` 配置
- [CLI 参考](/zh/cli/overview) - 完整命令参考
