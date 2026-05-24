use rstest::rstest;

#[cfg(unix)]
use std::collections::HashMap;
#[cfg(unix)]
use vx_resolver::{
    ExecuteStage, ExecutionConfig, ExecutionPlan, PlannedRuntime, PreparedExecution, Stage,
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
#[rstest]
#[tokio::test]
async fn test_execute_stage_repairs_vx_store_execute_permissions() {
    let vx_home = tempfile::TempDir::new().expect("temp vx home");
    unsafe {
        std::env::set_var("VX_HOME", vx_home.path());
    }

    let executable = vx_home.path().join("store/fake/1.0.0/bin/fake");
    std::fs::create_dir_all(executable.parent().expect("bin dir")).expect("create bin dir");
    std::fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("write script");
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o644))
        .expect("set non-executable mode");

    let plan = ExecutionPlan::new(
        PlannedRuntime::installed("fake", "1.0.0".to_string(), executable.clone()),
        ExecutionConfig::default(),
    );

    let prepared = PreparedExecution {
        executable: executable.clone(),
        command_prefix: Vec::new(),
        args: Vec::new(),
        env: HashMap::new(),
        inherit_vx_path: false,
        vx_tools_path: None,
        working_dir: None,
        plan,
        output_filter: None,
    };

    let exit_code = ExecuteStage::new()
        .execute(prepared)
        .await
        .expect("execution should repair permissions and run");

    assert_eq!(exit_code, 0);

    let repaired_mode = std::fs::metadata(&executable)
        .expect("metadata")
        .permissions()
        .mode();
    assert_ne!(repaired_mode & 0o111, 0);
}

#[cfg(not(unix))]
#[rstest]
fn test_execute_permission_repair_is_unix_only() {
    // Permission-bit repair is only meaningful on Unix.
}
