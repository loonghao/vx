# 科学计算与 HPC 工具

vx 支持科学计算和高性能计算（HPC）工具。

## Spack

专为超级计算机、Linux 和 macOS 设计的灵活包管理器。

```bash
vx install `spack@latest

vx spack --version
vx spack list                    # 列出可用包
vx spack info openmpi            # 显示包信息
vx spack install openmpi         # 安装包
vx spack find                    # 列出已安装的包
vx spack load openmpi            # 加载包到环境
```

**主要特性：**

- 科学软件包管理
- 多版本和配置支持
- 编译器管理
- HPC 集群支持
- 可复现环境

**支持的平台：**

- Linux（x64、ARM64）
- macOS（Intel、Apple Silicon）
- Windows（通过 WSL）

**工作流示例：**

```bash
# 安装 Spack
vx install `spack@latest

# 安装科学计算包
vx spack install python@3.11
vx spack install numpy
vx spack install openmpi +cuda

# 创建环境
vx spack env create myenv
vx spack env activate myenv
vx spack install hdf5 +mpi
```

**项目配置：**

```toml
[tools]
spack = "latest"

[scripts]
spack-setup = "spack install python numpy scipy"
spack-env = "spack env activate research"
```

## Rez

VFX/动画行业的跨平台包管理器。

```bash
vx install `rez@latest

vx rez --version
vx rez env package            # 进入包环境
vx rez build                  # 构建包
vx rez release                # 发布包
```

**主要特性：**

- VFX 流水线集成
- 环境解析
- 版本冲突处理
- 工作室级包管理

## 科学计算配置

```toml
[tools]
spack = "latest"
rez = "latest"
cmake = "latest"
ninja = "latest"

[scripts]
# HPC 设置
hpc-setup = "spack install openmpi hdf5 +mpi"

# 构建科学代码
build = "cmake -B build -G Ninja && ninja -C build"

# VFX 环境
vfx-env = "rez env maya-2024 houdini-20"
```

## 最佳实践

1. **环境隔离**：使用 Spack 环境管理项目特定依赖
2. **编译器选择**：指定编译器以获得可复现的构建
3. **模块集成**：与 HPC 模块系统集成（Lmod、Environment Modules）

```bash
# 使用特定编译器
vx spack install package %gcc@12

# 创建可复现环境
vx spack env create --with-view myproject
vx spack concretize
vx spack install
```
