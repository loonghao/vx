//! Tests for hook extension support

use rstest::rstest;
use vx_extension::{HookContext, HookEvent};

#[rstest]
#[case(HookEvent::PreInstall, "pre-install")]
#[case(HookEvent::PostInstall, "post-install")]
#[case(HookEvent::PreUninstall, "pre-uninstall")]
#[case(HookEvent::PostUninstall, "post-uninstall")]
#[case(HookEvent::PreRun, "pre-run")]
#[case(HookEvent::PostRun, "post-run")]
#[case(HookEvent::EnterProject, "enter-project")]
#[case(HookEvent::LeaveProject, "leave-project")]
fn test_hook_event_config_key(#[case] event: HookEvent, #[case] expected: &str) {
    assert_eq!(event.config_key(), expected);
}

#[rstest]
#[case("pre-install", Some(HookEvent::PreInstall))]
#[case("post-install", Some(HookEvent::PostInstall))]
#[case("pre-uninstall", Some(HookEvent::PreUninstall))]
#[case("post-uninstall", Some(HookEvent::PostUninstall))]
#[case("pre-run", Some(HookEvent::PreRun))]
#[case("post-run", Some(HookEvent::PostRun))]
#[case("enter-project", Some(HookEvent::EnterProject))]
#[case("leave-project", Some(HookEvent::LeaveProject))]
#[case("invalid", None)]
#[case("preinstall", None)]
fn test_hook_event_from_config_key(#[case] key: &str, #[case] expected: Option<HookEvent>) {
    assert_eq!(HookEvent::from_config_key(key), expected);
}

#[test]
fn test_hook_context_default() {
    let context = HookContext::new();
    assert!(context.runtime.is_none());
    assert!(context.version.is_none());
    assert!(context.command.is_none());
    assert!(context.args.is_empty());
    assert!(context.project_dir.is_none());
    assert!(context.env.is_empty());
}

#[test]
fn test_hook_context_builder() {
    let context = HookContext::new()
        .with_runtime("node")
        .with_version("18.0.0")
        .with_command("install")
        .with_args(vec!["express".to_string(), "lodash".to_string()])
        .with_project_dir("/path/to/project")
        .with_env("CUSTOM_VAR", "custom_value");

    assert_eq!(context.runtime, Some("node".to_string()));
    assert_eq!(context.version, Some("18.0.0".to_string()));
    assert_eq!(context.command, Some("install".to_string()));
    assert_eq!(context.args, vec!["express", "lodash"]);
    assert_eq!(context.project_dir, Some("/path/to/project".to_string()));
    assert_eq!(
        context.env.get("CUSTOM_VAR"),
        Some(&"custom_value".to_string())
    );
}

#[test]
fn test_hook_context_to_env_vars() {
    let context = HookContext::new()
        .with_runtime("python")
        .with_version("3.11")
        .with_command("run")
        .with_args(vec!["script.py".to_string()])
        .with_project_dir("/home/user/project");

    let env = context.to_env_vars();

    assert_eq!(env.get("VX_HOOK_RUNTIME"), Some(&"python".to_string()));
    assert_eq!(env.get("VX_HOOK_VERSION"), Some(&"3.11".to_string()));
    assert_eq!(env.get("VX_HOOK_COMMAND"), Some(&"run".to_string()));
    assert_eq!(env.get("VX_HOOK_ARGS"), Some(&"script.py".to_string()));
    assert_eq!(
        env.get("VX_HOOK_PROJECT_DIR"),
        Some(&"/home/user/project".to_string())
    );
}

#[test]
fn test_hook_context_to_env_vars_with_multiple_args() {
    let context = HookContext::new().with_args(vec![
        "arg1".to_string(),
        "arg2".to_string(),
        "arg3".to_string(),
    ]);

    let env = context.to_env_vars();
    assert_eq!(env.get("VX_HOOK_ARGS"), Some(&"arg1 arg2 arg3".to_string()));
}

#[test]
fn test_hook_context_to_env_vars_empty() {
    let context = HookContext::new();
    let env = context.to_env_vars();

    // Empty context should not have these keys
    assert!(!env.contains_key("VX_HOOK_RUNTIME"));
    assert!(!env.contains_key("VX_HOOK_VERSION"));
    assert!(!env.contains_key("VX_HOOK_COMMAND"));
    assert!(!env.contains_key("VX_HOOK_ARGS"));
    assert!(!env.contains_key("VX_HOOK_PROJECT_DIR"));
}

#[test]
fn test_hook_context_custom_env_preserved() {
    let context = HookContext::new()
        .with_env("MY_VAR", "my_value")
        .with_env("ANOTHER_VAR", "another_value")
        .with_runtime("go");

    let env = context.to_env_vars();

    assert_eq!(env.get("MY_VAR"), Some(&"my_value".to_string()));
    assert_eq!(env.get("ANOTHER_VAR"), Some(&"another_value".to_string()));
    assert_eq!(env.get("VX_HOOK_RUNTIME"), Some(&"go".to_string()));
}

#[test]
fn test_hook_event_display() {
    assert_eq!(format!("{}", HookEvent::PreInstall), "pre-install");
    assert_eq!(format!("{}", HookEvent::PostRun), "post-run");
    assert_eq!(format!("{}", HookEvent::EnterProject), "enter-project");
}
