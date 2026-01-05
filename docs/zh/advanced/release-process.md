# 发布流程

本文档描述了 vx 的发布和软件包发布流程。

## 概述

vx 项目使用 GitHub Actions 自动化发布管道,处理以下任务:
- 使用 release-please 创建发布版本
- 为多个平台构建二进制文件
- 发布到各种包管理器(WinGet、Chocolatey、Homebrew、Scoop)

## 版本格式

### Git 标签

vx 使用以下版本标签格式:

- **Release-Please 格式**: `vx-v0.1.0`(release-please 使用的当前格式)
- **标准格式**: `v0.1.0`(传统的语义化版本)

### 版本规范化

不同的包管理器需要不同的版本格式:

| 包管理器 | 期望格式 | 示例 | 说明 |
|---------|---------|------|------|
| WinGet | `0.1.0` | `0.1.0` | 不带 `v` 前缀,从 `vx-v0.1.0` 规范化而来 |
| Chocolatey | `0.1.0` | `0.1.0` | 不带 `v` 前缀 |
| Homebrew | `0.1.0` | `0.1.0` | 不带 `v` 前缀 |
| Scoop | `0.1.0` | `0.1.0` | 不带 `v` 前缀 |

工作流会自动处理版本规范化,以确保与各个包管理器兼容。

## GitHub Actions 工作流

### 发布工作流 (`.github/workflows/release.yml`)

此工作流在推送到主分支时运行,处理:

1. **Release-Please**: 创建发布 PR 和标签
2. **二进制构建**: 为所有支持的平台构建二进制文件
3. **资源上传**: 将二进制文件上传到 GitHub 发布版本

**版本提取**:
```bash
# 提取版本号: vx-v0.1.0 -> 0.1.0, v0.1.0 -> 0.1.0
VERSION=$(echo "${TAG}" | sed -E 's/^(vx-)?v//')
```

### 包管理器工作流 (`.github/workflows/package-managers.yml`)

此工作流在发布工作流完成后运行,并发布到包管理器。

**WinGet 版本规范化**:
```bash
# 移除 'vx-' 前缀和 'v' 前缀 (vx-v0.1.0 -> 0.1.0)
normalized_version="${version#vx-}"
normalized_version="${normalized_version#v}"
```

这确保 WinGet 接收到 `0.1.0` 而不是 `vx-v0.1.0`,从而解决了 WinGet 显示类似 "x-v0.1.0" 的版本号问题。

#### 发布步骤

1. **检查发布**: 验证发布工作流是否成功
2. **获取版本**: 检索最新的发布版本
3. **规范化版本**: 为包管理器移除 `vx-` 前缀
4. **验证发布**: 确认 GitHub 发布版本存在
5. **发布**: 并行发布到各个包管理器

#### 支持的包管理器

- **WinGet** (`publish-winget`): 使用 `vedantmgoyal9/winget-releaser`
- **Chocolatey** (`publish-chocolatey`): 下载二进制文件并创建 `.nupkg`
- **Homebrew** (`publish-homebrew`): 生成带校验和的 formula
- **Scoop** (`publish-scoop`): 创建 JSON 清单

## 测试版本提取

项目包含测试脚本来验证版本提取逻辑:

### 测试版本规范化

运行测试脚本验证版本规范化:

```bash
bash scripts/test-winget-version.sh
```

这会测试以下转换:

| 输入 | 期望输出 | 描述 |
|-----|---------|------|
| `vx-v0.1.0` | `0.1.0` | 移除 `vx-` 和 `v` 前缀 |
| `vx-v1.0.0` | `1.0.0` | 移除 `vx-` 和 `v` 前缀 |
| `v0.1.0` | `0.1.0` | 移除 `v` 前缀 |
| `v1.0.0` | `1.0.0` | 移除 `v` 前缀 |

## 手动发布

### 手动触发包发布

如果需要手动触发包发布:

1. 转到 **Actions** → **Package Managers**
2. 点击 **Run workflow**
3. 输入版本标签(例如 `vx-v0.1.0` 或 `v0.1.0`)
4. 如果需要,勾选 **Force run**

### 发布到特定包管理器

每个包管理器都可以通过运行相应的作业独立发布。

## 故障排除

### WinGet 版本问题

**问题**: WinGet 显示类似 "x-v0.1.0" 的版本,而不是 "0.1.0"

**原因**: `release-tag` 参数接收完整的标签名 `vx-v0.1.0` 而没有正确规范化,需要移除 `vx-` 和 `v` 两个前缀。

**解决方案**: 工作流现在包含规范化步骤:

```yaml
- name: Normalize version for WinGet
  id: normalize
  run: |
    version="${{ steps.version.outputs.version }}"
    # 如果存在 'vx-' 前缀则移除 (vx-v0.1.0 -> v0.1.0)
    normalized_version="${version#vx-}"
    # 为 WinGet 移除 'v' 前缀 (v0.1.0 -> 0.1.0)
    normalized_version="${normalized_version#v}"
    echo "normalized_version=$normalized_version" >> $GITHUB_OUTPUT
```

### 验证发布资源

要验证发布资源是否可用:

```bash
# 检查发布是否存在
curl -s https://api.github.com/repos/loonghao/vx/releases/tags/vx-v0.1.0

# 列出资源
curl -s https://api.github.com/repos/loonghao/vx/releases/tags/vx-v0.1.0 | \
  jq -r '.assets[] | "\(.name) (\(.size) bytes)"'
```

## 最佳实践

1. **始终使用语义化版本**: `主版本.次版本.修订版`
2. **测试版本提取**: 发布前运行 `scripts/test-winget-version.sh`
3. **验证发布资源**: 确保所有平台的二进制文件都已上传
4. **监控包发布**: 检查每个包管理器的工作流状态
5. **更新文档**: 保持文档中的版本引用最新

## 相关文件

- `.github/workflows/release.yml` - 主发布工作流
- `.github/workflows/package-managers.yml` - 包发布工作流
- `scripts/test-winget-version.sh` - 版本规范化测试
- `distribution.toml` - 分发渠道配置
