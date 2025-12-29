# 环境变量

vx 支持以下环境变量来配置其行为。

## 配置变量

| 变量 | 描述 | 默认值 |
|------|------|--------|
| `VX_HOME` | vx 数据目录 | `~/.local/share/vx` |
| `VX_CONFIG_DIR` | 配置目录 | `~/.config/vx` |
| `VX_CACHE_DIR` | 缓存目录 | `~/.cache/vx` |

## 行为变量

| 变量 | 描述 | 默认值 |
|------|------|--------|
| `VX_AUTO_INSTALL` | 启用自动安装 | `true` |
| `VX_AUTO_SWITCH` | 启用自动环境切换 | `true` |
| `VX_VERBOSE` | 启用详细输出 | `false` |
| `VX_DEBUG` | 启用调试输出 | `false` |

## CDN 加速

vx 通过 [turbo-cdn](https://github.com/loonghao/turbo-cdn) 支持 CDN 加速下载，这可以显著提高下载速度，尤其是在访问 GitHub 较慢的地区（如中国）。

### 工作原理

当 CDN 加速启用时：

1. 下载 URL 会自动优化为使用最佳可用的 CDN 镜像
2. 进度显示会在 CDN 激活时显示 `[CDN]` 标识
3. 如果 CDN 优化失败，会自动回退到原始 URL

### 启用 CDN 加速

当 vx 使用 `cdn-acceleration` 特性编译时，CDN 加速默认启用。官方发布的二进制文件包含此特性。

### 支持的源

CDN 加速适用于：

- GitHub Releases（如 Node.js、Go、Zig、Rust 工具）
- GitHub 原始内容
- npm 仓库
- PyPI 包
- 更多源（通过 turbo-cdn 的镜像网络）

## 运行时变量

这些变量在环境激活时由 vx 设置：

| 变量 | 描述 |
|------|------|
| `VX_ENV` | 当前环境名称 |
| `VX_ENV_DIR` | 环境目录路径 |
| `VX_PROJECT_DIR` | 项目目录路径 |

## 示例

```bash
# 禁用自动安装
export VX_AUTO_INSTALL=false

# 启用详细输出
export VX_VERBOSE=true

# 自定义数据目录
export VX_HOME=/opt/vx

# 运行命令
vx node --version
```

## 优先级

配置按以下顺序解析（后面覆盖前面）：

1. 内置默认值
2. 全局配置文件
3. 项目配置文件
4. 环境变量
5. 命令行标志
