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
