# Python 生态系统

vx 通过独立 Python 运行时和 `uv` 包管理器提供全面的 Python 支持。

## 支持的工具

| 工具 | 描述 |
|------|------|
| `python` | Python 解释器（使用 python-build-standalone；Python 2.7 使用 PyPy2.7 遗留兼容包） |
| `uv` | 快速 Python 包管理器 |
| `uvx` | Python 工具运行器（uv tool run） |

## Python 运行时

vx 使用 Astral 的 [python-build-standalone](https://github.com/astral-sh/python-build-standalone) 提供可移植的 Python 发行版。Python 2.7 使用官方 PyPy2.7 可移植包，因为 python-build-standalone 不发布 CPython 2.7 构建。支持 **Python 2.7 以及 Python 3.7 到 3.13+**。

### 版本支持状态

| 版本 | 状态 | 说明 |
|------|------|------|
| Python 3.13+ | 活跃 | 最新特性 |
| Python 3.12 | 活跃 | 生产环境推荐 |
| Python 3.11 | 活跃 | 稳定版 |
| Python 3.10 | 活跃 | 稳定版 |
| Python 3.9 | 已终止 | 最后构建: 20251120 |
| Python 3.8 | 已终止 | 有限可用性 |
| Python 3.7 | 已终止 | 固定 CPython 3.7.9，支持 Windows、Linux、macOS x64 |
| Python 2.7 | 已终止 | 通过 PyPy2.7 提供遗留兼容 |

> **注意**: 已达到生命周期终点 (EOL) 的 Python 版本可用性有限。Python 2.7 面向遗留测试和迁移场景；依赖 CPython 原生扩展的项目可能仍需要系统 CPython 2.7。

### 安装

```bash
# 安装最新版 Python
vx install python@latest

# 安装特定版本
vx install python@3.12.8
vx install python@3.11.11
vx install python@3.10.16
vx install python@3.9.21
vx install python@3.8.20
vx install python@3.7.9
vx install python@2.7

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
vx install uv@latest
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
vx uv venv .venv37 --python 3.7
vx uv venv .venv27 --python 2.7
```

当 `uv` 通过 `vx uv ... --python <version>` 接收到简单版本号时，vx 会先用自己的 Python Provider 解析并安装该版本，再把解释器路径传给 uv。Python 2.7 是特殊遗留场景：uv 本身要求 Python 3.6+，因此 `vx uv venv ... --python 2.7` 会保持同样的 vx 命令形态，但底层使用 PyPA 的 Python 2.7 `virtualenv.pyz` 创建环境。

### Python 3.7 生命周期契约

uv 当前仍将 Python 3.7 列为 [Tier 2 支持](https://docs.astral.sh/uv/reference/policies/python/)。[uv 0.7.0 移除了托管下载清单中的 Python 3.7](https://github.com/astral-sh/uv/pull/13022)，解释器运行支持与可下载发行版属于不同能力。因此，当前 uv 可以使用已有 CPython 3.7，但 `uv python install 3.7` 会提示没有可下载版本。

这条版本线由 VX 负责获取。Python Provider 固定 CPython **3.7.9** 的发布版本、归档 SHA-256 和平台身份，不依赖上游当前发布清单。支持 Windows x64、使用 glibc 的 Linux x64、macOS x64；不支持的平台或其他 3.7 补丁版本会明确报错，不替换成 PyPy 或其他 Python 次版本。macOS 使用 [20200823 修复版](https://github.com/astral-sh/python-build-standalone/releases/tag/20200823)，避免 20200822 对 `libintl.dylib` 的额外依赖。

完整性检查除了解释器可执行文件，还要求标准库、`venv`、Python 头文件和链接库。只有嵌入版解释器或未完成的解压目录不满足这个契约。

历史发布者没有提供归档校验值或签名。这里的 SHA-256 是 VX 从官方 HTTPS 资产接收时计算的固定值，同时记录 GitHub 资产 ID 和字节数，用于检测下载文件及缓存归档变化，**不代表发布者签名证明**。镜像必须提供完全相同的固定归档字节；此目录条目尚未启用公共镜像。

```bash
vx install python@3.7.9
vx python@3.7.9 -c "import sys, ssl, sqlite3; assert sys.version_info[:3] == (3, 7, 9); print(sys.executable)"
vx uv venv .venv37 --python 3.7.9
vx uv pip install --python .venv37 "requests==2.31.0"
vx uv pip check --python .venv37
```

构建依赖有自己的 Python 版本要求。如果项目必须在 3.7 解释器上执行 PEP 517 后端，需要在 `pyproject.toml` 中固定兼容版本，例如：

```toml
[build-system]
requires = ["setuptools==67.8.0", "wheel==0.42.0"]
build-backend = "setuptools.build_meta"
```

```bash
vx uv build --python .venv37 --no-python-downloads --out-dir dist37
```

缓存过所需包和构建依赖后，可以给 uv 操作添加 `--offline`。离线工作流需要同时保留 VX 解释器及归档缓存与 uv 包缓存。下载成功本身不能证明某个项目的依赖或原生扩展支持 Python 3.7。

uv 还提供[自定义 Python 下载目录](https://docs.astral.sh/uv/reference/environment/#uv_python_downloads_json_url)和 [Python 归档镜像](https://docs.astral.sh/uv/reference/environment/#uv_python_install_mirror)能力，可以持续维护发行版部署而无需永久冻结旧版 uv。常规 `vx uv ... --python 3.7.9` 工作流使用 VX 获取的解释器。

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

### `vx uvx` 与 `vx uvx:<package>`

- `vx uvx <cmd> ...` 是**运行时直连执行**（等价于 `uvx <cmd>` 风格）。
- `vx uvx:<package>[::executable] ...` 是**包语法**。

```bash
# 运行时直连执行
vx uvx ruff check .

# 包语法（显式指定包和可执行文件）
vx uvx:pyinstaller::pyinstaller --version
```

> 包语法里的 `::...` 表示包上下文中的可执行文件名，不是 shell。  
> shell 语法是 `vx <runtime>::<shell>`（例如：`vx uv::cmd`）。

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
vx install python@3.12.8

# 运行 Python
vx python --version
vx python script.py
```

### 遗留多 Python 测试

为每个 Python 版本使用独立虚拟环境，并把命令放进 `justfile`：

```makefile
venv37:
    vx uv venv .venv37 --python 3.7
    vx uv pip install --python .venv37 -r requirements-py37.txt

venv27:
    vx uv venv .venv27 --python 2.7
    .venv27/bin/python -m pip install -r requirements-py27.txt

test37: venv37
    .venv37/bin/python -m pytest

test27: venv27
    .venv27/bin/python -m pytest

test-legacy: test37 test27
```

Windows 项目在 `justfile` 中使用 `.venv37\Scripts\python.exe` 和 `.venv27\Scripts\python.exe`。

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
