pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_hook_timeout() -> u64 {
    30000
}

pub(crate) fn default_exit_codes() -> Vec<i32> {
    vec![0]
}

pub(crate) fn default_health_timeout() -> u64 {
    5000
}

pub(crate) fn default_check_on() -> Vec<String> {
    vec!["install".to_string()]
}

pub(crate) fn default_probe_timeout() -> u64 {
    3000
}

pub(crate) fn default_versions_ttl() -> u64 {
    3600
}

pub(crate) fn default_retention_days() -> u32 {
    30
}

pub(crate) fn default_test_timeout() -> u64 {
    30000
}

/// Default download timeout: 5 minutes (300,000 ms)
pub(crate) fn default_download_timeout() -> u64 {
    300_000
}

/// Default execution timeout: 30 seconds (30,000 ms)
pub(crate) fn default_execution_timeout() -> u64 {
    30_000
}

/// Default max retries for downloads
pub(crate) fn default_max_retries() -> u32 {
    3
}
