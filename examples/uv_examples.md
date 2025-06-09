# vx UV 使用示例

## 🐍 UV 插件完整指南

UV 是一个极快的 Python 包安装器和解析器，vx 通过插件系统提供了完整的 UV 支持。

## 🚀 快速开始

### 1. 检查 UV 状态
```bash
# 查看 UV 版本
vx uv --version

# 查看 UV 插件信息
vx plugin info uv
```

### 2. 创建 Python 项目
```bash
# 初始化新项目
vx uv init my-python-project
cd my-python-project

# 查看项目结构
ls -la
# 输出:
# pyproject.toml
# README.md
# src/
# tests/
```

### 3. 虚拟环境管理
```bash
# 创建虚拟环境
vx uv venv

# 创建指定 Python 版本的环境
vx uv venv --python 3.11

# 创建命名环境
vx uv venv myenv
```

## 📦 包管理

### 安装包
```bash
# 安装单个包
vx uv add requests

# 安装开发依赖
vx uv add --dev pytest black

# 安装特定版本
vx uv add "django>=4.0,<5.0"

# 安装可选依赖组
vx uv add --group test pytest coverage
```

### pip 兼容命令
```bash
# 安装包 (pip 风格)
vx uv pip install requests

# 从 requirements.txt 安装
vx uv pip install -r requirements.txt

# 列出已安装的包
vx uv pip list

# 显示包信息
vx uv pip show requests

# 卸载包
vx uv pip uninstall requests

# 导出依赖
vx uv pip freeze > requirements.txt
```

## 🏃‍♂️ 运行命令

### 基本运行
```bash
# 运行 Python 脚本
vx uv run python script.py

# 运行模块
vx uv run python -m pytest

# 运行测试
vx uv run pytest
```

### 临时依赖
```bash
# 使用临时依赖运行
vx uv run --with requests python -c "import requests; print(requests.get('https://httpbin.org/json').json())"

# 使用多个临时依赖
vx uv run --with "fastapi[all]" --with uvicorn uvicorn main:app

# 使用特定版本
vx uv run --with "numpy==1.24.0" python script.py
```

## 🔄 项目同步

### 环境同步
```bash
# 同步项目依赖
vx uv sync

# 同步包含开发依赖
vx uv sync --dev

# 同步所有可选依赖
vx uv sync --all-extras

# 强制重新安装
vx uv sync --reinstall
```

### 锁文件管理
```bash
# 更新锁文件
vx uv lock

# 升级所有依赖
vx uv lock --upgrade

# 升级特定包
vx uv lock --upgrade-package requests
```

## 🌳 依赖分析

### 依赖树
```bash
# 显示依赖树
vx uv tree

# 限制显示深度
vx uv tree --depth 2

# 显示特定包的依赖
vx uv tree --package requests
```

## 🛠️ 实际项目示例

### Web 应用项目
```bash
# 1. 创建项目
vx uv init web-app
cd web-app

# 2. 添加 Web 框架
vx uv add "fastapi[all]" uvicorn

# 3. 添加开发工具
vx uv add --dev pytest black isort mypy

# 4. 创建主应用文件
cat > src/main.py << EOF
from fastapi import FastAPI

app = FastAPI()

@app.get("/")
def read_root():
    return {"Hello": "World"}
EOF

# 5. 运行应用
vx uv run uvicorn src.main:app --reload
```

### 数据科学项目
```bash
# 1. 创建项目
vx uv init data-science
cd data-science

# 2. 添加数据科学包
vx uv add pandas numpy matplotlib seaborn jupyter

# 3. 添加开发工具
vx uv add --dev pytest ipykernel

# 4. 启动 Jupyter
vx uv run jupyter lab
```

### CLI 工具项目
```bash
# 1. 创建项目
vx uv init cli-tool
cd cli-tool

# 2. 添加 CLI 框架
vx uv add click rich

# 3. 添加构建工具
vx uv add --dev build twine

# 4. 创建 CLI 脚本
cat > src/cli_tool/main.py << EOF
import click

@click.command()
@click.option('--name', default='World', help='Name to greet.')
def hello(name):
    click.echo(f'Hello {name}!')

if __name__ == '__main__':
    hello()
EOF

# 5. 运行 CLI
vx uv run python -m cli_tool.main --name "vx user"
```

## ⚡ 性能优化技巧

### 1. 使用缓存
```bash
# UV 自动使用缓存，无需额外配置
# 缓存位置通常在 ~/.cache/uv/

# 清理缓存（如果需要）
vx uv cache clean
```

### 2. 并行安装
```bash
# UV 默认并行安装，比 pip 快 10-100 倍
# 无需特殊配置
```

### 3. 锁文件优化
```bash
# 定期更新锁文件以获得最新优化
vx uv lock --upgrade

# 使用精确版本以提高解析速度
vx uv add "requests==2.31.0"
```

## 🔧 配置和自定义

### 项目配置 (pyproject.toml)
```toml
[project]
name = "my-project"
version = "0.1.0"
description = "My awesome project"
dependencies = [
    "requests>=2.25.0",
    "click>=8.0.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0.0",
    "black>=22.0.0",
    "isort>=5.0.0",
]
test = [
    "pytest>=7.0.0",
    "coverage>=6.0.0",
]

[tool.uv]
dev-dependencies = [
    "mypy>=1.0.0",
]
```

### 环境变量
```bash
# 设置 Python 版本
export UV_PYTHON=3.11

# 设置缓存目录
export UV_CACHE_DIR=/custom/cache/path

# 禁用缓存
export UV_NO_CACHE=1
```

## 🚨 常见问题和解决方案

### 1. Python 版本问题
```bash
# 问题：找不到指定的 Python 版本
# 解决：安装对应版本或使用系统 Python
vx uv venv --python python3.11  # 使用系统 Python 3.11
vx uv venv --python 3.11         # 让 UV 查找 Python 3.11
```

### 2. 依赖冲突
```bash
# 问题：依赖版本冲突
# 解决：查看依赖树并手动指定版本
vx uv tree
vx uv add "package==specific.version"
```

### 3. 网络问题
```bash
# 问题：网络连接问题
# 解决：使用国内镜像
vx uv pip install -i https://pypi.tuna.tsinghua.edu.cn/simple/ package
```

## 📚 最佳实践

### 1. 项目结构
```
my-project/
├── pyproject.toml      # 项目配置
├── uv.lock            # 锁文件（自动生成）
├── README.md
├── src/
│   └── my_project/
│       ├── __init__.py
│       └── main.py
└── tests/
    └── test_main.py
```

### 2. 开发工作流
```bash
# 1. 克隆项目
git clone <repo>
cd <project>

# 2. 同步环境
vx uv sync

# 3. 开发
vx uv run python src/main.py

# 4. 测试
vx uv run pytest

# 5. 添加新依赖
vx uv add new-package

# 6. 提交更改（包括 uv.lock）
git add .
git commit -m "Add new feature"
```

### 3. CI/CD 集成
```yaml
# .github/workflows/test.yml
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install uv
      uses: astral-sh/setup-uv@v1
    - name: Install dependencies
      run: uv sync
    - name: Run tests
      run: uv run pytest
```

## 🎯 总结

vx 的 UV 插件提供了完整的 Python 开发环境管理功能：

- ✅ **快速安装** - 比 pip 快 10-100 倍
- ✅ **依赖解析** - 智能解决版本冲突
- ✅ **虚拟环境** - 轻松管理隔离环境
- ✅ **项目管理** - 现代化的项目配置
- ✅ **开发工具** - 集成测试、格式化等工具

通过 vx，你可以享受到 UV 的所有优势，同时保持与其他开发工具的一致体验！
