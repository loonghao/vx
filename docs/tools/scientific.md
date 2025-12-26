# Scientific & HPC Tools

vx supports tools for scientific computing and High-Performance Computing (HPC).

## Spack

A flexible package manager designed for supercomputers, Linux, and macOS.

```bash
vx install spack latest

vx spack --version
vx spack list                    # List available packages
vx spack info openmpi            # Show package info
vx spack install openmpi         # Install a package
vx spack find                    # List installed packages
vx spack load openmpi            # Load package into environment
```

**Key Features:**

- Scientific software package management
- Multiple versions and configurations
- Compiler management
- HPC cluster support
- Reproducible environments

**Supported Platforms:**

- Linux (x64, ARM64)
- macOS (Intel, Apple Silicon)
- Windows (via WSL)

**Example Workflow:**

```bash
# Install Spack
vx install spack latest

# Install scientific packages
vx spack install python@3.11
vx spack install numpy
vx spack install openmpi +cuda

# Create an environment
vx spack env create myenv
vx spack env activate myenv
vx spack install hdf5 +mpi
```

**Project Configuration:**

```toml
[tools]
spack = "latest"

[scripts]
spack-setup = "spack install python numpy scipy"
spack-env = "spack env activate research"
```

## Rez

Cross-platform package manager for VFX/animation industry.

```bash
vx install rez latest

vx rez --version
vx rez env package            # Enter package environment
vx rez build                  # Build a package
vx rez release                # Release a package
```

**Key Features:**

- VFX pipeline integration
- Environment resolution
- Version conflict handling
- Studio-wide package management

## Scientific Computing Configuration

```toml
[tools]
spack = "latest"
rez = "latest"
cmake = "latest"
ninja = "latest"

[scripts]
# HPC setup
hpc-setup = "spack install openmpi hdf5 +mpi"

# Build scientific code
build = "cmake -B build -G Ninja && ninja -C build"

# VFX environment
vfx-env = "rez env maya-2024 houdini-20"
```

## Best Practices

1. **Environment Isolation**: Use Spack environments for project-specific dependencies
2. **Compiler Selection**: Specify compilers for reproducible builds
3. **Module Integration**: Integrate with HPC module systems (Lmod, Environment Modules)

```bash
# Use specific compiler
vx spack install package %gcc@12

# Create reproducible environment
vx spack env create --with-view myproject
vx spack concretize
vx spack install
```
