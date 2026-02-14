---
name: rfc-creator
description: |
  This skill helps create RFC (Request for Comments) documents for proposing new features,
  architectural changes, or significant enhancements to the project. It provides templates,
  structure guidelines, and best practices for writing effective technical proposals.
  Use this skill when planning major changes that need team review and discussion.
  
  IMPORTANT: Before writing any RFC, you MUST conduct online research to survey current
  industry best practices, mainstream implementations, and latest trends.
---

# RFC Creator

This skill guides the creation of RFC (Request for Comments) documents for technical proposals.

## When to Use

- Proposing a new feature or capability
- Planning architectural changes
- Introducing breaking changes
- Major refactoring proposals
- New configuration formats or APIs
- Integration with external systems

## CRITICAL: Pre-RFC Research Phase

**Before writing ANY RFC, you MUST complete the following research steps:**

### Step 0: Get Current Date

Use MCP time tool to get the current date for accurate research:

```
mcp_call_tool("time", "get_current_time", {"timezone": "Asia/Shanghai"})
```

This ensures your research queries include the correct year (e.g., 2025) for finding the latest information.

### Step 0.1: Online Research (MANDATORY)

For any RFC topic, you MUST conduct comprehensive online research:

#### 1. Search for Mainstream Implementations

Use `web_search` to find how major projects solve similar problems:

```
web_search({
  "searchTerm": "[topic] implementation rust [current year] best practices",
  "explanation": "Researching mainstream implementations for RFC topic"
})
```

**Example searches for different RFC topics:**

| RFC Topic | Search Queries |
|-----------|----------------|
| Console output | "rust CLI console output cargo uv ripgrep 2025" |
| Config format | "rust config file format toml yaml comparison 2025" |
| Plugin system | "rust plugin architecture dynamic loading 2025" |
| Package manager | "rust package manager design cargo uv 2025" |

#### 2. Fetch Source Code from Major Projects

Use `web_fetch` to examine actual implementations:

```
web_fetch({
  "url": "https://github.com/[project]/[path]/[file]",
  "fetchInfo": "Examining [project]'s implementation of [feature]"
})
```

**Key projects to reference by domain:**

| Domain | Projects to Research |
|--------|---------------------|
| CLI/Console | Cargo, uv, ripgrep, rustup |
| Package Management | Cargo, uv, pnpm |
| Configuration | Cargo, rustup, deno |
| Build Systems | Cargo, buck2, bazel |
| Version Management | rustup, nvm, pyenv |

#### 3. Document Research Findings

Create a "主流方案调研" (Mainstream Solution Survey) section in the RFC:

```markdown
## 主流方案调研

在设计本方案之前，我们调研了以下主流实现：

### 1. [Project A] (org/project)

**架构**: [Brief architecture description]

**核心设计**:
```rust
// Key code snippet
```

**关键特性**:
- Feature 1
- Feature 2

**依赖库**:
- `lib1` - Description
- `lib2` - Description

### 2. [Project B] (org/project)

[Similar structure...]

### 方案对比

| 特性 | Project A | Project B | Project C |
|------|-----------|-----------|-----------|
| Feature 1 | ✓ | ✗ | ✓ |
| Feature 2 | ✗ | ✓ | ✓ |

### 设计启示

基于以上调研，本 RFC 应采用：
1. [Design decision 1 with rationale]
2. [Design decision 2 with rationale]
3. [Design decision 3 with rationale]
```

### Step 0.2: Research Checklist

Before proceeding to write the RFC, verify:

- [ ] **Current date obtained** - Used MCP time tool to get accurate date
- [ ] **Web searches completed** - At least 3 relevant searches performed
- [ ] **Source code examined** - Reviewed at least 2-3 major project implementations
- [ ] **Comparison table created** - Documented feature comparison across projects
- [ ] **Design decisions documented** - Listed what to adopt from each project

## RFC Document Structure

### Standard Sections

1. **Header** - Metadata (status, author, date, target version)
2. **摘要/Summary** - Brief overview of the proposal
3. **主流方案调研/Industry Survey** - Research on mainstream implementations (NEW - REQUIRED)
4. **动机/Motivation** - Why this change is needed
5. **设计方案/Design** - Detailed technical design
6. **向后兼容性/Backward Compatibility** - Migration and compatibility considerations
7. **实现计划/Implementation Plan** - Phased implementation roadmap
8. **替代方案/Alternatives** - Alternative approaches considered
9. **参考资料/References** - Related documents and resources
10. **更新记录/Changelog** - Document revision history

## Step 1: Create RFC Directory

```
docs/rfcs/
├── NNNN-short-title.md           # Main RFC document
└── NNNN-implementation-tracker.md # Implementation progress tracker (optional)
```

RFC numbers are assigned sequentially: `0001`, `0002`, etc.

## Step 2: RFC Header Template

```markdown
# RFC NNNN: Title

> **状态**: Draft | Review | Accepted | Implemented | Rejected
> **作者**: author name/team
> **创建日期**: YYYY-MM-DD
> **目标版本**: vX.Y.Z
```

### Status Definitions

| Status | Description |
|--------|-------------|
| **Draft** | Initial proposal, open for major changes |
| **Review** | Ready for team review and feedback |
| **Accepted** | Approved for implementation |
| **Implemented** | Fully implemented and released |
| **Rejected** | Not accepted (with documented reasons) |

## Step 3: Write the RFC Content

### 3.1 Summary Section

```markdown
## 摘要

[One paragraph describing what this RFC proposes and its main benefits]
```

### 3.2 Industry Survey Section (NEW - REQUIRED)

```markdown
## 主流方案调研

在设计本方案之前，我们调研了以下主流实现：

### 1. [Project Name] (org/repo)

**架构**: [Architecture description]

**核心设计**:
```rust
// Key implementation snippet
pub struct Example {
    field: Type,
}
```

**关键特性**:
- Feature 1 - Description
- Feature 2 - Description

**依赖库**:
- `library1` - Purpose
- `library2` - Purpose

### 2. [Another Project] (org/repo)

[Similar structure...]

### 方案对比

| 特性 | Project A | Project B | Project C |
|------|-----------|-----------|-----------|
| Feature X | ✓ impl | ✗ | ✓ partial |
| Feature Y | ✗ | ✓ | ✓ |
| Library | lib-a | lib-b | lib-c |

### 设计启示

基于以上调研，本 RFC 应采用：

1. **[Decision 1]** - 采用 [Project A] 的 [approach]，因为 [rationale]
2. **[Decision 2]** - 使用 [library]，这是 [Project B] 使用的方案
3. **[Decision 3]** - 参考 [Project C] 的 [feature] 设计
```

### 3.3 Motivation Section

```markdown
## 动机

### 当前状态分析
[Describe current limitations or problems]

### 需求分析
[List specific requirements this proposal addresses]

1. **Requirement 1** - Description
2. **Requirement 2** - Description
```

### 3.4 Design Section

```markdown
## 设计方案

### 完整配置/API 预览

```toml/yaml/json
# Complete example of the proposed format
```

### 详细说明

#### Section 1: Feature Name
[Detailed explanation with examples]

#### Section 2: Feature Name
[Detailed explanation with examples]
```

### 3.5 Backward Compatibility Section

```markdown
## 向后兼容性

### 兼容策略

1. **Version Detection** - How to detect old vs new format
2. **Gradual Enhancement** - All new fields are optional
3. **Default Values** - Sensible defaults for new fields
4. **Warning Handling** - Warn on unknown fields, don't error

### 迁移路径

```bash
# Check compatibility
command check

# Auto-migrate
command migrate --to v2

# Validate
command validate
```
```

### 3.6 Implementation Plan Section

```markdown
## 实现计划

### Phase 1: Core Features (vX.Y.0)

- [ ] Feature A
- [ ] Feature B
- [ ] Migration tooling

### Phase 2: Extended Features (vX.Y+1.0)

- [ ] Feature C
- [ ] Feature D

### Phase 3: Advanced Features (vX.Y+2.0)

- [ ] Feature E
- [ ] Feature F
```

### 3.7 References Section

```markdown
## 参考资料

### 主流项目源码
- [Project A Source](https://github.com/org/project/blob/main/src/module.rs) - 本 RFC 的主要参考
- [Project B Source](https://github.com/org/project/tree/main/crates) - 参考其模块设计

### 依赖库
- [library1](https://crates.io/crates/library1) - 用途说明
- [library2](https://crates.io/crates/library2) - 用途说明

### 相关文档
- [Related Tool Documentation](url)
- [Industry Standard](url)
```

### 3.8 Changelog Section

```markdown
## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| YYYY-MM-DD | Draft | 初始草案 |
| YYYY-MM-DD | Review | 根据反馈更新 |
```

## Step 4: Create Implementation Tracker (Optional)

For complex RFCs, create a separate tracker document:

```markdown
# RFC NNNN: Implementation Tracker

## 总体进度

| Phase | 状态 | 完成度 | 目标版本 |
|-------|------|--------|----------|
| Phase 1 | 进行中 | 60% | vX.Y.0 |
| Phase 2 | 待开始 | 0% | vX.Y+1.0 |

## 详细进度

### Phase 1: Core Features

#### Feature A
- [x] Design
- [x] Implementation
- [ ] Tests
- [ ] Documentation

#### Feature B
- [ ] Design
- [ ] Implementation
- [ ] Tests
- [ ] Documentation

## 测试计划

### 单元测试
- [ ] Test case 1
- [ ] Test case 2

### 集成测试
- [ ] Integration test 1
- [ ] Integration test 2

### E2E 测试
- [ ] E2E test 1
- [ ] E2E test 2

## 文档更新

- [ ] Config reference
- [ ] User guide
- [ ] Migration guide
- [ ] Best practices

## 更新日志

| 日期 | 变更 |
|------|------|
| YYYY-MM-DD | 创建跟踪文档 |
```

## Best Practices

### Writing Effective RFCs

1. **Be Specific** - Include concrete examples and code snippets
2. **Consider Edge Cases** - Address error handling and unusual scenarios
3. **Think About Migration** - Always plan for existing users
4. **Keep It Focused** - One RFC per major feature/change
5. **Iterate** - RFCs can be updated based on feedback

### Code Examples

- Use realistic, working examples
- Show both simple and advanced usage
- Include error cases where relevant

### Tables and Diagrams

- Use tables for comparisons and status tracking
- Include ASCII diagrams for architecture when helpful
- Keep formatting consistent

### Review Process

1. Share RFC with team for initial feedback
2. Address comments and update document
3. Move to "Review" status when ready
4. Get formal approval before implementation
5. Update status as implementation progresses

## RFC Naming Convention

```
NNNN-short-descriptive-title.md
```

Examples:
- `0001-config-v2-enhancement.md`
- `0002-plugin-architecture.md`
- `0003-remote-development-support.md`

## Reference Templates

See `references/templates.md` for complete RFC templates.
