use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::StarMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryConfig {
    pub providers_dir: PathBuf,
    pub chunk_size: usize,
    pub skip_always: BTreeSet<String>,
    pub runtime_filter: BTreeSet<String>,
    pub provider_filter: BTreeSet<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiscoveryPlatform {
    pub runtimes: Vec<String>,
    pub matrix: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiscoveryResult {
    pub total_runtimes: usize,
    pub testable_runtimes: usize,
    pub linux: DiscoveryPlatform,
    pub macos: DiscoveryPlatform,
    pub windows: DiscoveryPlatform,
}

impl DiscoveryConfig {
    pub fn new(providers_dir: impl Into<PathBuf>, chunk_size: usize) -> Self {
        Self {
            providers_dir: providers_dir.into(),
            chunk_size: chunk_size.max(1),
            skip_always: BTreeSet::new(),
            runtime_filter: BTreeSet::new(),
            provider_filter: BTreeSet::new(),
        }
    }
}

impl DiscoveryResult {
    pub fn to_key_value_lines(&self) -> Vec<String> {
        vec![
            format!("total-runtimes={}", self.total_runtimes),
            format!("testable-runtimes={}", self.testable_runtimes),
            format!("linux-count={}", self.linux.runtimes.len()),
            format!("macos-count={}", self.macos.runtimes.len()),
            format!("windows-count={}", self.windows.runtimes.len()),
            format!("linux-runtimes={}", self.linux.runtimes.join(" ")),
            format!("macos-runtimes={}", self.macos.runtimes.join(" ")),
            format!("windows-runtimes={}", self.windows.runtimes.join(" ")),
            format!("matrix-linux={}", json_array(&self.linux.matrix)),
            format!("matrix-macos={}", json_array(&self.macos.matrix)),
            format!("matrix-windows={}", json_array(&self.windows.matrix)),
            format!("has-linux={}", bool_string(!self.linux.runtimes.is_empty())),
            format!("has-macos={}", bool_string(!self.macos.runtimes.is_empty())),
            format!(
                "has-windows={}",
                bool_string(!self.windows.runtimes.is_empty())
            ),
        ]
    }
}

pub fn discover_providers(config: &DiscoveryConfig) -> io::Result<DiscoveryResult> {
    let runtime_platforms = collect_runtime_platforms(config)?;

    let linux = select_platform("linux", &runtime_platforms, config);
    let macos = select_platform("macos", &runtime_platforms, config);
    let windows = select_platform("windows", &runtime_platforms, config);

    let testable_runtimes = linux
        .runtimes
        .iter()
        .chain(macos.runtimes.iter())
        .chain(windows.runtimes.iter())
        .cloned()
        .collect::<BTreeSet<_>>()
        .len();

    Ok(DiscoveryResult {
        total_runtimes: runtime_platforms.len(),
        testable_runtimes,
        linux,
        macos,
        windows,
    })
}

fn collect_runtime_platforms(
    config: &DiscoveryConfig,
) -> io::Result<BTreeMap<String, Option<BTreeSet<String>>>> {
    let mut runtime_platforms = BTreeMap::new();

    for entry in fs::read_dir(&config.providers_dir)? {
        let entry = entry?;
        let provider_dir_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        let provider_path = entry.path().join("provider.star");
        if !provider_path.is_file() {
            continue;
        }

        let source = fs::read_to_string(&provider_path)?;
        let metadata = StarMetadata::parse(&source);

        // Skip package_alias providers (RFC 0033) — they route `vx <name>` to
        // `vx <ecosystem>:<package>` and are tested via ecosystem package managers,
        // not direct install.  E.g., openclaw, vite, turbo, nx, meson, etc.
        if metadata.package_alias.is_some() {
            continue;
        }

        let provider_name = metadata
            .name
            .clone()
            .unwrap_or_else(|| provider_dir_name.clone());
        let provider_filter_key = provider_name.trim().to_ascii_lowercase();
        if !config.provider_filter.is_empty()
            && !config.provider_filter.contains(&provider_dir_name)
            && !config.provider_filter.contains(&provider_filter_key)
        {
            continue;
        }

        let provider_platforms = normalize_platforms(metadata.platforms.as_deref());

        if metadata.runtimes.is_empty() {
            merge_runtime_platforms(&mut runtime_platforms, &provider_name, provider_platforms);
            continue;
        }

        for runtime in metadata.runtimes {
            // Skip bundled runtimes — they are tested as part of their parent runtime
            // e.g., npm/npx are tested when node is tested, ffplay when ffmpeg is tested
            if runtime.bundled_with.is_some() {
                continue;
            }

            let runtime_name = runtime.name.unwrap_or_else(|| provider_name.clone());
            let runtime_platforms_for_entry = normalize_platforms(Some(&runtime.platform_os))
                .or_else(|| provider_platforms.clone());
            merge_runtime_platforms(
                &mut runtime_platforms,
                &runtime_name,
                runtime_platforms_for_entry,
            );
        }
    }

    Ok(runtime_platforms)
}

fn select_platform(
    platform: &str,
    runtime_platforms: &BTreeMap<String, Option<BTreeSet<String>>>,
    config: &DiscoveryConfig,
) -> DiscoveryPlatform {
    let runtimes = runtime_platforms
        .iter()
        .filter_map(|(runtime, constraints)| {
            if config.skip_always.contains(runtime) {
                return None;
            }

            if !config.runtime_filter.is_empty() && !config.runtime_filter.contains(runtime) {
                return None;
            }

            if let Some(allowed) = constraints
                && !allowed.contains(platform)
            {
                return None;
            }

            Some(runtime.clone())
        })
        .collect::<Vec<_>>();

    DiscoveryPlatform {
        matrix: chunk_runtimes(&runtimes, config.chunk_size),
        runtimes,
    }
}

fn merge_runtime_platforms(
    runtime_platforms: &mut BTreeMap<String, Option<BTreeSet<String>>>,
    runtime_name: &str,
    new_value: Option<BTreeSet<String>>,
) {
    use std::collections::btree_map::Entry;

    match runtime_platforms.entry(runtime_name.to_string()) {
        Entry::Vacant(entry) => {
            entry.insert(new_value);
        }
        Entry::Occupied(mut entry) => {
            let existing = entry.get_mut();
            match new_value {
                None => *existing = None,
                Some(new_platforms) => match existing {
                    None => {}
                    Some(existing_platforms) => existing_platforms.extend(new_platforms),
                },
            }
        }
    }
}

fn normalize_platforms(values: Option<&[String]>) -> Option<BTreeSet<String>> {
    let platforms = values
        .into_iter()
        .flatten()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();

    if platforms.is_empty() {
        None
    } else {
        Some(platforms)
    }
}

fn chunk_runtimes(runtimes: &[String], chunk_size: usize) -> Vec<String> {
    runtimes
        .chunks(chunk_size.max(1))
        .map(|chunk| chunk.join(","))
        .collect()
}

fn json_array(values: &[String]) -> String {
    let escaped = values
        .iter()
        .map(|value| format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect::<Vec<_>>();
    format!("[{}]", escaped.join(","))
}

fn bool_string(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
