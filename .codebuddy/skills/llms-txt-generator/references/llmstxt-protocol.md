# llmstxt.org Protocol Specification

> Official protocol specification from https://llmstxt.org/

## Background

Websites are typically designed for humans, containing navigation elements, advertisements, JavaScript, CSS, and other content that is difficult for LLMs to process. The `/llms.txt` proposal suggests placing a Markdown file at the website's root directory to provide LLM-friendly content.

## Core Recommendations

1. **Add `/llms.txt` file**: Provide an `llms.txt` file at the website root (or optional subpath) for LLM-friendly content overview
2. **Provide Markdown versions**: For useful pages, provide Markdown versions with `.md` suffix (e.g., `page.html.md`)

## File Format Specification

The `llms.txt` file uses **Markdown** format with a specific structure that allows parsing by both regex and standard programming techniques.

### Required Structure Order

The file MUST contain these sections in order:

1. **H1 Title** (REQUIRED)
   - Project or website name
   - This is the ONLY required section

2. **Blockquote** (Recommended)
   - Short project summary
   - Contains key information needed to understand the rest of the file

3. **Detailed Information** (Optional)
   - Zero or more Markdown sections (paragraphs, lists, etc.)
   - **Restriction**: Cannot contain any headers (H2-H6) in this section

4. **File Lists** (Optional, separated by H2 headers)
   - Zero or more Markdown sections separated by H2 headers
   - Each "file list" is a Markdown list
   - List items MUST contain a Markdown hyperlink `[name](url)`
   - Links MAY be followed by a colon `:` and annotation about the file

### Special Section: `## Optional`

If a file list section is named `## Optional`, the URLs within are considered secondary information. These links can be skipped when context window size is limited.

## Format Example

```markdown
# Project/Website Title

> Short summary description of the project with key background information.

More detailed project description (optional).

## First Section Name (e.g., Core Documentation)

- [Link Title](https://example.com/doc1.md): Short description of this document.
- [Another Link](https://example.com/doc2.md)

## Optional

- [Secondary Info Link](https://example.com/extra.md): Can be skipped when context is limited.
```

## Comparison with Existing Standards

### vs robots.txt

- `robots.txt`: Tells crawlers what content can be scraped (mainly for training/crawling)
- `llms.txt`: Provides context when users request information (mainly for **inference**)

### vs sitemap.xml

- `sitemap.xml`: Lists all human-readable page indexes, usually doesn't include Markdown versions
- `sitemap.xml`: Total content is usually too large and contains unnecessary information
- `llms.txt`: Curated overview with external links and Markdown versions

## Best Practices

### Link Format

Always use full URLs, not relative paths:

```markdown
# Good
- [Guide](https://github.com/owner/repo/blob/main/docs/guide.md): Description

# Bad
- [Guide](./docs/guide.md): Description
```

### Description Format

Include colon and space before description:

```markdown
# Good
- [API Reference](url): Complete API documentation

# Bad
- [API Reference](url) - Complete API documentation
- [API Reference](url) Complete API documentation
```

### H1 Title

Use only the project name, no additional formatting:

```markdown
# Good
# AuroraView

# Bad
# 🚀 AuroraView - The Best WebView Framework
```

### Blockquote

Keep it concise (1-2 sentences):

```markdown
# Good
> A lightweight WebView framework for DCC software, built with Rust and Python.

# Bad
> A lightweight WebView framework for DCC software, built with Rust and Python.
> It supports Maya, Houdini, Blender, and more. Features include bidirectional
> communication, custom protocols, and native performance.
```

## Tools and Ecosystem

### CLI Tools

- `llms_txt2ctx`: Parse `llms.txt` and generate LLM context files

### Generator Plugins

- VitePress plugin
- Docusaurus plugin
- Static site generators

### IDE Extensions

- VS Code PagePilot extension: Auto-load external documentation context
