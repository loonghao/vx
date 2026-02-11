use rstest::rstest;
use vx_runtime::{Arch, Os, Platform, Provider, Runtime};

#[test]
fn test_provider_name() {
    let provider = vx_provider_skills::SkillsProvider::new();
    assert_eq!(provider.name(), "skills");
}

#[test]
fn test_provider_description() {
    let provider = vx_provider_skills::SkillsProvider::new();
    assert!(!provider.description().is_empty());
}

#[test]
fn test_provider_supports_skills() {
    let provider = vx_provider_skills::SkillsProvider::new();
    assert!(provider.supports("skills"));
    assert!(!provider.supports("unknown"));
}

#[test]
fn test_provider_get_runtime() {
    let provider = vx_provider_skills::SkillsProvider::new();
    assert!(provider.get_runtime("skills").is_some());
    assert!(provider.get_runtime("unknown").is_none());
}

#[test]
fn test_provider_runtimes() {
    let provider = vx_provider_skills::SkillsProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "skills");
}

#[test]
fn test_runtime_name() {
    let runtime = vx_provider_skills::SkillsRuntime::new();
    assert_eq!(runtime.name(), "skills");
}

#[test]
fn test_runtime_description() {
    let runtime = vx_provider_skills::SkillsRuntime::new();
    assert!(runtime.description().contains("Skills"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = vx_provider_skills::SkillsRuntime::new();
    assert_eq!(runtime.ecosystem(), vx_runtime::Ecosystem::NodeJs);
}

#[test]
fn test_runtime_metadata() {
    let runtime = vx_provider_skills::SkillsRuntime::new();
    let meta = runtime.metadata();
    assert_eq!(
        meta.get("homepage").unwrap(),
        "https://github.com/vercel-labs/skills"
    );
    assert_eq!(meta.get("category").unwrap(), "ai");
    assert_eq!(meta.get("install_method").unwrap(), "npm");
    assert_eq!(meta.get("npm_package").unwrap(), "skills");
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, "bin/skills.cmd")]
#[case(Os::Linux, Arch::X86_64, "bin/skills")]
#[case(Os::MacOS, Arch::Aarch64, "bin/skills")]
fn test_executable_relative_path(
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] expected: &str,
) {
    let runtime = vx_provider_skills::SkillsRuntime::new();
    let platform = Platform { os, arch, ..Default::default() };
    assert_eq!(runtime.executable_relative_path("1.0.0", &platform), expected);
}

#[test]
fn test_create_provider_factory() {
    let provider = vx_provider_skills::create_provider();
    assert_eq!(provider.name(), "skills");
}

#[test]
fn test_package_runtime_install_method() {
    use vx_runtime::PackageRuntime;
    let runtime = vx_provider_skills::SkillsRuntime::new();
    let method = runtime.install_method();
    match method {
        vx_runtime::InstallMethod::NpmPackage { package_name, .. } => {
            assert_eq!(package_name, "skills");
        }
        _ => panic!("Expected NpmPackage install method"),
    }
}

#[test]
fn test_package_runtime_required_runtime() {
    use vx_runtime::PackageRuntime;
    let runtime = vx_provider_skills::SkillsRuntime::new();
    assert_eq!(runtime.required_runtime(), "node");
    assert_eq!(runtime.required_runtime_version(), Some(">=18.0.0"));
}
