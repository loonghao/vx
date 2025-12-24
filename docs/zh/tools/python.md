# Python

vx 支持 Python 及其生态系统工具。

## 支持的工具

| 工具 | 描述 |
|------|------|
| `python` | Python 解释器 |
| `uv` | 快速 Python 包管理器 |
| `uvx` | UV 工具运行器 |
| `pip` | Python 包安装器 |

## 使用示例

```bash
# 运行 Python
vx python --version
vx python script.py
vx python -m pytest

# 使用 uv
vx uv pip install requests
vx uv venv .venv

# 使用 uvx
vx uvx ruff check .
vx uvx black .
vx uvx mypy src/
```

## 版本管理

```bash
# 安装特定版本
vx install python@3.11
vx install python@3.12.0
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

[scripts]
test = "pytest"
lint = "uvx ruff check ."
format = "uvx black ."
```

## 虚拟环境

vx 可以自动管理 Python 虚拟环境：

```toml
[python]
version = "3.11"
venv = ".venv"
```

运行 `vx setup` 时会自动创建虚拟环境。
