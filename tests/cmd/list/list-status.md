# Test: vx list --status

Verify that `vx list --status` shows tool status.

```console
$ vx list --status
[..]Available Tools[..]
[..]
  ✅ bun - Incredibly fast JavaScript runtime, bundler, test runner, and package manager
     Versions: 1.1.42, 1.2.9
  ❌ cargo - Rust package manager and build tool
  ❌ npm - Node.js package manager
  ❌ npx - Node.js package runner
  ✅ node - Node.js JavaScript runtime
     Versions: 22.12.0, 24.2.0
  ❌ go - Go programming language
  ❌ nodejs - Node.js JavaScript runtime
  ✅ uv - An extremely fast Python package installer and resolver
     Versions: 0.6.12, 0.7.13
  ❌ uvx - Python application runner
ℹ
📊 Summary: 3/10 tools installed

```
