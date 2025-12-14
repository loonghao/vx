# Test: vx list --status

Verify that `vx list --status` shows tool status.

```console
$ vx list --status
[..]Available Tools[..]
  ✅ node - Node.js JavaScript runtime
     Versions: 22.12.0, 24.2.0
  ❌ npx - Node.js package runner
  ✅ uv - An extremely fast Python package installer and resolver
[..]
  ❌ go - Go programming language
  ❌ npm - Node.js package manager
  ❌ nodejs - Node.js JavaScript runtime
  ❌ golang - Go programming language
  ❌ uvx - Python application runner
  ✅ bun - Incredibly fast JavaScript runtime, bundler, test runner, and package manager
     Versions: 1.1.42, 1.2.9
  ❌ cargo - Rust package manager and build tool
ℹ
📊 Summary: 3/10 tools installed

```
