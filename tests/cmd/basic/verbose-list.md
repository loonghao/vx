# Test: vx --verbose list

Verify that `vx --verbose list` works with verbose flag.

```console
$ vx --verbose list
[..]Available Tools[..]
[..]
  ❌ npx - Node.js package runner
  ❌ cargo - Rust package manager and build tool
  ✅ bun - Incredibly fast JavaScript runtime, bundler, test runner, and package manager
  ❌ golang - Go programming language
  ✅ node - Node.js JavaScript runtime
  ❌ uvx - Python application runner
  ❌ nodejs - Node.js JavaScript runtime
  ✅ uv - An extremely fast Python package installer and resolver
  ❌ npm - Node.js package manager

```
