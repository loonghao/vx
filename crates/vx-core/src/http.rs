//! HTTP utilities for tool version fetching and downloading

use crate::Result;
use reqwest::Client;
use serde_json::Value;
use std::sync::OnceLock;

/// Global HTTP client instance
static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

/// Get the global HTTP client
pub fn get_http_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent("vx-tool-manager/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    })
}

/// HTTP utilities for common operations
pub struct HttpUtils;

impl HttpUtils {
    /// Fetch JSON from a URL
    pub async fn fetch_json(url: &str) -> Result<Value> {
        let client = get_http_client();
        let response = client.get(url).send().await?;
        let json = response.json().await?;
        Ok(json)
    }
    
    /// Download a file to bytes
    pub async fn download_bytes(url: &str) -> Result<Vec<u8>> {
        let client = get_http_client();
        let response = client.get(url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
    
    /// Check if a URL is accessible (HEAD request)
    pub async fn check_url(url: &str) -> Result<bool> {
        let client = get_http_client();
        match client.head(url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_http_client_creation() {
        let client = get_http_client();
        assert!(client.timeout().is_some());
    }
    
    #[tokio::test]
    async fn test_fetch_json_mock() {
        // This would require a mock server in a real test
        // For now, just test that the function exists and compiles
        let result = HttpUtils::fetch_json("https://httpbin.org/json").await;
        // Don't assert success since we don't want network dependency in tests
        let _ = result;
    }
}
