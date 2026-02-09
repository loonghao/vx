# CDN Automatic Fallback Mechanism

## Overview

vx's CDN acceleration now supports an intelligent fallback mechanism. When CDN mirrors are unavailable, vx automatically switches to the original URL for downloads, ensuring download reliability.

## How It Works

### 1. URL Optimization Phase

When CDN acceleration is enabled (via `VX_CDN_ENABLED=true`), vx attempts to optimize the download URL:

```rust
let optimized = cdn_optimizer.optimize_url(original_url).await?;
```

The optimization result contains two URLs:
- **Primary URL**: CDN-optimized URL (if optimization succeeded)
- **Fallback URL**: Original URL (as backup)

### 2. Automatic Fallback Download

The downloader tries all available URLs in sequence:

1. **Try Primary URL First** (CDN mirror)
   - If successful, download completes
   - If failed (timeout, HTTP error, etc.), proceed to next step

2. **Automatically Fallback to Fallback URL** (original URL)
   - Show message: `Retrying with original URL...`
   - Retry download using original URL

### 3. Error Handling

Supported recoverable error types:
- Network timeout (`NetworkTimeout`)
- Connection errors (`Connection error`)
- HTTP status code errors (`HTTP 4xx/5xx`)

Only returns error when all URLs have failed.

## Usage Examples

### Enable CDN Acceleration

```bash
# Set environment variable
export VX_CDN_ENABLED=true

# Install tool (automatically uses CDN acceleration + fallback)
vx install node@20.0.0
```

### Log Output Examples

#### CDN Success Case

```
[DEBUG] CDN URL optimized, original kept as fallback
[DEBUG] Downloading from: https://cdn.npmmirror.com/binaries/node/v20.0.0/node-v20.0.0-linux-x64.tar.gz
```

#### CDN Failure with Automatic Fallback

```
[WARN] Download from CDN failed: Connection error, will try fallback
[WARN] Primary CDN URL failed, attempting fallback to original URL: https://nodejs.org/dist/v20.0.0/node-v20.0.0-linux-x64.tar.gz
[DEBUG] Fallback URL succeeded
```

## Configuration Options

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `VX_CDN_ENABLED` | Enable/disable CDN acceleration | `false` |

### Compile-time Features

CDN functionality requires the `cdn-acceleration` feature:

```toml
[dependencies]
vx-installer = { version = "0.6", features = ["cdn-acceleration"] }
```

## Technical Implementation

### OptimizedUrl Structure

```rust
pub struct OptimizedUrl {
    /// Primary URL (CDN-optimized URL)
    pub primary: String,
    /// Fallback URL (original URL)
    pub fallback: Option<String>,
}

impl OptimizedUrl {
    /// Get all available URLs (sorted by priority)
    pub fn urls(&self) -> Vec<&str> {
        let mut urls = vec![self.primary.as_str()];
        if let Some(fallback) = &self.fallback {
            urls.push(fallback.as_str());
        }
        urls
    }
}
```

### Download Flow

```rust
async fn download_once(url: &str) -> Result<()> {
    let optimized = cdn_optimizer.optimize_url(url).await?;
    
    // Try all available URLs
    for (index, download_url) in optimized.urls().iter().enumerate() {
        let is_fallback = index > 0;
        
        match try_download(download_url).await {
            Ok(_) => return Ok(()),
            Err(e) if is_fallback => return Err(e),  // Last URL, return error
            Err(e) => {
                warn!("CDN failed: {}, trying fallback", e);
                continue;  // Try next URL
            }
        }
    }
}
```

## Benefits

1. **Higher Success Rate**: Downloads succeed even when CDN is unavailable
2. **Transparent Operation**: Users don't need manual intervention, automatic fallback handling
3. **Maintains Performance**: Prioritizes CDN, only falls back on failure
4. **Detailed Logging**: Clear records of CDN usage and fallback situations

## Relationship with turbo-cdn

### No Need to Modify turbo-cdn Interface

vx's fallback mechanism is implemented entirely at the application layer, without requiring modifications to the `turbo-cdn` library:

- `turbo-cdn` is responsible for: URL optimization (providing CDN mirror URLs)
- `vx-installer` is responsible for: Download logic and fallback mechanism

### turbo-cdn's Responsibility

```rust
// Functionality provided by turbo-cdn
let optimized_url = turbo_cdn::optimize_url(original_url).await?;
```

Return result:
- Success: Returns CDN mirror URL
- Failure: Returns error (vx falls back to original URL)

### vx's Responsibility

```rust
// vx handles fallback logic
match turbo_cdn::optimize_url(url).await {
    Ok(cdn_url) => OptimizedUrl {
        primary: cdn_url,
        fallback: Some(original_url),  // Keep original URL as backup
    },
    Err(_) => OptimizedUrl {
        primary: original_url,  // Optimization failed, use original URL directly
        fallback: None,
    }
}
```

## Testing

Run CDN fallback mechanism tests:

```bash
# Run all CDN-related tests
cargo test --package vx-installer --test cdn_fallback_tests

# Run specific test
cargo test --package vx-installer test_optimized_url_with_fallback
```

## Related Resources

- [turbo-cdn](https://github.com/loonghao/turbo-cdn) - CDN optimization library
- [Environment Variables Configuration](../config/env-vars.md)
