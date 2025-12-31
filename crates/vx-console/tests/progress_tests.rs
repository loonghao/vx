//! Progress tests.

use rstest::rstest;
use vx_console::{
    DownloadProgress, InstallProgress, MultiStepProgress, ProgressManager, ProgressSpinner,
};

#[rstest]
fn test_progress_manager_new() {
    let pm = ProgressManager::new();
    // Just verify it doesn't panic
    let _ = pm.multi();
}

#[rstest]
fn test_progress_manager_default() {
    let pm = ProgressManager::default();
    let _ = pm.multi();
}

#[rstest]
fn test_progress_manager_add_spinner() {
    let pm = ProgressManager::new();
    let spinner = pm.add_spinner("test");
    spinner.finish_and_clear();
}

#[rstest]
fn test_progress_manager_add_download() {
    let pm = ProgressManager::new();
    let download = pm.add_download(1000, "test");
    download.finish_and_clear();
}

#[rstest]
fn test_progress_manager_add_task() {
    let pm = ProgressManager::new();
    let task = pm.add_task(10, "test");
    task.finish_and_clear();
}

#[rstest]
fn test_progress_manager_clear_all() {
    let pm = ProgressManager::new();
    let _ = pm.add_spinner("test1");
    let _ = pm.add_spinner("test2");
    pm.clear_all();
}

#[rstest]
fn test_progress_manager_suspend() {
    let pm = ProgressManager::new();
    let result = pm.suspend(|| 42);
    assert_eq!(result, 42);
}

#[rstest]
fn test_managed_spinner_set_message() {
    let pm = ProgressManager::new();
    let spinner = pm.add_spinner("initial");
    spinner.set_message("updated");
    spinner.finish_and_clear();
}

#[rstest]
fn test_managed_spinner_finish_success() {
    let pm = ProgressManager::new();
    let spinner = pm.add_spinner("test");
    spinner.finish_success("done");
}

#[rstest]
fn test_managed_spinner_finish_error() {
    let pm = ProgressManager::new();
    let spinner = pm.add_spinner("test");
    spinner.finish_error("failed");
}

#[rstest]
fn test_managed_download_operations() {
    let pm = ProgressManager::new();
    let download = pm.add_download(1000, "test");
    download.set_length(2000);
    download.set_position(500);
    download.inc(100);
    download.set_message("downloading...");
    download.finish_with_message("done");
}

#[rstest]
fn test_managed_task_operations() {
    let pm = ProgressManager::new();
    let task = pm.add_task(10, "test");
    task.set_position(5);
    task.inc(1);
    task.set_message("processing...");
    task.finish_with_message("done");
}

#[rstest]
fn test_progress_spinner_new() {
    let spinner = ProgressSpinner::new("test");
    spinner.finish_and_clear();
}

#[rstest]
fn test_progress_spinner_new_download() {
    let spinner = ProgressSpinner::new_download("file.zip");
    spinner.finish_and_clear();
}

#[rstest]
fn test_progress_spinner_new_install() {
    let spinner = ProgressSpinner::new_install("node@20");
    spinner.finish_and_clear();
}

#[rstest]
fn test_progress_spinner_operations() {
    let spinner = ProgressSpinner::new("test");
    spinner.set_message("updated");
    spinner.finish_with_message("done");
}

#[rstest]
fn test_progress_spinner_finish_with_error() {
    let spinner = ProgressSpinner::new("test");
    spinner.finish_with_error("failed");
}

#[rstest]
fn test_download_progress_new() {
    let progress = DownloadProgress::new(1000, "test");
    progress.finish_and_clear();
}

#[rstest]
fn test_download_progress_new_unknown() {
    let progress = DownloadProgress::new_unknown("test");
    progress.finish_and_clear();
}

#[rstest]
fn test_download_progress_operations() {
    let progress = DownloadProgress::new(1000, "test");
    progress.set_length(2000);
    progress.set_position(500);
    progress.inc(100);
    progress.set_message("downloading...");
    progress.finish_with_message("done");
}

#[rstest]
fn test_multi_step_progress() {
    let mut progress = MultiStepProgress::new(vec![
        "Step 1".to_string(),
        "Step 2".to_string(),
        "Step 3".to_string(),
    ]);
    progress.next_step();
    progress.next_step();
    progress.finish("Done");
}

#[rstest]
fn test_install_progress() {
    let mut progress = InstallProgress::new(3, "Installing tools");
    progress.start_tool("node@20");
    progress.complete_tool(true);
    progress.start_tool("npm@10");
    progress.complete_tool(true);
    progress.start_tool("yarn@4");
    progress.complete_tool(false);
    progress.finish("Installed 2/3 tools");
}
