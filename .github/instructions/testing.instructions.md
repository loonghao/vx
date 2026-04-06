---
applyTo: "crates/*/tests/**/*.rs,tests/**/*.rs"
---

# Testing Instructions for vx

## File Location

Tests MUST go in `crates/<name>/tests/` directories. NEVER create inline `#[cfg(test)]` modules.

```
crates/vx-resolver/tests/
├── resolver_tests.rs
├── executor_tests.rs
└── spec_tests.rs
```

## Framework

Use `rstest` for parameterized tests:

```rust
use rstest::rstest;

#[rstest]
#[case("node", Ecosystem::NodeJs)]
#[case("npm", Ecosystem::NodeJs)]
#[case("go", Ecosystem::Go)]
fn test_ecosystem_detection(#[case] name: &str, #[case] expected: Ecosystem) {
    let spec = RuntimeSpec::new(name);
    assert_eq!(spec.ecosystem, expected);
}
```

## Naming Convention

- Sync: `fn test_<function_name>_<scenario>()`
- Async: `async fn test_<function_name>_<scenario>()`

## Rules

- Mock network calls in unit tests — never use real HTTP
- Each test must be independent — no shared mutable state
- Run tests: `vx just test` or `vx cargo test -p <crate>`
