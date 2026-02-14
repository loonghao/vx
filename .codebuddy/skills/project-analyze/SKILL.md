---
name: project-analyze
description: |
  This skill analyzes real-world open source projects to test and validate the vx-project-analyzer crate.
  It should be used when testing the project analyzer against external projects, discovering issues,
  and iterating on fixes. The skill handles the full workflow: searching for projects, cloning them
  to a temporary location, running analysis, fixing any issues found, and cleaning up afterward.
---

# Project Analyze Skill

This skill provides a systematic workflow for testing vx-project-analyzer against real-world projects.

## When to Use

- Testing the project analyzer against specific open source projects
- Validating analyzer changes work correctly across different project types
- Discovering edge cases and missing features in the analyzer
- Iterating on fixes until analysis works correctly

## Workflow

### Step 1: Project Discovery

To find the project repository:

1. If the project name is ambiguous (e.g., "codex"), search GitHub to find the correct repository
2. Common project mappings are documented in `references/known-projects.md`
3. Verify the repository URL before proceeding

### Step 2: Clone Project

To clone the project to a temporary location:

```powershell
# Windows
cd c:/github && git clone --depth 1 <repo-url> <project-name>-test

# Unix
cd /tmp && git clone --depth 1 <repo-url> <project-name>-test
```

Use `--depth 1` for shallow clone to save time and space.

### Step 3: Run Analysis

To run the project analyzer:

```bash
cargo run --manifest-path c:/github/vx/Cargo.toml -p vx-project-analyzer --example analyze_project -- <project-path>
```

### Step 4: Evaluate Results

To evaluate the analysis results, check:

1. **Ecosystems detected** - Are all languages in the project detected?
2. **Dependencies** - Are dependencies correctly parsed? Count should be reasonable.
3. **Scripts** - Are common scripts detected? No false positives?
4. **Required tools** - Are the right tools identified? No misidentified npm scripts?

Common issues to look for:
- Monorepo projects with language files in subdirectories
- Missing language support (e.g., Go, Java)
- False positive tool detection from internal script references
- Workspace/monorepo dependencies not being counted

### Step 5: Fix Issues

If issues are found:

1. Identify the root cause in vx-project-analyzer code
2. Make targeted fixes to the relevant modules
3. Re-run analysis to verify the fix
4. Repeat until analysis is correct

### Step 6: Cleanup

After testing is complete, remove the cloned project:

```powershell
# Windows
Remove-Item -Recurse -Force c:/github/<project-name>-test

# Unix
rm -rf /tmp/<project-name>-test
```

## Quick Reference

### Supported Languages

| Language | Marker File | Analyzer |
|----------|-------------|----------|
| Python | `pyproject.toml`, `setup.py` | PythonAnalyzer |
| Node.js | `package.json` | NodeJsAnalyzer |
| Rust | `Cargo.toml` | RustAnalyzer |
| Go | `go.mod` | GoAnalyzer |

### Common Test Projects

See `references/known-projects.md` for a curated list of test projects.

### Analysis Output Format

```
=== Analyzing: <path> ===

📦 Detected ecosystems: [<ecosystems>]

📋 Dependencies: <count> found
    - <name> (<version>) [<ecosystem>]
    ...

📜 Scripts: <count> found
    - <name>: `<command>`
    ...

🔧 Required tools:
    - <tool>: <description>
    ...
```

## Troubleshooting

### No ecosystems detected
- Check if marker files exist in root or immediate subdirectories
- Verify `max_depth` config allows subdirectory scanning

### Dependencies count is 0
- For workspace projects, dependencies may be in member crates
- Check if the dependency parser handles the file format correctly

### Too many required tools (false positives)
- Check if internal script references are being filtered
- Verify `known_scripts` context is passed to parser

### Missing language support
- Check if a language analyzer exists in `languages/` module
- May need to implement a new analyzer
