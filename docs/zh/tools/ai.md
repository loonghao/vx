# AI 工具

vx 支持 AI 和机器学习开发工具，让你轻松搭建本地 AI 环境并将 AI 工作流集成到项目中。

## Ollama

在本地运行大型语言模型。

```bash
vx install ollama@latest

vx ollama --version
vx ollama serve              # 启动 Ollama 服务器
vx ollama pull llama3.2      # 下载模型
vx ollama run llama3.2       # 交互式运行模型
vx ollama list               # 列出已安装的模型
```

**主要特性：**

- 本地运行 LLM（Llama、Mistral、Gemma、Phi 等）
- GPU 加速（CUDA、ROCm、Metal）
- REST API 集成
- 模型管理

**支持的平台：**

- Windows（x64、ARM64）
- macOS（Intel、Apple Silicon）
- Linux（x64、ARM64）

**使用示例：**

```bash
# 后台启动服务器
vx ollama serve &

# 拉取并运行模型
vx ollama pull llama3.2
vx ollama run llama3.2 "解释量子计算"

# 使用 API
curl http://localhost:11434/api/generate -d '{
  "model": "llama3.2",
  "prompt": "你好！"
}'
```

**项目配置：**

```toml
[tools]
ollama = "latest"

[scripts]
ai-serve = "ollama serve"
ai-chat = "ollama run llama3.2"
```

## AI 驱动的开发工作流

vx 非常适合 AI 开发，因为它可以从单一配置管理你的整个工具链 — Python、Node.js 和 AI 工具。

### 本地 LLM + Python AI 技术栈

搭建完整的本地 AI 开发环境：

```toml
# vx.toml
[tools]
ollama = "latest"
uv = "latest"
node = "22"

[scripts]
# 启动本地 LLM 服务器
ai-serve = "ollama serve"
ai-pull = "ollama pull llama3.2"

# Python ML 环境
ml-setup = "uv sync"
ml-train = "uv run python train.py"
ml-eval = "uv run python evaluate.py"

# Jupyter 实验
notebook = "uvx jupyter lab"
```

```bash
# 搭建 AI 项目
vx uv init ai-project && cd ai-project
vx uv add torch transformers datasets accelerate
vx uv add langchain langchain-community

# 启动本地 LLM
vx ollama serve &
vx ollama pull llama3.2

# 运行 AI 应用
vx uv run python app.py
```

### AI Agent 开发

构建使用本地或云端 LLM 的 AI Agent：

```bash
# 搭建带有 AI 依赖的 Python 项目
vx uv init my-agent && cd my-agent
vx uv add langchain openai anthropic
vx uv add chromadb faiss-cpu     # 向量存储
vx uv add beautifulsoup4 httpx   # 网页抓取

# 运行 Agent
vx uv run python agent.py
```

```toml
# vx.toml
[tools]
uv = "latest"
ollama = "latest"

[scripts]
agent = "uv run python agent.py"
agent-local = "OLLAMA_HOST=http://localhost:11434 uv run python agent.py"
embeddings = "uv run python generate_embeddings.py"
```

### RAG（检索增强生成）管道

```bash
# 搭建 RAG 项目
vx uv init rag-pipeline && cd rag-pipeline
vx uv add langchain chromadb sentence-transformers
vx uv add fastapi uvicorn         # API 服务器
vx uv add unstructured pypdf      # 文档加载器

# 启动 Ollama 用于本地 Embedding
vx ollama serve &
vx ollama pull nomic-embed-text   # Embedding 模型
vx ollama pull llama3.2           # 对话模型

# 运行 RAG API
vx uv run uvicorn main:app --reload
```

### AI + 全栈应用

将 AI 与全栈 Web 应用结合：

```toml
# vx.toml
[tools]
node = "22"
uv = "latest"
ollama = "latest"

[scripts]
# 前端（Next.js / React）
frontend = "cd frontend && npm run dev"

# 后端 AI API（Python FastAPI）
backend = "cd backend && uv run uvicorn main:app --reload"

# 本地 LLM
llm = "ollama serve"

# 启动全部
dev = "just dev"  # 使用 just 编排
```

```makefile
# justfile — 编排 AI 全栈应用
dev:
    # 后台启动 Ollama
    ollama serve &
    # 启动后端 API
    cd backend && uv run uvicorn main:app --reload --port 8000 &
    # 启动前端
    cd frontend && npm run dev
```

```bash
vx just dev   # 一个命令启动所有服务
```

### MCP（模型上下文协议）服务器开发

构建 MCP 服务器来扩展 AI 助手能力：

```bash
# Python MCP 服务器
vx uv init mcp-server && cd mcp-server
vx uv add mcp fastmcp
vx uv run python server.py

# Node.js MCP 服务器
vx npx create-mcp-server my-server
cd my-server
vx npm install
vx npm run dev
```

```toml
# MCP 服务器项目的 vx.toml
[tools]
uv = "latest"
node = "22"

[scripts]
# 启动 MCP 服务器（Python）
serve = "uv run python server.py"
# 启动 MCP 服务器（Node）
serve-node = "npx ts-node server.ts"
# 使用 MCP inspector 测试
inspect = "uvx mcp-inspector"
```

### ML 模型训练管道

```bash
# 搭建训练环境
vx uv init ml-training && cd ml-training
vx uv add torch torchvision torchaudio
vx uv add transformers datasets accelerate
vx uv add wandb tensorboard       # 实验跟踪
vx uv add lightning                # PyTorch Lightning

# 运行训练
vx uv run python train.py --epochs 10 --lr 1e-4

# 使用 TensorBoard 监控
vx uvx tensorboard --logdir runs/
```

### AI 代码质量工具

使用 AI 驱动的工具提升代码质量：

```bash
# Python 检查和格式化
vx uvx ruff check . --fix          # AI 辅助代码检查
vx uvx ruff format .               # 代码格式化

# 类型检查
vx uvx mypy src/ --strict

# 安全扫描
vx uvx bandit -r src/
vx uvx safety check
```

```toml
# 带有 AI 辅助代码质量的 vx.toml
[scripts]
lint = "uvx ruff check . --fix"
format = "uvx ruff format ."
typecheck = "uvx mypy src/"
security = "uvx bandit -r src/"
quality = "just quality"          # 运行所有检查
```

### 使用 Dagu 编排 AI 管道

使用 Dagu 编排复杂的 AI 工作流：

```yaml
# ai-pipeline.yaml — ML 管道的 DAG 工作流
schedule: "0 2 * * *"  # 每天凌晨 2 点运行
steps:
  - name: fetch-data
    command: uv run python fetch_data.py
  - name: preprocess
    command: uv run python preprocess.py
    depends:
      - fetch-data
  - name: train
    command: uv run python train.py --epochs 50
    depends:
      - preprocess
  - name: evaluate
    command: uv run python evaluate.py
    depends:
      - train
  - name: deploy
    command: uv run python deploy.py
    depends:
      - evaluate
    preconditions:
      - condition: "`cat metrics.json | jq '.accuracy'` > 0.95"
```

```bash
# 运行管道
vx dagu start ai-pipeline

# 通过 Web UI 监控
vx dagu server  # 打开 http://localhost:8080
```

## AI 项目的 CI/CD

### GitHub Actions

```yaml
name: AI Pipeline
on:
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: loonghao/vx@main
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}

      # 搭建 Python AI 环境
      - run: vx uv sync

      # 运行 AI 测试
      - run: vx uv run pytest tests/
      - run: vx uvx ruff check .

      # 运行模型评估
      - run: vx uv run python evaluate.py
```

## 最佳实践

1. **使用 `uv` 管理 Python AI 项目** — 安装 PyTorch 和 transformers 等 ML 依赖比 pip 快 10-100 倍。

2. **开发时使用本地 LLM** — 开发阶段使用 Ollama 避免 API 费用，生产环境切换到云端 API。

3. **固定模型版本** — 记录使用的模型版本以确保可重现性。

4. **使用 `uvx` 运行一次性工具** — 运行 `jupyter`、`tensorboard`、`ruff` 等工具而不污染项目环境。

5. **使用 Dagu 编排** — 对于有多个步骤的复杂 ML 管道，使用 Dagu 定义基于 DAG 的工作流。

6. **在 `vx.toml` 中管理所有版本** — 固定 Ollama、uv 和 Node.js 版本以获得可重现的 AI 环境。

```toml
[tools]
# 固定版本以确保可重现性
ollama = "0.5"
uv = "0.5"
node = "22"
```
