//! Download performance benchmarks for vx-installer
//!
//! These benchmarks compare download performance with and without CDN acceleration.
//!
//! Run with: cargo bench -p vx-installer
//!
//! For CDN-enabled benchmarks:
//! cargo bench -p vx-installer --features cdn-acceleration

#[cfg(feature = "cdn-acceleration")]
use criterion::BenchmarkId;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Test URLs for benchmarking
/// Using small files to keep benchmark times reasonable
mod test_urls {
    /// Small file from GitHub releases (~1KB)
    pub const GITHUB_SMALL: &str =
        "https://github.com/loonghao/vx/releases/download/v0.4.0/checksums.txt";

    /// Medium file from GitHub releases (~100KB)
    pub const GITHUB_MEDIUM: &str = "https://raw.githubusercontent.com/loonghao/vx/main/Cargo.lock";

    /// Go download page JSON (~50KB)
    pub const GO_VERSION_JSON: &str = "https://go.dev/dl/?mode=json";

    /// Node.js version index (~200KB)
    pub const NODE_VERSION_INDEX: &str = "https://nodejs.org/dist/index.json";
}

/// Benchmark URL fetching without CDN
fn bench_fetch_without_cdn(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("fetch_without_cdn");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    // Small file benchmark
    group.throughput(Throughput::Elements(1));
    group.bench_function("github_small", |b| {
        b.to_async(&rt).iter(|| async {
            let client = reqwest::Client::new();
            let response = client.get(test_urls::GITHUB_SMALL).send().await;
            black_box(response.ok().map(|r| r.status().is_success()))
        });
    });

    // Medium file benchmark
    group.bench_function("github_medium", |b| {
        b.to_async(&rt).iter(|| async {
            let client = reqwest::Client::new();
            let response = client.get(test_urls::GITHUB_MEDIUM).send().await;
            black_box(response.ok().map(|r| r.status().is_success()))
        });
    });

    // Go version JSON
    group.bench_function("go_version_json", |b| {
        b.to_async(&rt).iter(|| async {
            let client = reqwest::Client::new();
            let response = client.get(test_urls::GO_VERSION_JSON).send().await;
            black_box(response.ok().map(|r| r.status().is_success()))
        });
    });

    // Node.js version index
    group.bench_function("node_version_index", |b| {
        b.to_async(&rt).iter(|| async {
            let client = reqwest::Client::new();
            let response = client.get(test_urls::NODE_VERSION_INDEX).send().await;
            black_box(response.ok().map(|r| r.status().is_success()))
        });
    });

    group.finish();
}

/// Benchmark URL fetching with CDN optimization
#[cfg(feature = "cdn-acceleration")]
fn bench_fetch_with_cdn(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("fetch_with_cdn");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    group.throughput(Throughput::Elements(1));

    // Small file with CDN
    group.bench_function("github_small_cdn", |b| {
        b.to_async(&rt).iter(|| async {
            let optimizer = vx_installer::CdnOptimizer::new(true);
            let optimized_url = optimizer
                .optimize_url(test_urls::GITHUB_SMALL)
                .await
                .unwrap_or_else(|_| test_urls::GITHUB_SMALL.to_string());

            let client = reqwest::Client::new();
            let response = client.get(&optimized_url).send().await;
            black_box(response.ok().map(|r| r.status().is_success()))
        });
    });

    // Medium file with CDN
    group.bench_function("github_medium_cdn", |b| {
        b.to_async(&rt).iter(|| async {
            let optimizer = vx_installer::CdnOptimizer::new(true);
            let optimized_url = optimizer
                .optimize_url(test_urls::GITHUB_MEDIUM)
                .await
                .unwrap_or_else(|_| test_urls::GITHUB_MEDIUM.to_string());

            let client = reqwest::Client::new();
            let response = client.get(&optimized_url).send().await;
            black_box(response.ok().map(|r| r.status().is_success()))
        });
    });

    group.finish();
}

/// Benchmark CDN URL optimization latency
#[cfg(feature = "cdn-acceleration")]
fn bench_cdn_optimization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cdn_optimization");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    let test_urls = vec![
        (
            "github_release",
            "https://github.com/user/repo/releases/download/v1.0/file.zip",
        ),
        (
            "pypi",
            "https://files.pythonhosted.org/packages/xx/yy/package.whl",
        ),
        (
            "golang",
            "https://golang.org/dl/go1.21.0.linux-amd64.tar.gz",
        ),
    ];

    for (name, url) in test_urls {
        group.bench_with_input(BenchmarkId::new("optimize", name), &url, |b, url| {
            b.to_async(&rt).iter(|| async {
                let optimizer = vx_installer::CdnOptimizer::new(true);
                black_box(optimizer.optimize_url(url).await)
            });
        });
    }

    group.finish();
}

/// Benchmark download with actual file transfer
fn bench_download_transfer(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("download_transfer");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(5);

    // Benchmark actual download throughput
    group.throughput(Throughput::Bytes(1024)); // Approximate size

    group.bench_function("download_small_file", |b| {
        b.to_async(&rt).iter(|| async {
            let client = reqwest::Client::new();
            let response = client.get(test_urls::GITHUB_SMALL).send().await;
            if let Ok(resp) = response {
                let bytes = resp.bytes().await;
                black_box(bytes.ok().map(|b| b.len()))
            } else {
                black_box(None)
            }
        });
    });

    group.finish();
}

/// Benchmark comparison: direct vs CDN
#[cfg(feature = "cdn-acceleration")]
fn bench_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("direct_vs_cdn");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    let url = test_urls::GITHUB_SMALL;

    // Direct download
    group.bench_function("direct", |b| {
        b.to_async(&rt).iter(|| async {
            let client = reqwest::Client::new();
            let response = client.get(url).send().await;
            if let Ok(resp) = response {
                black_box(resp.bytes().await.ok())
            } else {
                black_box(None)
            }
        });
    });

    // CDN optimized download
    group.bench_function("cdn_optimized", |b| {
        b.to_async(&rt).iter(|| async {
            let optimizer = vx_installer::CdnOptimizer::new(true);
            let optimized_url = optimizer
                .optimize_url(url)
                .await
                .unwrap_or_else(|_| url.to_string());

            let client = reqwest::Client::new();
            let response = client.get(&optimized_url).send().await;
            if let Ok(resp) = response {
                black_box(resp.bytes().await.ok())
            } else {
                black_box(None)
            }
        });
    });

    group.finish();
}

// Configure criterion groups based on features
#[cfg(feature = "cdn-acceleration")]
criterion_group!(
    benches,
    bench_fetch_without_cdn,
    bench_fetch_with_cdn,
    bench_cdn_optimization,
    bench_download_transfer,
    bench_comparison,
);

#[cfg(not(feature = "cdn-acceleration"))]
criterion_group!(benches, bench_fetch_without_cdn, bench_download_transfer,);

criterion_main!(benches);
