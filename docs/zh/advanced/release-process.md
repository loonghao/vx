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

#### 工作流触发逻辑

发布工作流使用复杂的触发机制来处理不同场景:

| 场景 | Release-Please 作业 | 构建作业 | 说明 |
|------|-------------------|---------|------|
| 常规推送 (feat, fix 等) | 运行 | 如果创建了发布则触发 | 正常开发流程 |
| 发布 PR 合并 (`chore: release vX.Y.Z`) | **跳过** | **触发** | 从提交消息中提取版本 |
| Dependabot PR (`chore(deps): bump...`) | 跳过 | 不触发 | 防止重复构建 |
| 手动工作流触发 | 跳过 | 触发 | 紧急/手动发布 |

**核心逻辑**:
```yaml
# Release-please 作业跳过发布提交以防止递归
if: |
  github.event_name == 'push' &&
  github.ref == 'refs/heads/main' &&
  !contains(github.event.head_commit.message, 'chore: release') &&
  github.event.head_commit.author.name != 'github-actions[bot]'

# 构建作业在以下情况触发:
# 1. Release-please 创建了发布
# 2. 发布 PR 合并 (通过提交消息检测)
# 3. 手动工作流触发
if: |
  always() &&
  (
    (needs.release-please.result == 'success' && needs.release-please.outputs.release_created == 'true') ||
    github.event_name == 'workflow_dispatch' ||
    (github.event_name == 'push' && contains(github.event.head_commit.message, 'chore: release'))
  )
```

这确保了当发布 PR 被合并时(例如 "chore: release v0.6.24")，即使 release-please 被跳过，构建作业仍会运行。

### 发布工作流中的 WinGet 发布

从 v0.7.9 开始,WinGet 发布已直接集成到发布工作流 (`release.yml`) 中作为 `publish-winget` 作业。这确保了:

1. WinGet 在发布资源上传后立即发布(无 `workflow_run` 延迟)
2. 发布标签是精确已知的(来自 `plan` 作业输出),避免 API 版本查找问题
3. 预发布过滤与其他发布作业保持一致

**安装程序正则表达式**:
```yaml
# 仅匹配 cargo-dist 原始的不带版本号的 zip 文件,以避免重复条目。
# cargo-dist 生成: vx-x86_64-pc-windows-msvc.zip (不带版本号)
# release.yml 还会创建: vx-0.7.8-x86_64-pc-windows-msvc.zip (带版本号的副本)
# 必须只匹配其中一个,以避免 winget 清单中出现重复的安装程序条目。
installers-regex: 'vx-(x86_64|aarch64)-pc-windows-msvc\.zip$'
```

### 包管理器工作流 (`.github/workflows/package-managers.yml`)

此工作流在发布工作流完成后运行,并发布到包管理器。
它作为 WinGet 发布的**备份**(以防 `release.yml` 中的 publish-winget 作业失败),
同时也是 Chocolatey 和 Scoop 的**主要**发布者。

**WinGet 版本规范化**:
```bash
# 鲁棒地去除所有已知前缀: vx-v0.7.8 -> 0.7.8, v0.7.8 -> 0.7.8
normalized_version="${version}"
normalized_version="${normalized_version#vx-v}"
normalized_version="${normalized_version#vx-}"
normalized_version="${normalized_version#v}"
```

这确保 WinGet 接收到 `0.1.0` 而不是 `vx-v0.1.0`,从而解决了 WinGet 显示类似 "x-v0.1.0" 的版本号问题。

#### 发布步骤

1. **检查发布**: 验证发布工作流是否成功
2. **获取版本**: 检索最新的发布版本
3. **规范化版本**: 为包管理器去除所有标签前缀
4. **验证发布**: 确认 GitHub 发布版本存在
5. **发布**: 并行发布到各个包管理器

#### 支持的包管理器

- **WinGet** (`publish-winget`): 使用 `vedantmgoyal9/winget-releaser`(同时也在 `release.yml` 中)
- **Chocolatey** (`publish-chocolatey`): 下载二进制文件并创建 `.nupkg`
- **Homebrew**: 由 cargo-dist 的 `publish-homebrew-formula` 作业在 `release.yml` 中处理
- **Scoop** (`publish-scoop`): 创建 JSON 清单

## 测试发布工作流逻辑

项目包含测试来验证发布工作流触发逻辑:

### 运行工作流测试

```bash
cargo test --test release_workflow_tests
```

这验证:
- 从提交消息中提取版本
- 版本规范化
- 发布提交检测
- 不同场景的工作流触发条件

### 测试版本提取

项目还包含测试脚本来验证版本提取逻辑:

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

### 发布工作流未触发

**问题**: 合并发布 PR 后(例如 "chore: release v0.6.24")，构建作业没有运行。

**原因**: 原始工作流逻辑要求 `release-please` 作业成功并创建发布。但是，当发布 PR 被合并时，`release-please` 作业会被有意跳过以防止递归 PR 创建。这导致 `get-tag` 作业(及后续构建作业)也被跳过。

**解决方案**: 工作流现在包含额外的条件来检测发布 PR 合并:

```yaml
# 构建作业现在在发布 PR 合并时触发
if: |
  always() &&
  (
    (needs.release-please.result == 'success' && needs.release-please.outputs.release_created == 'true') ||
    github.event_name == 'workflow_dispatch' ||
    (github.event_name == 'push' && contains(github.event.head_commit.message, 'chore: release'))  # <-- 新增
  )
```

当提交消息包含 "chore: release" 时，工作流会从消息中提取版本并继续构建。

### WinGet 版本问题

**问题**: WinGet 显示类似 "x-v0.1.0" 的版本,而不是 "0.1.0"

**原因**: `release-tag` 参数接收完整的标签名 `vx-v0.1.0` 而没有正确规范化,需要移除 `vx-` 和 `v` 两个前缀。

**解决方案**: 工作流现在包含鲁棒的规范化步骤,可处理所有已知的标签格式:

```yaml
- name: Normalize version for WinGet
  id: normalize
  run: |
    version="${{ steps.version.outputs.version }}"
    # 鲁棒地去除所有已知前缀
    normalized_version="${version}"
    normalized_version="${normalized_version#vx-v}"
    normalized_version="${normalized_version#vx-}"
    normalized_version="${normalized_version#v}"
    echo "normalized_version=$normalized_version" >> $GITHUB_OUTPUT
```

### WinGet 重复安装程序条目

**问题**: WinGet 清单 PR 中包含同一架构的重复安装程序条目。

**原因**: 发布版本中同时包含带版本号的 (`vx-0.7.8-x86_64-pc-windows-msvc.zip`) 和
不带版本号的 (`vx-x86_64-pc-windows-msvc.zip`) 同一构件的副本。如果 `installers-regex`
同时匹配两者,komac 会创建两个安装程序条目。

**解决方案**: 使用精确的正则表达式,仅匹配 cargo-dist 原始的不带版本号的文件:

```yaml
# 仅匹配 "vx-{arch}-pc-windows-msvc.zip" (排除 "vx-0.7.8-{arch}-...")
installers-regex: 'vx-(x86_64|aarch64)-pc-windows-msvc\.zip$'
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

### 发布提交触发不必要的 CI 运行

**问题**: 当 Release Please 合并发布 PR（例如 "chore: release v0.7.6"）时，产生的提交会不必要地触发 CI、CodeQL 和 Benchmark 工作流，浪费大量 CI 资源（CI 超过 15 分钟，CodeQL 超过 12 分钟）。

**原因**: CI、CodeQL 和 Benchmark 工作流在推送到 main 分支时没有过滤机制来排除发布提交。由于发布提交会修改 `Cargo.toml` 和 `Cargo.lock`（版本号更新），即使有路径过滤的工作流（如 Benchmark）也会被触发。

**解决方案**: 在作业级别添加 `if` 条件来跳过发布提交：

```yaml
# 跳过发布提交（应用于 CI、CodeQL 和 Benchmark）
if: >-
  github.event_name != 'push' ||
  !startsWith(github.event.head_commit.message, 'chore: release')
```

此条件：
- 允许作业在 PR、定时运行和手动触发时正常运行
- 仅当事件为推送且提交消息以 `chore: release` 开头时跳过
- 当工作流中的第一个作业被跳过时，所有依赖的下游作业也会自动跳过

**受影响的工作流**：
- `.github/workflows/ci.yml` - `detect-changes` 作业（控制所有下游 CI 作业）
- `.github/workflows/codeql.yml` - `analyze` 作业
- `.github/workflows/benchmark.yml` - `benchmark` 作业

**不受影响**（有意为之）：
- `.github/workflows/release-please.yml` - 必须在发布提交时继续运行，以检测 `releases_created` 并触发 Release 工作流

## 最佳实践

1. **始终使用语义化版本**: `主版本.次版本.修订版`
2. **测试版本提取**: 发布前运行 `scripts/test-winget-version.sh`
3. **验证发布资源**: 确保所有平台的二进制文件都已上传
4. **监控包发布**: 检查每个包管理器的工作流状态
5. **更新文档**: 保持文档中的版本引用最新

## 相关文件

- `.github/workflows/release.yml` - 主发布工作流
- `.github/workflows/release-please.yml` - Release Please 工作流（创建发布 PR 和标签）
- `.github/workflows/package-managers.yml` - 包发布工作流
- `.github/workflows/ci.yml` - CI 工作流（跳过发布提交）
- `.github/workflows/codeql.yml` - CodeQL 分析（跳过发布提交）
- `.github/workflows/benchmark.yml` - 性能基准测试（跳过发布提交）
- `scripts/test-winget-version.sh` - 版本规范化测试
- `distribution.toml` - 分发渠道配置
