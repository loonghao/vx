# AI Tools

vx supports tools for AI and machine learning development.

## Ollama

Run large language models locally.

```bash
vx install ollama latest

vx ollama --version
vx ollama serve              # Start Ollama server
vx ollama pull llama3.2      # Download a model
vx ollama run llama3.2       # Run a model interactively
vx ollama list               # List installed models
```

**Key Features:**

- Run LLMs locally (Llama, Mistral, Gemma, etc.)
- GPU acceleration (CUDA, ROCm)
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

## AI Development Workflow

Combine vx with AI tools for a complete development environment:

```toml
[tools]
# AI/ML tools
ollama = "latest"
uv = "latest"          # Python package manager

# Development tools
node = "22"
vscode = "latest"

[scripts]
# Start local AI server
ai-start = "ollama serve"

# Python ML environment
ml-setup = "uv venv && uv pip install torch transformers"

# Development
dev = "code ."
```

## Best Practices

1. **Model Management**: Use `ollama list` to track installed models
2. **Resource Usage**: Monitor GPU/CPU usage when running models
3. **Version Control**: Pin Ollama version for reproducible environments

```toml
[tools]
# Pin version for consistency
ollama = "0.5"
```
