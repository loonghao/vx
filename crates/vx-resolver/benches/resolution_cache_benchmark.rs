//! Resolution cache performance benchmarks for vx-resolver
//!
//! These benchmarks compare resolution performance with and without cache hits.
//!
//! Run with: cargo bench -p vx-resolver
//!
//! Results help validate RFC 0011 goals:
//! - Cache hit should be significantly faster than cache miss
//! - Repeated resolutions should benefit from caching

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use vx_resolver::{
    ResolutionCache, ResolutionCacheKey, ResolvedGraph, RESOLUTION_CACHE_SCHEMA_VERSION,
};
use vx_runtime::CacheMode;

/// Create a test cache key with configurable parameters
fn make_cache_key(cwd: PathBuf, runtime: &str, args_count: usize) -> ResolutionCacheKey {
    let args: Vec<String> = (0..args_count).map(|i| format!("arg{}", i)).collect();
    let args_digest = format!("{:x}", md5_hash(&args.join("\0")));

    ResolutionCacheKey {
        schema_version: RESOLUTION_CACHE_SCHEMA_VERSION,
        vx_version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        cwd,
        runtime: runtime.to_string(),
        version: None,
        args_digest,
        config_digest: None,
        lock_digest: None,
        prefer_vx_managed: true,
        fallback_to_system: true,
    }
}

/// Create a test resolved graph with configurable complexity
fn make_resolved_graph(runtime: &str, deps_count: usize) -> ResolvedGraph {
    let missing_deps: Vec<String> = (0..deps_count).map(|i| format!("dep{}", i)).collect();
    let install_order = missing_deps.clone();

    ResolvedGraph {
        runtime: runtime.to_string(),
        executable: PathBuf::from(runtime),
        command_prefix: vec![],
        missing_dependencies: missing_deps,
        install_order,
        runtime_needs_install: deps_count > 0,
        incompatible_dependencies: vec![],
    }
}

/// Simple hash function for args digest (not cryptographic, just for testing)
fn md5_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Benchmark cache read operations (hit vs miss)
fn bench_cache_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolution_cache_read");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // Test different runtime types
    let runtimes = ["node", "npm", "npx", "go", "uv", "uvx", "cargo"];

    for runtime in runtimes {
        let dir = TempDir::new().unwrap();
        let cache = ResolutionCache::new(dir.path().to_path_buf())
            .with_mode(CacheMode::Normal)
            .with_ttl(Duration::from_secs(900)); // 15 minutes

        let key = make_cache_key(dir.path().to_path_buf(), runtime, 5);
        let value = make_resolved_graph(runtime, 2);

        // Pre-populate cache for hit test
        cache.set(&key, &value).unwrap();

        // Benchmark cache hit
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("cache_hit", runtime), &key, |b, key| {
            b.iter(|| {
                let result = cache.get(black_box(key));
                black_box(result)
            });
        });

        // Benchmark cache miss (different key)
        let miss_key = make_cache_key(dir.path().to_path_buf(), &format!("{}_miss", runtime), 5);
        group.bench_with_input(
            BenchmarkId::new("cache_miss", runtime),
            &miss_key,
            |b, key| {
                b.iter(|| {
                    let result = cache.get(black_box(key));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cache write operations
fn bench_cache_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolution_cache_write");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    // Test different graph complexities
    let complexities = [0, 2, 5, 10];

    for deps_count in complexities {
        let dir = TempDir::new().unwrap();
        let cache = ResolutionCache::new(dir.path().to_path_buf())
            .with_mode(CacheMode::Normal)
            .with_ttl(Duration::from_secs(900));

        let value = make_resolved_graph("node", deps_count);

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("write", format!("{}_deps", deps_count)),
            &deps_count,
            |b, &deps_count| {
                let mut counter = 0u64;
                b.iter(|| {
                    // Use unique key each iteration to avoid overwrite optimization
                    let key = make_cache_key(
                        dir.path().to_path_buf(),
                        &format!("node_{}", counter),
                        deps_count,
                    );
                    counter += 1;
                    let result = cache.set(black_box(&key), black_box(&value));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cache key hashing
fn bench_cache_key_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolution_cache_key_hash");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    // Test different key complexities
    let args_counts = [0, 5, 20, 100];

    for args_count in args_counts {
        let dir = TempDir::new().unwrap();
        let key = make_cache_key(dir.path().to_path_buf(), "node", args_count);

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("hash", format!("{}_args", args_count)),
            &key,
            |b, key| {
                b.iter(|| {
                    let hash = black_box(key).hash_hex();
                    black_box(hash)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark repeated resolution pattern (simulating real-world usage)
fn bench_repeated_resolution_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolution_pattern");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(50);

    let dir = TempDir::new().unwrap();
    let cache = ResolutionCache::new(dir.path().to_path_buf())
        .with_mode(CacheMode::Normal)
        .with_ttl(Duration::from_secs(900));

    // Simulate typical development pattern: same command repeated many times
    let key = make_cache_key(dir.path().to_path_buf(), "npm", 10);
    let value = make_resolved_graph("npm", 3);

    // First resolution (cache miss + write)
    group.bench_function("first_resolution_cold", |b| {
        b.iter(|| {
            // Simulate cold cache - create new cache each time
            let temp_dir = TempDir::new().unwrap();
            let cold_cache = ResolutionCache::new(temp_dir.path().to_path_buf())
                .with_mode(CacheMode::Normal)
                .with_ttl(Duration::from_secs(900));
            let cold_key = make_cache_key(temp_dir.path().to_path_buf(), "npm", 10);

            // Cache miss
            let miss = cold_cache.get(black_box(&cold_key));
            assert!(miss.is_none());

            // Write to cache
            cold_cache
                .set(black_box(&cold_key), black_box(&value))
                .unwrap();

            black_box(miss)
        });
    });

    // Pre-populate for warm cache tests
    cache.set(&key, &value).unwrap();

    // Subsequent resolutions (cache hit)
    group.bench_function("subsequent_resolution_warm", |b| {
        b.iter(|| {
            let hit = cache.get(black_box(&key));
            black_box(hit)
        });
    });

    // Mixed pattern: 90% hits, 10% misses (realistic scenario)
    group.bench_function("mixed_pattern_90_10", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            counter += 1;
            if counter % 10 == 0 {
                // 10% miss
                let miss_key = make_cache_key(
                    dir.path().to_path_buf(),
                    &format!("npm_miss_{}", counter),
                    10,
                );
                let result = cache.get(black_box(&miss_key));
                black_box(result)
            } else {
                // 90% hit
                let result = cache.get(black_box(&key));
                black_box(result)
            }
        });
    });

    group.finish();
}

/// Benchmark cache stats and maintenance operations
fn bench_cache_maintenance(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolution_cache_maintenance");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    // Populate cache with many entries
    let dir = TempDir::new().unwrap();
    let cache = ResolutionCache::new(dir.path().to_path_buf())
        .with_mode(CacheMode::Normal)
        .with_ttl(Duration::from_secs(900));

    // Add 100 entries
    for i in 0..100 {
        let key = make_cache_key(dir.path().to_path_buf(), &format!("runtime_{}", i), 5);
        let value = make_resolved_graph(&format!("runtime_{}", i), 2);
        cache.set(&key, &value).unwrap();
    }

    // Benchmark stats collection
    group.bench_function("stats_100_entries", |b| {
        b.iter(|| {
            let stats = cache.stats();
            black_box(stats)
        });
    });

    // Benchmark prune (no expired entries)
    group.bench_function("prune_no_expired", |b| {
        b.iter(|| {
            let pruned = cache.prune_expired();
            black_box(pruned)
        });
    });

    group.finish();
}

/// Benchmark cache mode switching overhead
fn bench_cache_mode_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolution_cache_mode");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    let dir = TempDir::new().unwrap();
    let key = make_cache_key(dir.path().to_path_buf(), "node", 5);
    let value = make_resolved_graph("node", 2);

    // Pre-populate with Normal mode
    let normal_cache = ResolutionCache::new(dir.path().to_path_buf())
        .with_mode(CacheMode::Normal)
        .with_ttl(Duration::from_secs(900));
    normal_cache.set(&key, &value).unwrap();

    // Benchmark different modes
    let modes = [
        ("normal", CacheMode::Normal),
        ("refresh", CacheMode::Refresh),
        ("offline", CacheMode::Offline),
        ("no_cache", CacheMode::NoCache),
    ];

    for (name, mode) in modes {
        let cache = ResolutionCache::new(dir.path().to_path_buf())
            .with_mode(mode)
            .with_ttl(Duration::from_secs(900));

        group.bench_with_input(BenchmarkId::new("get", name), &key, |b, key| {
            b.iter(|| {
                let result = cache.get(black_box(key));
                black_box(result)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_read,
    bench_cache_write,
    bench_cache_key_hash,
    bench_repeated_resolution_pattern,
    bench_cache_maintenance,
    bench_cache_mode_overhead,
);

criterion_main!(benches);
