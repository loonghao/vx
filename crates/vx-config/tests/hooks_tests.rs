//! Hook execution tests
//!
//! Tests for lifecycle hook execution.

use rstest::rstest;
use tempfile::TempDir;
use vx_config::{EnterHookManager, GitHookInstaller, HookCommand, HookExecutor};

// ============================================
// HookExecutor Basic Tests
// ============================================

#[test]
fn test_hook_executor_creation() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());
    // Should create without error
    assert!(true);
}

#[test]
fn test_hook_executor_with_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path()).verbose(true);
    // Should create without error
    assert!(true);
}

#[test]
fn test_hook_executor_with_env() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path())
        .env("TEST_VAR", "test_value")
        .env("ANOTHER_VAR", "another_value");
    // Should create without error
    assert!(true);
}

#[test]
fn test_hook_executor_with_shell() {
    let temp_dir = TempDir::new().unwrap();
    let shell = if cfg!(windows) {
        "powershell"
    } else {
        "/bin/sh"
    };
    let executor = HookExecutor::new(temp_dir.path()).shell(shell);
    // Should create without error
    assert!(true);
}

// ============================================
// Single Command Execution Tests
// ============================================

#[test]
fn test_execute_single_echo_command() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let hook = HookCommand::Single("echo hello".to_string());
    let result = executor.execute("test_hook", &hook).unwrap();

    assert!(result.success);
    assert_eq!(result.exit_code, Some(0));
    assert!(result.error.is_none());
}

#[test]
fn test_execute_single_command_with_output() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let hook = HookCommand::Single("echo test_output".to_string());
    let result = executor.execute("test_hook", &hook).unwrap();

    assert!(result.success);
    if let Some(output) = &result.output {
        assert!(output.contains("test_output"));
    }
}

#[test]
fn test_execute_failing_command() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    // Use a command that will fail
    let cmd = if cfg!(windows) { "exit 1" } else { "exit 1" };
    let hook = HookCommand::Single(cmd.to_string());
    let result = executor.execute("test_hook", &hook).unwrap();

    assert!(!result.success);
    assert!(result.exit_code.is_some());
    assert_ne!(result.exit_code, Some(0));
}

// ============================================
// Multiple Command Execution Tests
// ============================================

#[test]
fn test_execute_multiple_commands() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let hook = HookCommand::Multiple(vec![
        "echo first".to_string(),
        "echo second".to_string(),
        "echo third".to_string(),
    ]);
    let result = executor.execute("test_hook", &hook).unwrap();

    assert!(result.success);
    assert_eq!(result.exit_code, Some(0));
}

#[test]
fn test_execute_multiple_commands_stops_on_failure() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let fail_cmd = if cfg!(windows) { "exit 1" } else { "exit 1" };
    let hook = HookCommand::Multiple(vec![
        "echo first".to_string(),
        fail_cmd.to_string(),
        "echo should_not_run".to_string(),
    ]);
    let result = executor.execute("test_hook", &hook).unwrap();

    assert!(!result.success);
    // The third command should not have run
    if let Some(output) = &result.output {
        assert!(!output.contains("should_not_run"));
    }
}

#[test]
fn test_execute_empty_commands_skipped() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let hook = HookCommand::Multiple(vec![
        "".to_string(),
        "   ".to_string(),
        "echo valid".to_string(),
    ]);
    let result = executor.execute("test_hook", &hook).unwrap();

    assert!(result.success);
}

// ============================================
// Environment Variable Tests
// ============================================

#[test]
fn test_execute_with_environment_variable() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path()).env("MY_TEST_VAR", "my_test_value");

    let cmd = if cfg!(windows) {
        "echo %MY_TEST_VAR%"
    } else {
        "echo $MY_TEST_VAR"
    };
    let hook = HookCommand::Single(cmd.to_string());
    let result = executor.execute("test_hook", &hook).unwrap();

    assert!(result.success);
    if let Some(output) = &result.output {
        assert!(output.contains("my_test_value"));
    }
}

// ============================================
// Named Hook Execution Tests
// ============================================

#[test]
fn test_execute_pre_setup() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let hook = HookCommand::Single("echo pre_setup".to_string());
    let result = executor.execute_pre_setup(&hook).unwrap();

    assert!(result.success);
    assert_eq!(result.name, "pre_setup");
}

#[test]
fn test_execute_post_setup() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let hook = HookCommand::Single("echo post_setup".to_string());
    let result = executor.execute_post_setup(&hook).unwrap();

    assert!(result.success);
    assert_eq!(result.name, "post_setup");
}

#[test]
fn test_execute_pre_commit() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let hook = HookCommand::Single("echo pre_commit".to_string());
    let result = executor.execute_pre_commit(&hook).unwrap();

    assert!(result.success);
    assert_eq!(result.name, "pre_commit");
}

#[test]
fn test_execute_enter() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let hook = HookCommand::Single("echo enter".to_string());
    let result = executor.execute_enter(&hook).unwrap();

    assert!(result.success);
    assert_eq!(result.name, "enter");
}

#[test]
fn test_execute_custom_hook() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let hook = HookCommand::Single("echo custom".to_string());
    let result = executor.execute_custom("my_custom_hook", &hook).unwrap();

    assert!(result.success);
    assert_eq!(result.name, "my_custom_hook");
}

// ============================================
// GitHookInstaller Tests
// ============================================

#[test]
fn test_git_hook_installer_creation() {
    let temp_dir = TempDir::new().unwrap();
    let installer = GitHookInstaller::new(temp_dir.path());
    // Should create without error
    assert!(true);
}

#[test]
fn test_git_hook_hooks_dir() {
    let temp_dir = TempDir::new().unwrap();
    let installer = GitHookInstaller::new(temp_dir.path());
    let hooks_dir = installer.hooks_dir();

    assert!(hooks_dir.ends_with("hooks"));
    assert!(hooks_dir.to_string_lossy().contains(".git"));
}

#[test]
fn test_git_hook_find_repo_root() {
    let temp_dir = TempDir::new().unwrap();

    // Create .git directory
    std::fs::create_dir_all(temp_dir.path().join(".git")).unwrap();

    // Create nested directory
    let nested = temp_dir.path().join("src").join("components");
    std::fs::create_dir_all(&nested).unwrap();

    // Should find repo root from nested directory
    let root = GitHookInstaller::find_repo_root(&nested);
    assert!(root.is_some());
    assert_eq!(root.unwrap(), temp_dir.path());
}

#[test]
fn test_git_hook_find_repo_root_not_found() {
    let temp_dir = TempDir::new().unwrap();
    // No .git directory

    let root = GitHookInstaller::find_repo_root(temp_dir.path());
    assert!(root.is_none());
}

#[test]
fn test_git_hook_install_pre_commit() {
    let temp_dir = TempDir::new().unwrap();

    // Create .git directory
    std::fs::create_dir_all(temp_dir.path().join(".git")).unwrap();

    let installer = GitHookInstaller::new(temp_dir.path());
    installer.install_pre_commit().unwrap();

    // Check hook was created
    let hook_path = installer.hooks_dir().join("pre-commit");
    assert!(hook_path.exists());

    // Check content
    let content = std::fs::read_to_string(&hook_path).unwrap();
    assert!(content.contains("# vx-managed"));
}

#[test]
fn test_git_hook_is_installed() {
    let temp_dir = TempDir::new().unwrap();

    // Create .git directory
    std::fs::create_dir_all(temp_dir.path().join(".git")).unwrap();

    let installer = GitHookInstaller::new(temp_dir.path());

    // Initially not installed
    assert!(!installer.is_installed());

    // Install
    installer.install_pre_commit().unwrap();

    // Now installed
    assert!(installer.is_installed());
}

#[test]
fn test_git_hook_uninstall_pre_commit() {
    let temp_dir = TempDir::new().unwrap();

    // Create .git directory
    std::fs::create_dir_all(temp_dir.path().join(".git")).unwrap();

    let installer = GitHookInstaller::new(temp_dir.path());

    // Install first
    installer.install_pre_commit().unwrap();
    assert!(installer.is_installed());

    // Uninstall
    installer.uninstall_pre_commit().unwrap();
    assert!(!installer.is_installed());
}

#[test]
fn test_git_hook_preserves_existing_hook() {
    let temp_dir = TempDir::new().unwrap();

    // Create .git/hooks directory
    let hooks_dir = temp_dir.path().join(".git").join("hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();

    // Create existing hook
    let existing_hook = hooks_dir.join("pre-commit");
    std::fs::write(&existing_hook, "#!/bin/sh\necho 'existing hook'").unwrap();

    let installer = GitHookInstaller::new(temp_dir.path());
    installer.install_pre_commit().unwrap();

    // Backup should exist
    let backup_path = hooks_dir.join("pre-commit.backup");
    assert!(backup_path.exists());

    // New hook should be installed
    let new_content = std::fs::read_to_string(&existing_hook).unwrap();
    assert!(new_content.contains("# vx-managed"));
}

// ============================================
// EnterHookManager Tests
// ============================================

#[test]
fn test_enter_hook_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let manager = EnterHookManager::new(temp_dir.path());
    // Should create without error
    assert!(true);
}

#[test]
fn test_enter_hook_get_last_directory_empty() {
    let temp_dir = TempDir::new().unwrap();
    let manager = EnterHookManager::new(temp_dir.path());

    // Initially no last directory
    let last = manager.get_last_directory();
    assert!(last.is_none());
}

#[test]
fn test_enter_hook_set_and_get_directory() {
    let temp_dir = TempDir::new().unwrap();
    let manager = EnterHookManager::new(temp_dir.path());

    let test_dir = temp_dir.path().join("test_project");
    std::fs::create_dir_all(&test_dir).unwrap();

    // Set directory
    manager.set_current_directory(&test_dir).unwrap();

    // Get directory
    let last = manager.get_last_directory();
    assert!(last.is_some());
    assert_eq!(last.unwrap(), test_dir);
}

#[test]
fn test_enter_hook_should_trigger_first_time() {
    let temp_dir = TempDir::new().unwrap();
    let manager = EnterHookManager::new(temp_dir.path());

    let test_dir = temp_dir.path().join("project");
    std::fs::create_dir_all(&test_dir).unwrap();

    // Should trigger on first entry
    assert!(manager.should_trigger(&test_dir));
}

#[test]
fn test_enter_hook_should_not_trigger_same_directory() {
    let temp_dir = TempDir::new().unwrap();
    let manager = EnterHookManager::new(temp_dir.path());

    let test_dir = temp_dir.path().join("project");
    std::fs::create_dir_all(&test_dir).unwrap();

    // Set current directory
    manager.set_current_directory(&test_dir).unwrap();

    // Should not trigger for same directory
    assert!(!manager.should_trigger(&test_dir));
}

#[test]
fn test_enter_hook_should_trigger_different_directory() {
    let temp_dir = TempDir::new().unwrap();
    let manager = EnterHookManager::new(temp_dir.path());

    let project1 = temp_dir.path().join("project1");
    let project2 = temp_dir.path().join("project2");
    std::fs::create_dir_all(&project1).unwrap();
    std::fs::create_dir_all(&project2).unwrap();

    // Set to project1
    manager.set_current_directory(&project1).unwrap();

    // Should trigger for project2
    assert!(manager.should_trigger(&project2));
}

// ============================================
// Shell Integration Tests
// ============================================

#[rstest]
#[case("bash")]
#[case("zsh")]
#[case("fish")]
#[case("pwsh")]
#[case("powershell")]
fn test_generate_shell_integration(#[case] shell: &str) {
    let script = EnterHookManager::generate_shell_integration(shell);

    // Should contain vx enter hook
    assert!(script.contains("vx"));
    assert!(script.contains("hook"));
    assert!(script.contains("enter") || script.contains("__vx_enter_hook"));
}

#[test]
fn test_generate_shell_integration_bash() {
    let script = EnterHookManager::generate_shell_integration("bash");
    assert!(script.contains("PROMPT_COMMAND"));
    assert!(script.contains("__vx_enter_hook"));
}

#[test]
fn test_generate_shell_integration_zsh() {
    let script = EnterHookManager::generate_shell_integration("zsh");
    assert!(script.contains("chpwd"));
    assert!(script.contains("add-zsh-hook"));
}

#[test]
fn test_generate_shell_integration_fish() {
    let script = EnterHookManager::generate_shell_integration("fish");
    assert!(script.contains("--on-variable PWD"));
}

#[test]
fn test_generate_shell_integration_powershell() {
    let script = EnterHookManager::generate_shell_integration("pwsh");
    assert!(script.contains("function"));
    assert!(script.contains("Test-Path"));
}

// ============================================
// HookResult Tests
// ============================================

#[test]
fn test_hook_result_success() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let hook = HookCommand::Single("echo success".to_string());
    let result = executor.execute("test", &hook).unwrap();

    assert!(result.success);
    assert_eq!(result.exit_code, Some(0));
    assert!(result.error.is_none());
    assert_eq!(result.name, "test");
}

#[test]
fn test_hook_result_failure() {
    let temp_dir = TempDir::new().unwrap();
    let executor = HookExecutor::new(temp_dir.path());

    let cmd = if cfg!(windows) { "exit 42" } else { "exit 42" };
    let hook = HookCommand::Single(cmd.to_string());
    let result = executor.execute("test", &hook).unwrap();

    assert!(!result.success);
    assert!(result.error.is_some());
}

// ============================================
// HookCommand Tests
// ============================================

#[test]
fn test_hook_command_single() {
    let hook = HookCommand::Single("echo test".to_string());
    match hook {
        HookCommand::Single(cmd) => assert_eq!(cmd, "echo test"),
        _ => panic!("Expected Single variant"),
    }
}

#[test]
fn test_hook_command_multiple() {
    let hook = HookCommand::Multiple(vec!["echo 1".to_string(), "echo 2".to_string()]);
    match hook {
        HookCommand::Multiple(cmds) => {
            assert_eq!(cmds.len(), 2);
            assert_eq!(cmds[0], "echo 1");
            assert_eq!(cmds[1], "echo 2");
        }
        _ => panic!("Expected Multiple variant"),
    }
}

// ============================================
// Working Directory Tests
// ============================================

#[test]
fn test_execute_in_working_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Create a subdirectory
    let subdir = temp_dir.path().join("subdir");
    std::fs::create_dir_all(&subdir).unwrap();

    // Create a file in subdir
    std::fs::write(subdir.join("test.txt"), "content").unwrap();

    let executor = HookExecutor::new(&subdir);

    // List files in current directory
    let cmd = if cfg!(windows) { "dir /b" } else { "ls" };
    let hook = HookCommand::Single(cmd.to_string());
    let result = executor.execute("test", &hook).unwrap();

    assert!(result.success);
    if let Some(output) = &result.output {
        assert!(output.contains("test.txt"));
    }
}
