//! Tool-specific E2E Tests for vx CLI
//!
//! This module organizes E2E tests by tool/ecosystem:
//! - node/    - Node.js, npm, npx tests
//! - uv/      - UV, uvx, Python tests
//! - go/      - Go toolchain tests
//! - rust/    - Cargo, rustc tests
//! - bun/     - Bun runtime tests
//! - pnpm/    - pnpm package manager tests
//! - yarn/    - Yarn package manager tests
//! - cross_tool/      - Cross-tool scenarios
//! - project_context/ - vx.toml and project configuration tests
//!
//! # Running Tests
//!
//! ```bash
//! # Run all tool E2E tests
//! cargo test --package vx-cli --test tool_e2e_tests
//!
//! # Run specific tool tests
//! cargo test --package vx-cli --test tool_e2e_tests node
//! cargo test --package vx-cli --test tool_e2e_tests uv
//! cargo test --package vx-cli --test tool_e2e_tests go
//! cargo test --package vx-cli --test tool_e2e_tests rust
//! cargo test --package vx-cli --test tool_e2e_tests bun
//! cargo test --package vx-cli --test tool_e2e_tests pnpm
//! cargo test --package vx-cli --test tool_e2e_tests yarn
//!
//! # Run cross-tool tests
//! cargo test --package vx-cli --test tool_e2e_tests cross_tool
//!
//! # Run project context tests
//! cargo test --package vx-cli --test tool_e2e_tests project_context
//!
//! # Run tests that require network (ignored by default)
//! cargo test --package vx-cli --test tool_e2e_tests -- --ignored
//! ```

#[macro_use]
mod common;

mod bun;
mod cross_tool;
mod go;
mod node;
mod pnpm;
mod project_context;
mod rust;
mod uv;
mod yarn;
