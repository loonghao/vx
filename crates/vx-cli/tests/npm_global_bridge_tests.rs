use rstest::rstest;
use vx_cli::npm_global_bridge::{parse_global_install_bridge_args, parse_npm_global_install_args};

fn to_args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

#[rstest]
#[case(&["install", "-g", "@tencent-ai/codebuddy-code"], &["@tencent-ai/codebuddy-code"], &[], false, false)]
#[case(&["i", "--global", "typescript", "eslint"], &["typescript", "eslint"], &[], false, false)]
#[case(&["-g", "install", "@scope/pkg@1.2.3"], &["@scope/pkg@1.2.3"], &[], false, false)]
#[case(&["install", "--global", "--registry", "https://registry.npmmirror.com", "@scope/pkg"], &["@scope/pkg"], &["--registry", "https://registry.npmmirror.com"], false, false)]
#[case(&["add", "--global", "--force", "--verbose", "codebuddy"], &["codebuddy"], &[], true, true)]
fn test_parse_valid_npm_global_install(
    #[case] raw: &[&str],
    #[case] expected_packages: &[&str],
    #[case] expected_extra: &[&str],
    #[case] expected_force: bool,
    #[case] expected_verbose: bool,
) {
    let parsed = parse_npm_global_install_args(&to_args(raw))
        .expect("expected npm global install args to be parsed");

    assert_eq!(
        parsed.packages,
        expected_packages
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        parsed.extra_args,
        expected_extra
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(parsed.force, expected_force);
    assert_eq!(parsed.verbose, expected_verbose);
}

#[rstest]
#[case(&["install", "typescript"])]
#[case(&["run", "build"])]
#[case(&["install", "-g"])]
#[case(&[])]
fn test_parse_non_global_or_incomplete_install(#[case] raw: &[&str]) {
    let parsed = parse_npm_global_install_args(&to_args(raw));
    assert!(parsed.is_none());
}

#[rstest]
#[case("pnpm", &["add", "-g", "eslint"], "pnpm", &["eslint"])]
#[case("yarn", &["global", "add", "eslint"], "yarn", &["eslint"])]
#[case("pip", &["install", "--user", "ruff"], "pip", &["ruff"])]
#[case("cargo", &["install", "ripgrep", "--version", "14.1.1"], "cargo", &["ripgrep@14.1.1"])]
#[case("go", &["install", "golang.org/x/tools/gopls@latest"], "go", &["golang.org/x/tools/gopls@latest"])]
#[case("gem", &["install", "bundler", "--version", "2.5.0"], "gem", &["bundler@2.5.0"])]
fn test_parse_cross_ecosystem_global_install(
    #[case] runtime: &str,
    #[case] raw: &[&str],
    #[case] expected_ecosystem: &str,
    #[case] expected_packages: &[&str],
) {
    let parsed = parse_global_install_bridge_args(runtime, &to_args(raw))
        .expect("expected cross-ecosystem global install args to be parsed");

    assert_eq!(parsed.ecosystem, expected_ecosystem);
    assert_eq!(
        parsed.packages,
        expected_packages
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>()
    );
}

#[rstest]
#[case("pip", &["install", "ruff"])]
#[case("cargo", &["build"])]
#[case("yarn", &["add", "eslint"])]
fn test_parse_cross_ecosystem_invalid_patterns(#[case] runtime: &str, #[case] raw: &[&str]) {
    let parsed = parse_global_install_bridge_args(runtime, &to_args(raw));
    assert!(parsed.is_none());
}
