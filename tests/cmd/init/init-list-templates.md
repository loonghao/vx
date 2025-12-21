# Test: vx init --list-templates

Verify that `vx init --list-templates` shows available templates.

```console
$ vx init --list-templates
[..]template[..]

  node        - Node.js project with npm
  node-pnpm   - Node.js project with pnpm
  node-yarn   - Node.js project with yarn
  node-bun    - Node.js project with bun
  python      - Python project with uv
  python-pip  - Python project with pip
  rust        - Rust project with cargo
  go          - Go project
  fullstack   - Full-stack project (Node.js + Python)
  minimal     - Minimal configuration

Usage: vx init --template <template>

```
