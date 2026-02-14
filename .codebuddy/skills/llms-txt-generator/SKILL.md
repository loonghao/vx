---
name: llms-txt-generator
description: |
  Generate llms.txt and llms-full.txt files following the llmstxt.org protocol specification.
  This skill should be used when users want to create, update, or generate llms.txt files for
  their projects to make them more accessible to Large Language Models (LLMs). Triggers include:
  "create llms.txt", "generate llms-full.txt", "update llms.txt", "add llmstxt.org support",
  "make project LLM-friendly", "llms.txt protocol".
---

# LLMs.txt Generator Skill

Generate `llms.txt` and `llms-full.txt` files following the [llmstxt.org](https://llmstxt.org/) protocol specification to make projects more accessible to Large Language Models.

## What is llms.txt?

The `/llms.txt` protocol is a standard for providing structured information about a website/project to LLMs. It helps LLMs quickly understand the project structure and access key documentation without parsing complex HTML pages.

### Protocol Benefits

- **Better LLM Interaction**: LLMs can quickly understand project capabilities
- **Accurate Responses**: Provides authoritative documentation links
- **Developer Experience**: Improves AI-assisted development
- **Discoverability**: Makes projects accessible to AI coding assistants

## File Structure Requirements

### llms.txt Format (Mandatory Structure)

The file MUST follow this exact order:

```markdown
# Project Title

> Brief project summary in a blockquote (1-2 sentences)

Additional project details (optional paragraphs/lists, no H2-H6 headers here)

## Section Name

- [Link Title](url): Description of the link
- [Another Link](url)

## Optional

- [Secondary Info](url): Links that can be skipped when context is limited
```

### Two File Types

| File | Purpose | Size |
|------|---------|------|
| `llms.txt` | Concise overview with essential links | ~100-200 lines |
| `llms-full.txt` | Comprehensive documentation with code examples | ~500-1000 lines |

## Generation Workflow

### Step 1: Gather Project Information

To generate accurate llms.txt files, collect the following information:

1. **Project README**: Read `README.md` for project overview, features, and quick start
2. **Documentation Structure**: List files in `docs/` directory to identify available guides
3. **API Reference**: Find API documentation files
4. **Examples**: List examples in `examples/` directory
5. **Package Info**: Read `package.json`, `pyproject.toml`, or `Cargo.toml` for metadata

### Step 2: Generate llms.txt (Concise Version)

Create `llms.txt` with:

1. **H1 Title**: Project name
2. **Blockquote**: One-sentence project summary
3. **Key Info**: 
   - Package installation command
   - Supported platforms/versions
   - License
   - Repository URL
4. **Key Features**: Bullet list of main features (5-10 items)
5. **Documentation Section** (`## Documentation`):
   - Getting started guide
   - Installation
   - Core concepts
   - Architecture overview
6. **API Reference Section** (`## API Reference`):
   - Main API documentation links
7. **Integration/Platform Sections** (if applicable):
   - Platform-specific guides
8. **Optional Section** (`## Optional`):
   - Advanced guides
   - Examples
   - Contributing guide

### Step 3: Generate llms-full.txt (Comprehensive Version)

Create `llms-full.txt` with everything from `llms.txt` plus:

1. **Extended Features**: Complete feature list with descriptions
2. **Technical Stack**: Detailed technical information
3. **Quick Start Code**: Actual code examples for common use cases
4. **Complete API Reference**: All API classes and methods with signatures
5. **All Documentation Links**: Every guide and reference document
6. **Code Examples**: Inline code snippets demonstrating key functionality
7. **RFCs/Architecture Docs**: Technical design documents
8. **Full Examples List**: All example files with descriptions

## Format Rules

### Link Format

```markdown
- [Title](https://github.com/owner/repo/blob/main/path/to/file.md): Brief description
```

### Code Blocks in llms-full.txt

Include actual code examples:

```markdown
### Quick Start

\`\`\`python
from mypackage import MyClass

instance = MyClass()
instance.do_something()
\`\`\`
```

### Section Guidelines

| Section | Required | Content |
|---------|----------|---------|
| H1 Title | Yes | Project name only |
| Blockquote | Yes | 1-2 sentence summary |
| Key Features | Yes | 5-10 main features |
| Documentation | Yes | Core documentation links |
| API Reference | Yes | API documentation links |
| Optional | No | Secondary/advanced content |

## Example Templates

### llms.txt Template

```markdown
# ProjectName

> Brief description of the project in one or two sentences.

ProjectName is a [type of project] that provides [main capability]. Built with [technology], it offers [key benefit].

- **Package**: `pip install projectname`
- **Platforms**: Windows, macOS, Linux
- **License**: MIT
- **Repository**: https://github.com/owner/projectname

## Key Features

- Feature 1 description
- Feature 2 description
- Feature 3 description

## Documentation

- [Getting Started](https://github.com/owner/repo/blob/main/docs/getting-started.md): Quick start guide
- [Installation](https://github.com/owner/repo/blob/main/docs/installation.md): Installation instructions
- [Core Concepts](https://github.com/owner/repo/blob/main/docs/concepts.md): Core concepts

## API Reference

- [Main API](https://github.com/owner/repo/blob/main/docs/api/main.md): Main API reference

## Optional

- [Advanced Usage](https://github.com/owner/repo/blob/main/docs/advanced.md): Advanced features
- [Examples](https://github.com/owner/repo/tree/main/examples): Code examples
```

### llms-full.txt Template

```markdown
# ProjectName

> Brief description of the project in one or two sentences.

ProjectName is a [type of project] that provides [main capability]. Built with [technology], it offers [key benefit].

- **Package**: `pip install projectname`
- **Platforms**: Windows, macOS, Linux
- **License**: MIT
- **Repository**: https://github.com/owner/projectname
- **PyPI/NPM**: https://pypi.org/project/projectname/

## Key Features

- **Feature 1**: Detailed description of feature 1
- **Feature 2**: Detailed description of feature 2
- **Feature 3**: Detailed description of feature 3

## Technical Stack

- Core: Technology 1, Technology 2
- Runtime: Platform requirements
- Packaging: Build system details

## Quick Start

### Installation

\`\`\`bash
pip install projectname
\`\`\`

### Basic Usage

\`\`\`python
from projectname import MainClass

instance = MainClass(param="value")
result = instance.method()
print(result)
\`\`\`

## Core API

### MainClass

\`\`\`python
MainClass(
    param1: str = "default",
    param2: int = 100,
)
\`\`\`

### Methods

\`\`\`python
instance.method1()  # Description
instance.method2(arg)  # Description
\`\`\`

## Documentation

- [Getting Started](url): Quick start guide
- [Installation](url): Installation instructions
- [Core Concepts](url): Core concepts
- [Architecture](url): System architecture

## API Reference

- [Main API](url): Main API reference
- [Secondary API](url): Secondary API reference

## Advanced Guides

- [Advanced Topic 1](url): Description
- [Advanced Topic 2](url): Description

## Examples

- [Example 1](url): Description of example 1
- [Example 2](url): Description of example 2
```

## Update Workflow

To update existing llms.txt files:

1. **Read Current Files**: Load existing `llms.txt` and `llms-full.txt`
2. **Identify Changes**: Compare with current documentation structure
3. **Update Sections**: Add new documentation links, update descriptions
4. **Maintain Format**: Preserve the protocol structure
5. **Validate**: Ensure all links are valid and descriptions are accurate

## Validation Checklist

Before finalizing, verify:

- [ ] H1 title is present and matches project name
- [ ] Blockquote summary is present
- [ ] Key information (package, platforms, license, repo) is included
- [ ] All links use full GitHub URLs (not relative paths)
- [ ] Links include `: description` format
- [ ] `## Optional` section contains secondary content
- [ ] No H2-H6 headers before the first `## Section`
- [ ] Code examples in llms-full.txt are syntactically correct
- [ ] All referenced files exist in the repository

## Common Patterns

### For Python Projects

```markdown
- **Package**: `pip install packagename`
- **Python**: 3.7+
```

### For Rust Projects

```markdown
- **Crate**: `cargo add cratename`
- **Rust**: 1.70+
```

### For Node.js Projects

```markdown
- **Package**: `npm install packagename`
- **Node.js**: 18+
```

### For Multi-Language Projects

```markdown
- **Python Package**: `pip install packagename`
- **Rust Core**: Built with Rust 1.75+
- **Python**: 3.7+
```
