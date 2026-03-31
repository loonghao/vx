use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;

use vx_star_metadata::{DiscoveryConfig, discover_providers};

fn main() {
    let mut providers_dir = PathBuf::from("crates/vx-providers");
    let mut chunk_size = 8usize;
    let mut skip_always = BTreeSet::new();
    let mut runtime_filter = BTreeSet::new();
    let mut provider_filter = BTreeSet::new();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--providers-dir" => providers_dir = PathBuf::from(next_arg(&mut args, &arg)),
            "--chunk-size" => {
                let value = next_arg(&mut args, &arg);
                chunk_size = value.parse::<usize>().unwrap_or(8).max(1);
            }
            "--skip" => skip_always = parse_csv(&next_arg(&mut args, &arg)),
            "--runtimes" => runtime_filter = parse_csv(&next_arg(&mut args, &arg)),
            "--providers" => provider_filter = parse_csv(&next_arg(&mut args, &arg)),
            other => panic!("Unknown option: {other}"),
        }
    }

    let config = DiscoveryConfig {
        providers_dir,
        chunk_size,
        skip_always,
        runtime_filter,
        provider_filter,
    };

    let result = discover_providers(&config).expect("provider discovery failed");
    for line in result.to_key_value_lines() {
        println!("{line}");
    }
}

fn next_arg(args: &mut impl Iterator<Item = String>, option: &str) -> String {
    args.next()
        .unwrap_or_else(|| panic!("Missing value for {option}"))
}

fn parse_csv(input: &str) -> BTreeSet<String> {
    input
        .split(',')
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}
