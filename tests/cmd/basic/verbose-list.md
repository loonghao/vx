# Test: vx --verbose list

Verify that `vx --verbose list` works with verbose flag.

```console
$ vx --verbose list
[..]Available Tools[..]
  ✅ uv - An extremely fast Python package installer and resolver
  ❌ nodejs - Node.js JavaScript runtime
[..]
  ❌ go - Go programming language
  ❌ golang - Go programming language
  ✅ bun - Incredibly fast JavaScript runtime, bundler, test runner, and package manager
  ✅ node - Node.js JavaScript runtime
  ❌ npx - Node.js package runner
  ❌ cargo - Rust package manager and build tool
  ❌ npm - Node.js package manager

```
