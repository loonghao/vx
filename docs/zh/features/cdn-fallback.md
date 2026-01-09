# CDN 自动回退机制

## 概述

vx 的 CDN 加速功能现在支持智能回退机制。当 CDN 镜像不可用时，vx 会自动切换到原始 URL 进行下载，确保下载的可靠性。

## 工作原理

### 1. URL 优化阶段

当 CDN 加速启用时（通过 `VX_CDN_ENABLED=true`），vx 会尝试优化下载 URL：

```rust
let optimized = cdn_optimizer.optimize_url(original_url).await?;
```

优化结果包含两个 URL：
- **Primary URL**: CDN 优化后的 URL（如果优化成功）
- **Fallback URL**: 原始 URL（作为备用）

### 2. 自动回退下载

下载器会按顺序尝试所有可用的 URL：

1. **首先尝试 Primary URL**（CDN 镜像）
   - 如果成功，直接完成下载
   - 如果失败（连接超时、HTTP 错误等），继续下一步

2. **自动回退到 Fallback URL**（原始 URL）
   - 显示提示信息：`Retrying with original URL...`
   - 使用原始 URL 重新下载

### 3. 错误处理

支持的可恢复错误类型：
- 网络超时 (`NetworkTimeout`)
- 连接错误（`Connection error`）
- HTTP 状态码错误（`HTTP 4xx/5xx`）

只有在所有 URL 都失败后，才会返回错误。

## 使用示例

### 启用 CDN 加速

```bash
# 设置环境变量
export VX_CDN_ENABLED=true

# 安装工具（会自动使用 CDN 加速 + 回退机制）
vx install node@20.0.0
```

### 日志输出示例

#### CDN 成功的情况

```
[DEBUG] CDN URL optimized, original kept as fallback
[DEBUG] Downloading from: https://cdn.npmmirror.com/binaries/node/v20.0.0/node-v20.0.0-linux-x64.tar.gz
```

#### CDN 失败自动回退

```
[WARN] Download from CDN failed: Connection error, will try fallback
[WARN] Primary CDN URL failed, attempting fallback to original URL: https://nodejs.org/dist/v20.0.0/node-v20.0.0-linux-x64.tar.gz
[DEBUG] Fallback URL succeeded
```

## 配置选项

### 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `VX_CDN_ENABLED` | 启用/禁用 CDN 加速 | `false` |

### 编译时特性

CDN 功能需要启用 `cdn-acceleration` feature：

```toml
[dependencies]
vx-installer = { version = "0.6", features = ["cdn-acceleration"] }
```

## 技术实现

### OptimizedUrl 结构

```rust
pub struct OptimizedUrl {
    /// 主 URL（CDN 优化后的 URL）
    pub primary: String,
    /// 备用 URL（原始 URL）
    pub fallback: Option<String>,
}

impl OptimizedUrl {
    /// 获取所有可用 URL（按优先级排序）
    pub fn urls(&self) -> Vec<&str> {
        let mut urls = vec![self.primary.as_str()];
        if let Some(fallback) = &self.fallback {
            urls.push(fallback.as_str());
        }
        urls
    }
}
```

### 下载流程

```rust
async fn download_once(url: &str) -> Result<()> {
    let optimized = cdn_optimizer.optimize_url(url).await?;
    
    // 尝试所有可用的 URL
    for (index, download_url) in optimized.urls().iter().enumerate() {
        let is_fallback = index > 0;
        
        match try_download(download_url).await {
            Ok(_) => return Ok(()),
            Err(e) if is_fallback => return Err(e),  // 最后一个 URL，返回错误
            Err(e) => {
                warn!("CDN failed: {}, trying fallback", e);
                continue;  // 尝试下一个 URL
            }
        }
    }
}
```

## 优势

1. **提高成功率**：即使 CDN 不可用，也能从原始源下载
2. **透明操作**：用户无需手动干预，自动处理回退
3. **保持性能**：优先使用 CDN，仅在失败时回退
4. **详细日志**：清晰记录 CDN 使用和回退情况

## 与 turbo-cdn 的关系

### 不需要修改 turbo-cdn 接口

vx 的回退机制完全在应用层实现，无需修改 `turbo-cdn` 库：

- `turbo-cdn` 负责：URL 优化（提供 CDN 镜像 URL）
- `vx-installer` 负责：下载逻辑和回退机制

### turbo-cdn 的职责

```rust
// turbo-cdn 提供的功能
let optimized_url = turbo_cdn::optimize_url(original_url).await?;
```

返回结果：
- 成功：返回 CDN 镜像 URL
- 失败：返回错误（vx 会回退到原始 URL）

### vx 的职责

```rust
// vx 处理回退逻辑
match turbo_cdn::optimize_url(url).await {
    Ok(cdn_url) => OptimizedUrl {
        primary: cdn_url,
        fallback: Some(original_url),  // 保留原始 URL 作为备用
    },
    Err(_) => OptimizedUrl {
        primary: original_url,  // 优化失败，直接使用原始 URL
        fallback: None,
    }
}
```

## 测试

运行 CDN 回退机制的测试：

```bash
# 运行所有 CDN 相关测试
cargo test --package vx-installer --test cdn_fallback_tests

# 运行特定测试
cargo test --package vx-installer test_optimized_url_with_fallback
```

## 相关资源

- [turbo-cdn](https://github.com/loonghao/turbo-cdn) - CDN 优化库
- [环境变量配置](../config/env-vars.md)
