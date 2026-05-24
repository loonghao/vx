use std::time::{Duration, Instant};

use vx_runtime::{RuntimeTester, TestCheckType, TestCommand, TestConfig};

#[test]
fn runtime_tester_honors_per_command_timeout() {
    let command = if cfg!(windows) {
        "powershell -NoProfile -Command Start-Sleep -Seconds 5"
    } else {
        "sleep 5"
    };

    let tester = RuntimeTester::new("vx-runtime-timeout-test")
        .with_executable(std::env::current_exe().expect("current test executable should exist"))
        .with_config(TestConfig {
            functional_commands: vec![TestCommand {
                command: command.to_string(),
                check_type: TestCheckType::Command,
                expect_success: true,
                expected_output: None,
                expected_exit_code: None,
                name: Some("slow_command".to_string()),
                timeout_ms: Some(200),
            }],
            timeout_ms: 10_000,
            ..Default::default()
        });

    let start = Instant::now();
    let result = tester.run_all();

    assert!(
        start.elapsed() < Duration::from_secs(4),
        "per-command timeout should stop the slow command early"
    );
    assert_eq!(result.test_cases.len(), 1);

    let test_case = &result.test_cases[0];
    assert!(!test_case.passed);
    assert!(
        test_case
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("timed out"),
        "timeout failure should be reported clearly: {test_case:?}"
    );
}
