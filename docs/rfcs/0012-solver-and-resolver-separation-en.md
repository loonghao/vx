# RFC 0012: Solver and Resolver Separation

> **Status**: Draft
> **Author**: vx team
> **Created**: 2026-01-15
> **Target Version**: v0.9.0

## Summary

This RFC proposes a clear separation of responsibilities between `vx-resolver` and a new `vx-solver` crate, avoiding functional overlap and preparing the architecture for future SAT-based dependency resolution (e.g., pubgrub).

## Current State Analysis

### Existing vx-resolver Responsibilities

`vx-resolver` currently handles multiple responsibilities:

```
vx-resolver/
├── config.rs           # Resolver configuration
├── executor.rs         # Command execution (core: forward commands to runtime)
├── resolver.rs         # Runtime status detection (installed/system/missing)
├── runtime_map.rs      # Runtime mapping (name → spec)
├── runtime_spec.rs     # Runtime spec definitions
├── runtime_index.rs    # Runtime index cache
├── resolution_cache.rs # Resolution result cache
└── version/
    ├── constraint.rs   # Version constraint expressions
    ├── lockfile.rs     # vx.lock read/write
    ├── request.rs      # Version request parsing
    ├── resolved.rs     # Resolved version
    ├── solver.rs       # Single-tool version resolution
    └── strategy.rs     # Ecosystem-specific version semantics
```

### Problems Identified

1. **Unclear Responsibilities**
   - `Resolver` (resolver.rs) detects runtime status
   - `VersionSolver` (solver.rs) resolves version constraints
   - Similar names, unclear boundaries

2. **Incomplete Dependency Resolution**
   - Current `VersionSolver` only handles single-tool version resolution
   - Dependency chain resolution was ad-hoc in `lock.rs` (recently fixed hardcoded issue)
   - No true dependency graph solving

3. **Lack of Conflict Explanation**
   - When version constraints conflict, no clear explanation is provided
   - Mainstream tools (uv, cargo) have human-readable conflict reports

## Design Proposal

### Responsibility Separation Principle

```
┌──────────────────────────────────────────────────────────────────┐
│                          vx-resolver                              │
│  Responsibility: Runtime Execution & Environment Resolution       │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────────┐     ┌─────────────────┐                     │
│  │    Executor     │     │    Resolver     │                     │
│  │  Command        │     │  Runtime Status │                     │
│  │  Execution      │     │  Detection      │                     │
│  └────────┬────────┘     └────────┬────────┘                     │
│           │                       │                               │
│           ▼                       ▼                               │
│  • Command routing           • Detect vx-managed vs system        │
│  • PATH management           • Dependency availability check      │
│  • Process forwarding        • Executable location                │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘

                              │ calls
                              ▼

┌──────────────────────────────────────────────────────────────────┐
│                          vx-solver (new)                          │
│  Responsibility: Version & Dependency Solving                     │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────────┐     ┌─────────────────┐                     │
│  │  VersionSolver  │     │ DependencySolver│                     │
│  │  Version        │     │  Dependency     │                     │
│  │  Constraint     │     │  Graph          │                     │
│  │  Solving        │     │  Solving        │                     │
│  └────────┬────────┘     └────────┬────────┘                     │
│           │                       │                               │
│           ▼                       ▼                               │
│  • Version constraint parsing • Transitive dependency resolution  │
│  • Partial version matching   • Topological sorting               │
│  • Ecosystem version semantics• Conflict detection & explanation  │
│                                                                   │
│  ┌─────────────────┐     ┌─────────────────┐                     │
│  │   LockFile      │     │ ConflictExplainer│                    │
│  │  Lock File      │     │  Conflict       │                     │
│  │  Management     │     │  Explanation    │                     │
│  └─────────────────┘     └─────────────────┘                     │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

### Module Migration Plan

| Current Location | Move To | Notes |
|-----------------|---------|-------|
| `vx-resolver/version/solver.rs` | `vx-solver/version_solver.rs` | Version constraint solving |
| `vx-resolver/version/lockfile.rs` | `vx-solver/lockfile.rs` | Lock file management |
| `vx-resolver/version/constraint.rs` | `vx-solver/constraint.rs` | Constraint expressions |
| `vx-resolver/version/strategy.rs` | `vx-solver/strategy.rs` | Version semantic strategies |
| `vx-resolver/version/request.rs` | `vx-solver/request.rs` | Version requests |
| `vx-resolver/version/resolved.rs` | `vx-solver/resolved.rs` | Resolution results |
| `vx-resolver/executor.rs` | **Keep** | Command execution |
| `vx-resolver/resolver.rs` | **Keep** | Runtime status detection |
| `vx-resolver/runtime_*.rs` | **Keep** | Runtime mapping |

### vx-solver Detailed Design

#### Directory Structure

```
crates/vx-solver/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API
│   ├── version/
│   │   ├── mod.rs
│   │   ├── constraint.rs   # Version constraint expressions
│   │   ├── request.rs      # Version request parsing
│   │   ├── resolved.rs     # Resolution results
│   │   ├── solver.rs       # Version solver
│   │   └── strategy.rs     # Ecosystem strategies
│   ├── dependency/
│   │   ├── mod.rs
│   │   ├── graph.rs        # Dependency graph building
│   │   ├── solver.rs       # Dependency solver
│   │   └── topo_sort.rs    # Topological sorting
│   ├── lockfile/
│   │   ├── mod.rs
│   │   ├── format.rs       # Lock file format
│   │   ├── reader.rs       # Reading
│   │   ├── writer.rs       # Writing
│   │   └── diff.rs         # Diff comparison
│   ├── conflict/
│   │   ├── mod.rs
│   │   ├── detector.rs     # Conflict detection
│   │   └── explainer.rs    # Human-readable explanation
│   └── pubgrub/            # Optional: SAT solver adapter
│       ├── mod.rs
│       └── adapter.rs      # pubgrub adapter
└── tests/
    ├── version_tests.rs
    ├── dependency_tests.rs
    └── lockfile_tests.rs
```

#### Core Type Definitions

```rust
// crates/vx-solver/src/lib.rs

//! VX Solver - Version and Dependency Resolution
//!
//! This crate provides:
//! - Version constraint parsing and matching
//! - Dependency graph resolution
//! - Lock file management
//! - Conflict detection and explanation

pub mod conflict;
pub mod dependency;
pub mod lockfile;
pub mod version;

// Optional pubgrub integration
#[cfg(feature = "pubgrub")]
pub mod pubgrub;

// Unified solve request
pub struct SolveRequest {
    /// Tools to resolve
    pub tools: Vec<ToolRequest>,
    /// Existing lock file (for incremental resolution)
    pub existing_lock: Option<LockFile>,
    /// Whether to allow updates to locked versions
    pub allow_updates: bool,
}

pub struct ToolRequest {
    /// Tool name
    pub name: String,
    /// Version constraint
    pub constraint: VersionConstraint,
    /// Source (vx.toml line number for error reporting)
    pub source: Option<SourceLocation>,
}

// Unified solve result
pub struct SolveResult {
    /// Solve status
    pub status: SolveStatus,
    /// Resolved versions
    pub resolved: HashMap<String, ResolvedTool>,
    /// Dependency graph
    pub dependencies: DependencyGraph,
    /// Warnings
    pub warnings: Vec<SolveWarning>,
}

pub enum SolveStatus {
    /// Solving succeeded
    Success,
    /// Solving failed (with conflict explanation)
    Failed(ConflictExplanation),
    /// Partial success (some tools couldn't be resolved)
    Partial { failed: Vec<String> },
}
```

#### Dependency Graph Solving

```rust
// crates/vx-solver/src/dependency/graph.rs

use std::collections::{HashMap, HashSet};

/// Dependency graph
pub struct DependencyGraph {
    /// Nodes: tool name -> tool info
    nodes: HashMap<String, DependencyNode>,
    /// Edges: tool name -> dependency list
    edges: HashMap<String, Vec<DependencyEdge>>,
}

pub struct DependencyNode {
    pub name: String,
    pub resolved_version: Option<String>,
    pub constraint: VersionConstraint,
    pub is_direct: bool, // Declared directly in vx.toml
}

pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub constraint: Option<VersionConstraint>,
}

impl DependencyGraph {
    /// Build dependency graph from registry
    pub fn build(
        tools: &[ToolRequest],
        registry: &ProviderRegistry,
    ) -> Result<Self, GraphBuildError> {
        let mut graph = Self::new();
        let mut visited = HashSet::new();
        
        for tool in tools {
            graph.add_tool_recursive(&tool.name, true, registry, &mut visited)?;
        }
        
        Ok(graph)
    }
    
    /// Topological sort, returns installation order
    pub fn topological_sort(&self) -> Result<Vec<String>, CycleError> {
        // Kahn's algorithm implementation
        // ...
    }
}
```

#### Conflict Explainer

```rust
// crates/vx-solver/src/conflict/explainer.rs

/// Conflict explanation
pub struct ConflictExplanation {
    /// Conflict type
    pub kind: ConflictKind,
    /// Human-readable message
    pub message: String,
    /// Affected tools
    pub affected_tools: Vec<String>,
    /// Suggested solutions
    pub suggestions: Vec<String>,
    /// Conflict chain (for debugging)
    pub chain: Vec<ConflictStep>,
}

pub enum ConflictKind {
    /// Version constraint unsatisfiable
    UnsatisfiableConstraint,
    /// Cyclic dependency
    CyclicDependency,
    /// Tool doesn't exist
    UnknownTool,
    /// Platform not supported
    UnsupportedPlatform,
}

/// uv-style conflict explanation
/// 
/// Example:
/// ```
/// ✗ No solution found when resolving dependencies:
///   Because pre-commit requires uv>=0.5.0 and uv 0.5.0 is not available
///   for windows-x86, we can not satisfy pre-commit.
///
/// Suggestions:
///   • Try using a different platform
///   • Check if uv has a newer version that supports your platform
/// ```
```

### pubgrub Integration (Optional)

```toml
# crates/vx-solver/Cargo.toml

[package]
name = "vx-solver"
version.workspace = true
edition.workspace = true

[dependencies]
semver = "1.0"
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"

# Optional SAT solver
pubgrub = { version = "0.3", optional = true }

[features]
default = []
pubgrub = ["dep:pubgrub"]
```

### Migration Path

#### Phase 1: Create vx-solver (Non-breaking)

1. Create new `crates/vx-solver/`
2. Copy (not move) version/lockfile code to new crate
3. Add `vx-solver` as dependency to `vx-resolver`
4. Add re-exports for backward compatibility

```rust
// crates/vx-resolver/src/lib.rs (Phase 1)

// Backward-compatible re-exports
pub use vx_solver::{
    LockFile, LockFileError, LockedTool, 
    VersionConstraint, VersionRequest, VersionSolver,
};
```

#### Phase 2: Migrate Callers

1. Update `vx-cli` to use `vx-solver` directly
2. Update import paths in other crates
3. Run tests to ensure compatibility

#### Phase 3: Clean up vx-resolver

1. Remove `vx-resolver/version/` directory
2. Remove unnecessary re-exports
3. Update documentation

#### Phase 4: Enhance vx-solver (Optional)

1. Implement dependency graph solving
2. Add conflict explainer
3. Optional: Integrate pubgrub

### Naming Conventions

| Term | Definition | Crate |
|------|------------|-------|
| **Resolver** | Runtime status detection, check if installed | vx-resolver |
| **Executor** | Command forwarding and execution | vx-resolver |
| **Solver** | Version/dependency constraint solving | vx-solver |
| **LockFile** | Lock file management | vx-solver |

## Relationship with Existing RFCs

| RFC | Relationship |
|-----|-------------|
| RFC 0008 (Version Solver) | This RFC clarifies architecture for 0008; version resolution moves to vx-solver |

## Alternatives Considered

### Option A: No separation, continue extending vx-resolver

**Pros**:
- No migration needed
- Fewer crates

**Cons**:
- Unclear responsibilities
- Naming confusion (resolver vs solver)
- Harder to test independently

### Option B: Rename vx-resolver to vx-executor

**Pros**:
- More accurate naming

**Cons**:
- Many breaking changes
- Need to update all dependencies

### Option C: This RFC's separation proposal (Recommended)

**Pros**:
- Clear responsibility boundaries
- Gradual migration
- Easier to test and reuse independently
- Prepares for pubgrub integration

**Cons**:
- One more crate
- Migration work required

## Implementation Timeline

| Phase | Work | Estimated Time |
|-------|------|----------------|
| Phase 1 | Create vx-solver, copy code, add re-exports | 2 days |
| Phase 2 | Migrate callers, update import paths | 1 day |
| Phase 3 | Clean up vx-resolver, remove duplicates | 0.5 days |
| Phase 4 | Implement dependency graph solving and conflict explanation | 3-5 days |
| Phase 5 | (Optional) Integrate pubgrub | 3-5 days |

## Open Questions

1. **Should we use pubgrub?**
   - vx's dependency scenario is relatively simple (runtime tool dependencies, not deep package dependencies)
   - pubgrub's main advantage is complex constraint solving and conflict explanation
   - Suggestion: Start with simple implementation in Phase 4, integrate pubgrub if needed

2. **Does the lock file format need changes?**
   - Is current format sufficient to express dependency graph?
   - Do we need to add a `dependencies` field?

3. **Should vx-solver be async?**
   - Version fetching requires network requests
   - Suggestion: Core solving logic sync, version fetching abstracted via traits

## References

- [pubgrub (Astral-sh fork)](https://github.com/astral-sh/pubgrub) - PubGrub algorithm Rust implementation
- [resolvo](https://github.com/prefix-dev/resolvo) - CDCL SAT solver
- [RFC 0008: Version Solver](./0008-version-solver.md) - Version resolver design
- [uv resolver design](https://github.com/astral-sh/uv) - uv's dependency resolver
