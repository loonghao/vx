# AI Tools

vx supports tools for AI and machine learning development, making it easy to set up local AI environments and integrate AI workflows into your projects.

## Ollama

Run large language models locally.

```bash
vx install ollama@latest

vx ollama --version
vx ollama serve              # Start Ollama server
vx ollama pull llama3.2      # Download a model
vx ollama run llama3.2       # Run a model interactively
vx ollama list               # List installed models
```

**Key Features:**

- Run LLMs locally (Llama, Mistral, Gemma, Phi, etc.)
- GPU acceleration (CUDA, ROCm, Metal)
- REST API for integration
- Model management

**Supported Platforms:**

- Windows (x64, ARM64)
- macOS (Intel, Apple Silicon)
- Linux (x64, ARM64)

**Example Usage:**

```bash
# Start the server in background
vx ollama serve &

# Pull and run a model
vx ollama pull llama3.2
vx ollama run llama3.2 "Explain quantum computing"

# Use the API
curl http://localhost:11434/api/generate -d '{
  "model": "llama3.2",
  "prompt": "Hello!"
}'
```

**Project Configuration:**

```toml
[tools]
ollama = "latest"

[scripts]
ai-serve = "ollama serve"
ai-chat = "ollama run llama3.2"
```

## AI-Powered Development Workflows

vx is uniquely suited for AI development because it manages your entire toolchain — Python, Node.js, and AI tools — from a single configuration.

### Local LLM + Python AI Stack

Set up a complete local AI development environment:

```toml
# vx.toml
[tools]
ollama = "latest"
uv = "latest"
node = "22"

[scripts]
# Start local LLM server
ai-serve = "ollama serve"
ai-pull = "ollama pull llama3.2"

# Python ML environment
ml-setup = "uv sync"
ml-train = "uv run python train.py"
ml-eval = "uv run python evaluate.py"

# Jupyter for experimentation
notebook = "uvx jupyter lab"
```

```bash
# Set up your AI project
vx uv init ai-project && cd ai-project
vx uv add torch transformers datasets accelerate
vx uv add langchain langchain-community

# Start local LLM
vx ollama serve &
vx ollama pull llama3.2

# Run your AI app
vx uv run python app.py
```

### AI Agent Development

Build AI agents that use local or cloud LLMs:

```bash
# Set up Python project with AI dependencies
vx uv init my-agent && cd my-agent
vx uv add langchain openai anthropic
vx uv add chromadb faiss-cpu     # Vector stores
vx uv add beautifulsoup4 httpx   # Web scraping

# Run your agent
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

### RAG (Retrieval-Augmented Generation) Pipeline

```bash
# Set up RAG project
vx uv init rag-pipeline && cd rag-pipeline
vx uv add langchain chromadb sentence-transformers
vx uv add fastapi uvicorn         # API server
vx uv add unstructured pypdf      # Document loaders

# Start Ollama for local embeddings
vx ollama serve &
vx ollama pull nomic-embed-text   # Embedding model
vx ollama pull llama3.2           # Chat model

# Run your RAG API
vx uv run uvicorn main:app --reload
```

### AI + Full-Stack Application

Combine AI with a full-stack web application:

```toml
# vx.toml
[tools]
node = "22"
uv = "latest"
ollama = "latest"

[scripts]
# Frontend (Next.js / React)
frontend = "cd frontend && npm run dev"

# Backend AI API (Python FastAPI)
backend = "cd backend && uv run uvicorn main:app --reload"

# Local LLM
llm = "ollama serve"

# Start everything
dev = "just dev"  # Use just to orchestrate
```

```just
# justfile — orchestrate AI full-stack app
dev:
    # Start Ollama in background
    ollama serve &
    # Start backend API
    cd backend && uv run uvicorn main:app --reload --port 8000 &
    # Start frontend
    cd frontend && npm run dev
```

```bash
vx just dev   # Start everything with one command
```

### MCP (Model Context Protocol) Server Development

Build MCP servers to extend AI assistants:

```bash
# Python MCP server
vx uv init mcp-server && cd mcp-server
vx uv add mcp fastmcp
vx uv run python server.py

# Node.js MCP server
vx npx create-mcp-server my-server
cd my-server
vx npm install
vx npm run dev
```

```toml
# vx.toml for MCP server project
[tools]
uv = "latest"
node = "22"

[scripts]
# Start MCP server (Python)
serve = "uv run python server.py"
# Start MCP server (Node)
serve-node = "npx ts-node server.ts"
# Test with MCP inspector
inspect = "uvx mcp-inspector"
```

### ML Model Training Pipeline

```bash
# Set up training environment
vx uv init ml-training && cd ml-training
vx uv add torch torchvision torchaudio
vx uv add transformers datasets accelerate
vx uv add wandb tensorboard       # Experiment tracking
vx uv add lightning                # PyTorch Lightning

# Run training
vx uv run python train.py --epochs 10 --lr 1e-4

# Monitor with TensorBoard
vx uvx tensorboard --logdir runs/
```

### AI Code Quality Tools

Use AI-powered tools for code quality:

```bash
# Python linting and formatting
vx uvx ruff check . --fix          # AI-assisted linting
vx uvx ruff format .               # Code formatting

# Type checking
vx uvx mypy src/ --strict

# Security scanning
vx uvx bandit -r src/
vx uvx safety check
```

```toml
# vx.toml with AI-assisted code quality
[scripts]
lint = "uvx ruff check . --fix"
format = "uvx ruff format ."
typecheck = "uvx mypy src/"
security = "uvx bandit -r src/"
quality = "just quality"          # Run all checks
```

### Dagu for AI Pipeline Orchestration

Use Dagu to orchestrate complex AI workflows:

```yaml
# ai-pipeline.yaml — DAG workflow for ML pipeline
schedule: "0 2 * * *"  # Run daily at 2 AM
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
# Run the pipeline
vx dagu start ai-pipeline

# Monitor via web UI
vx dagu server  # Open http://localhost:8080
```

## CI/CD for AI Projects

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

      # Set up Python AI environment
      - run: vx uv sync

      # Run AI tests
      - run: vx uv run pytest tests/
      - run: vx uvx ruff check .

      # Run model evaluation
      - run: vx uv run python evaluate.py
```

## Best Practices

1. **Use `uv` for Python AI projects** — It's 10-100x faster than pip for installing ML dependencies like PyTorch and transformers.

2. **Local LLMs for development** — Use Ollama during development to avoid API costs. Switch to cloud APIs in production.

3. **Pin model versions** — Track which model versions you're using for reproducibility.

4. **Use `uvx` for one-off tools** — Run tools like `jupyter`, `tensorboard`, `ruff` without polluting your project environment.

5. **Orchestrate with Dagu** — For complex ML pipelines with multiple steps, use Dagu to define DAG-based workflows.

6. **Version everything in `vx.toml`** — Pin Ollama, uv, and Node.js versions for reproducible AI environments.

```toml
[tools]
# Pin versions for reproducibility
ollama = "0.5"
uv = "0.5"
node = "22"
```
