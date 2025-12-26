# AI 工具

vx 支持 AI 和机器学习开发工具。

## Ollama

在本地运行大型语言模型。

```bash
vx install ollama latest

vx ollama --version
vx ollama serve              # 启动 Ollama 服务器
vx ollama pull llama3.2      # 下载模型
vx ollama run llama3.2       # 交互式运行模型
vx ollama list               # 列出已安装的模型
```

**主要特性：**

- 本地运行 LLM（Llama、Mistral、Gemma 等）
- GPU 加速（CUDA、ROCm）
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

## AI 开发工作流

结合 vx 和 AI 工具构建完整的开发环境：

```toml
[tools]
# AI/ML 工具
ollama = "latest"
uv = "latest"          # Python 包管理器

# 开发工具
node = "22"
vscode = "latest"

[scripts]
# 启动本地 AI 服务器
ai-start = "ollama serve"

# Python ML 环境
ml-setup = "uv venv && uv pip install torch transformers"

# 开发
dev = "code ."
```

## 最佳实践

1. **模型管理**：使用 `ollama list` 跟踪已安装的模型
2. **资源使用**：运行模型时监控 GPU/CPU 使用情况
3. **版本控制**：锁定 Ollama 版本以获得可复现的环境

```toml
[tools]
# 锁定版本以保持一致性
ollama = "0.5"
```
