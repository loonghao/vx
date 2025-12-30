# Python 生态系统

vx 通过独立 Python 运行时和 `uv` 包管理器提供全面的 Python 支持。

## 支持的工具

| 工具 | 描述 |
|------|------|
| `python` | Python 解释器（使用 python-build-standalone） |
| `uv` | 快速 Python 包管理器 |
| `uvx` | Python 工具运行器（uv tool run） |

## Python 运行时

vx 使用 Astral 的 [python-build-standalone](https://github.com/astral-sh/python-build-standalone) 提供可移植的 Python 发行版。支持 **Python 3.7 到 3.13+**。

### 安装

```bash
# 安装最新版 Python
vx install python latest

# 安装特定版本
vx install python 3.12.8
vx install python 3.11.11
vx install python 3.10.16
vx install python 3.9.21
vx install python 3.8.20
vx install python 3.7.17

# 列出可用版本
vx list python
```

### 运行 Python

```bash
vx python --version
vx python script.py
vx python -m pytest
```

> **推荐**: 对于纯 Python 开发，我们建议使用 `uv` 而不是直接管理 Python。`uv` 提供更快的包安装、内置虚拟环境管理和自动 Python 版本管理。

## uv（推荐）

[uv](https://github.com/astral-sh/uv) 是一个极速的 Python 包和项目管理器。**我们强烈建议使用 uv 进行 Python 开发**，因为它提供：

- 比 pip 快 10-100 倍的包安装速度
- 内置虚拟环境管理
- 自动 Python 版本管理
- 使用 `pyproject.toml` 的现代项目管理

### 安装

```bash
vx install uv latest
```

### 包管理

```bash
vx uv pip install requests
vx uv pip install -r requirements.txt
vx uv pip list
```

### 虚拟环境

```bash
vx uv venv .venv
vx uv venv .venv --python 3.11
```

### 项目管理

```bash
vx uv init
vx uv add requests
vx uv sync
vx uv run python script.py
```

## uvx

uvx 可以运行 Python 工具而无需全局安装。

```bash
vx uvx ruff check .
vx uvx black .
vx uvx mypy src/
vx uvx pytest
vx uvx jupyter notebook
```

## 项目配置

```toml
[tools]
uv = "latest"

[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["requirements.txt"]
packages = ["pytest", "black", "ruff"]
git = [
    "https://github.com/user/repo.git",
]
dev = ["pytest", "mypy"]

[scripts]
test = "pytest"
lint = "uvx ruff check ."
format = "uvx black ."
typecheck = "uvx mypy src/"
```

## 常见工作流程

### 新建 Python 项目（推荐）

```bash
# 使用 uv 初始化项目
vx uv init my-project
cd my-project

# 添加依赖
vx uv add requests pandas

# 运行代码
vx uv run python main.py
```

### 使用独立 Python

```bash
# 直接安装 Python
vx install python 3.12.8

# 运行 Python
vx python --version
vx python script.py
```

### 数据科学

```bash
# 启动 Jupyter
vx uvx jupyter notebook

# 或 JupyterLab
vx uvx jupyter lab
```

### 代码质量

```bash
# 使用 ruff 检查
vx uvx ruff check .

# 使用 black 格式化
vx uvx black .

# 使用 mypy 类型检查
vx uvx mypy src/
```

### 测试

```bash
# 运行 pytest
vx uvx pytest

# 带覆盖率
vx uvx pytest --cov=src
```

## 虚拟环境设置

当在 `vx.toml` 中配置了 `[python]` 时，`vx setup` 会：

1. 创建虚拟环境
2. 从 requirements 文件安装依赖
3. 安装列出的包
4. 安装 git 依赖

```bash
vx setup
# 创建 .venv，安装依赖
```

## 提示

1. **使用 uv 进行开发**: uv 提供最佳的 Python 开发体验，具有自动版本管理和快速包安装
2. **使用 uvx 运行工具**: 使用 `uvx` 运行 linter、formatter 等工具，而不是全局安装
3. **固定 Python 版本**: 在 `vx.toml` 中指定版本以确保可重现性
4. **特定需求使用独立 Python**: 当你需要特定 Python 版本而不需要 uv 管理时
