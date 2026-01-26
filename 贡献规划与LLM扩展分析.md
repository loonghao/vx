# vx 贡献规划与 LLM Provider 扩展分析

## 📋 贡献规划

### 贡献指南位置

- **英文版**：`docs/advanced/contributing.md`
- **中文版**：`docs/zh/advanced/contributing.md`

### 核心贡献流程

#### 1. 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/loonghao/vx.git
cd vx

# 构建项目
cargo build

# 运行测试
cargo test

# 代码检查
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt
```

#### 2. 添加新 Provider 的步骤

根据 `docs/advanced/contributing.md`：

1. **创建 Provider Crate**
   ```bash
   mkdir -p crates/vx-providers/mytool/src
   ```

2. **实现 Provider Trait**
   - 实现 `Provider` trait
   - 实现 `Runtime` trait

3. **添加测试**
   - 在 `tests/` 目录下添加测试

4. **注册 Provider**
   - 在 `vx-cli/src/registry.rs` 中注册
   - 在 `Cargo.toml` 中添加依赖

5. **更新文档**
   - 更新 `docs/tools/` 下的文档
   - 添加使用示例

#### 3. CI 流程

vx 使用 **crate 级别的变更检测** 优化 CI：

| 变更的 Crate | 影响的 Crate |
|-------------|-------------|
| `vx-core` | 所有依赖它的 crate |
| `vx-paths` | runtime, resolver, env, setup, migration, args, extension, cli |
| `vx-runtime` | resolver, extension, cli, 所有 providers |
| `vx-config` | project-analyzer, cli |
| Provider crates | 只有该 provider 和 cli |
| `vx-cli` | 只有 cli 本身 |

**CI Jobs**：
- `test-targeted` - 只测试受影响的 crate
- `test-full` - 完整测试（核心 crate 变更时）
- `code-quality` - 格式和 Clippy 检查
- `dogfood` - 集成测试

#### 4. 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/)：

```
feat: add support for new tool
fix: resolve version parsing issue
docs: update installation guide
test: add provider tests
refactor: simplify version resolution
```

---

## 🔍 现有 LLM 能力分析

### 当前已实现：Ollama Provider

**位置**：`crates/vx-providers/ollama/`

**功能**：
- ✅ 安装 Ollama 工具
- ✅ 版本管理
- ✅ 跨平台支持（Windows/macOS/Linux）
- ✅ 基本命令执行（`vx ollama serve`, `vx ollama pull`, etc.）

**实现方式**：
```rust
// 标准的 Provider 实现
pub struct OllamaProvider;

impl Provider for OllamaProvider {
    fn name(&self) -> &str { "ollama" }
    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(OllamaRuntime::new())]
    }
}
```

**使用方式**：
```bash
vx ollama --version
vx ollama serve
vx ollama pull llama3.2
vx ollama run llama3.2
```

**限制**：
- ❌ 只支持 Ollama 一个工具
- ❌ 没有统一的 LLM 抽象
- ❌ 不支持其他 LLM 工具（llama.cpp, LangChain 等）
- ❌ 没有 LLM 特定的功能（模型管理、上下文管理等）

---

## 💡 LLM Provider 扩展建议

### 方案 1：统一 LLM Provider（推荐）

**目标**：创建一个统一的 LLM Provider，支持多种 LLM 工具

#### 架构设计

```rust
// crates/vx-providers/llm/
pub struct LlmProvider;

impl Provider for LlmProvider {
    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            // 保留现有的 Ollama（向后兼容）
            Arc::new(OllamaRuntime::new()),
            
            // 新增其他 LLM 工具
            Arc::new(LlamaCppRuntime::new()),
            Arc::new(LangChainRuntime::new()),
            Arc::new(LlamaIndexRuntime::new()),
            Arc::new(TransformersRuntime::new()),
        ]
    }
}
```

#### 与现有能力的区别

| 特性 | 现有 Ollama Provider | 统一 LLM Provider |
|------|---------------------|------------------|
| **工具支持** | 只有 Ollama | Ollama + llama.cpp + LangChain + ... |
| **抽象层次** | 工具级别 | LLM 生态系统级别 |
| **模型管理** | 通过 Ollama CLI | 统一的模型管理接口 |
| **上下文管理** | ❌ 不支持 | ✅ 支持（AI 上下文导出） |
| **统一接口** | ❌ 每个工具独立 | ✅ 统一的 LLM 命令接口 |
| **扩展性** | 需要为每个工具创建 Provider | 在 LLM Provider 内添加 Runtime |

#### 实现示例

```rust
// crates/vx-providers/llm/src/runtime.rs

// 统一的 LLM Runtime trait
pub trait LlmRuntime: Runtime {
    /// 列出可用模型
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    
    /// 下载模型
    async fn download_model(&self, model: &str) -> Result<()>;
    
    /// 运行模型
    async fn run_model(&self, model: &str, prompt: &str) -> Result<String>;
    
    /// 获取模型信息
    async fn model_info(&self, model: &str) -> Result<ModelInfo>;
}

// Ollama 实现
pub struct OllamaRuntime;

impl LlmRuntime for OllamaRuntime {
    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // 调用 ollama list
    }
    
    async fn download_model(&self, model: &str) -> Result<()> {
        // 调用 ollama pull
    }
    
    async fn run_model(&self, model: &str, prompt: &str) -> Result<String> {
        // 调用 ollama run
    }
}

// llama.cpp 实现
pub struct LlamaCppRuntime;

impl LlmRuntime for LlamaCppRuntime {
    // 类似的实现，但调用 llama.cpp 的命令
}
```

#### 统一命令接口

```bash
# 统一的 LLM 命令（自动选择后端）
vx llm list                    # 列出所有可用模型（跨工具）
vx llm pull llama3.2           # 下载模型（自动选择最快源）
vx llm run llama3.2 "prompt"  # 运行模型（自动选择可用后端）

# 特定工具的命令（向后兼容）
vx ollama serve                # 仍然可用
vx llama-cpp --help           # 新工具的命令
```

---

### 方案 2：LLM 生态系统扩展（更全面）

**目标**：不仅支持 LLM 工具，还支持整个 AI 开发生态系统

#### 架构设计

```
crates/vx-providers/
├── ollama/          # 保留现有实现
├── llama-cpp/       # llama.cpp 支持
├── langchain/       # LangChain 框架
├── transformers/    # Hugging Face Transformers
└── ai-tools/        # 统一的 AI 工具管理
    ├── model-manager.rs    # 模型管理
    ├── context-manager.rs  # 上下文管理
    └── ai-config.rs        # AI 配置
```

#### 新增功能

**1. 模型管理**
```bash
# 统一的模型管理
vx ai models list                    # 列出所有模型（跨工具）
vx ai models download llama3.2      # 下载模型
vx ai models remove llama3.2        # 删除模型
vx ai models info llama3.2          # 查看模型信息
```

**2. 上下文管理**
```bash
# AI 上下文导出（为 AI 助手提供项目信息）
vx ai context export                 # 导出项目上下文
vx ai context update                 # 更新上下文
vx ai context format --cursor        # 格式化为 Cursor 格式
```

**3. AI 配置管理**
```toml
# vx.toml
[ai]
# LLM 工具配置
default_llm = "ollama"  # ollama | llama-cpp | langchain

[ai.ollama]
server_url = "http://localhost:11434"
default_model = "llama3.2"

[ai.context]
auto_export = true
export_format = "markdown"
include_files = [".cursorrules", "README.md"]
```

---

## 🎯 推荐实现方案

### 阶段 1：增强现有 Ollama Provider（短期）

**目标**：在现有基础上添加 LLM 特定功能

**实现**：
1. 添加模型管理命令
2. 添加上下文导出功能
3. 改进错误处理和用户体验

**优势**：
- ✅ 不破坏现有功能
- ✅ 快速实现
- ✅ 向后兼容

### 阶段 2：创建统一 LLM Provider（中期）

**目标**：支持多种 LLM 工具，统一接口

**实现**：
1. 创建 `vx-providers/llm` crate
2. 实现统一的 `LlmRuntime` trait
3. 集成 Ollama、llama.cpp 等
4. 提供统一的命令接口

**优势**：
- ✅ 统一的用户体验
- ✅ 易于扩展新工具
- ✅ 符合 vx 的设计理念

### 阶段 3：完整的 AI 生态系统（长期）

**目标**：支持整个 AI 开发生态系统

**实现**：
1. 添加更多 AI 工具（LangChain、Transformers 等）
2. 实现 AI 上下文管理
3. 添加 AI 配置管理
4. 集成到项目工作流

---

## 📝 具体实现建议

### 1. LLM Provider 结构

```
crates/vx-providers/llm/
├── Cargo.toml
├── provider.toml
└── src/
    ├── lib.rs
    ├── provider.rs          # 统一 LLM Provider
    ├── runtime.rs           # LlmRuntime trait
    ├── ollama.rs            # Ollama 实现（复用现有）
    ├── llama_cpp.rs         # llama.cpp 实现
    ├── langchain.rs         # LangChain 实现
    ├── model_manager.rs     # 模型管理
    └── context_manager.rs  # 上下文管理
```

### 2. 与现有 Ollama Provider 的关系

**选项 A：保留独立，添加统一接口**
- 保留 `vx-providers/ollama/` 不变
- 创建 `vx-providers/llm/` 作为统一接口
- `llm` provider 内部使用 `ollama` provider

**选项 B：迁移到统一 Provider**
- 将 `ollama` 的代码迁移到 `llm` provider
- 保持 `ollama` 命令的向后兼容（通过别名）

**推荐**：选项 A（更安全，向后兼容）

### 3. 新增功能示例

```rust
// crates/vx-providers/llm/src/model_manager.rs
pub struct ModelManager {
    backends: Vec<Box<dyn LlmBackend>>,
}

impl ModelManager {
    /// 列出所有后端的模型
    pub async fn list_all_models(&self) -> Result<Vec<ModelInfo>> {
        let mut all_models = Vec::new();
        for backend in &self.backends {
            let models = backend.list_models().await?;
            all_models.extend(models);
        }
        Ok(all_models)
    }
    
    /// 智能选择后端下载模型
    pub async fn download_model(&self, model: &str) -> Result<()> {
        // 1. 检查哪个后端支持该模型
        // 2. 选择最快的下载源
        // 3. 下载模型
    }
}
```

---

## 🔄 迁移路径

### 向后兼容策略

```rust
// 在 llm provider 中
impl Provider for LlmProvider {
    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            // 统一的 LLM runtime
            Arc::new(UnifiedLlmRuntime::new()),
            
            // 保留 ollama 作为别名（向后兼容）
            Arc::new(OllamaRuntime::new()),
        ]
    }
    
    fn supports(&self, name: &str) -> bool {
        matches!(name, "llm" | "ollama" | "llama-cpp" | "langchain")
    }
}
```

**使用方式**：
```bash
# 新方式（统一接口）
vx llm list
vx llm pull llama3.2

# 旧方式（仍然可用）
vx ollama list
vx ollama pull llama3.2
```

---

## 📊 对比总结

| 维度 | 现有 Ollama Provider | 统一 LLM Provider |
|------|---------------------|------------------|
| **工具数量** | 1 个（Ollama） | 多个（Ollama + 其他） |
| **抽象层次** | 工具级别 | 生态系统级别 |
| **命令接口** | `vx ollama <cmd>` | `vx llm <cmd>` + `vx ollama <cmd>` |
| **模型管理** | 通过 Ollama CLI | 统一的模型管理 |
| **上下文管理** | ❌ | ✅ |
| **扩展性** | 需要新 Provider | 在 Provider 内添加 Runtime |
| **向后兼容** | ✅ | ✅（保留 ollama 命令） |
| **实现复杂度** | 低 | 中 |
| **用户价值** | 基础功能 | 完整 AI 开发体验 |

---

## 🚀 开始贡献

### 第一步：讨论和规划

1. 在 GitHub Discussions 中提出想法
2. 创建 RFC（如果功能较大）
3. 获得维护者反馈

### 第二步：实现

1. Fork 仓库
2. 创建功能分支：`git checkout -b feat/llm-provider`
3. 实现功能
4. 添加测试
5. 更新文档

### 第三步：提交 PR

1. 确保所有测试通过
2. 更新相关文档
3. 创建 Pull Request
4. 描述实现细节和使用场景

---

## 💭 总结

**现有能力**：
- ✅ Ollama Provider 已实现
- ✅ 基本的 LLM 工具支持
- ✅ 跨平台支持

**LLM Provider 扩展的价值**：
- 🎯 **统一接口** - 一个命令管理所有 LLM 工具
- 🎯 **模型管理** - 跨工具的模型管理
- 🎯 **上下文管理** - 为 AI 助手提供项目上下文
- 🎯 **生态系统** - 支持完整的 AI 开发生态

**建议**：
1. 先增强现有 Ollama Provider（快速见效）
2. 然后创建统一 LLM Provider（长期价值）
3. 最后扩展到完整 AI 生态系统（终极目标）

这样的实现路径既保证了向后兼容，又提供了清晰的扩展路径！
