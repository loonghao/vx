//! Task execution tests.

use rstest::rstest;
use std::time::Duration;
use vx_console::{format_bytes, format_duration, format_speed, TaskResult, TimedTask};

#[rstest]
#[case(Duration::from_micros(500), "500µs")]
#[case(Duration::from_millis(500), "500ms")]
#[case(Duration::from_secs_f64(2.5), "2.5s")]
#[case(Duration::from_secs(125), "2m 5s")]
#[case(Duration::from_secs(3725), "1h 2m")]
fn test_format_duration(#[case] duration: Duration, #[case] expected: &str) {
    assert_eq!(format_duration(duration), expected);
}

#[rstest]
#[case(500, "500 B")]
#[case(1536, "1.5 KB")]
#[case(1_572_864, "1.5 MB")]
fn test_format_bytes(#[case] bytes: u64, #[case] expected: &str) {
    assert_eq!(format_bytes(bytes), expected);
}

#[rstest]
#[case(500.0, "500 B/s")]
#[case(1536.0, "1.5 KB/s")]
#[case(1_572_864.0, "1.5 MB/s")]
fn test_format_speed(#[case] speed: f64, #[case] expected: &str) {
    assert_eq!(format_speed(speed), expected);
}

#[rstest]
fn test_task_result_new() {
    let result = TaskResult::new(42, Duration::from_secs(2));
    assert_eq!(result.value, 42);
    assert_eq!(result.duration, Duration::from_secs(2));
}

#[rstest]
fn test_task_result_duration_string() {
    let result = TaskResult::new((), Duration::from_secs_f64(1.5));
    assert_eq!(result.duration_string(), "1.5s");
}

#[rstest]
fn test_task_result_map() {
    let result = TaskResult::new(10, Duration::from_secs(1));
    let mapped = result.map(|v| v * 2);
    assert_eq!(mapped.value, 20);
    assert_eq!(mapped.duration, Duration::from_secs(1));
}

#[rstest]
fn test_timed_task_new() {
    let task = TimedTask::new("test task");
    assert_eq!(task.name(), "test task");
    assert_eq!(task.elapsed(), Duration::ZERO);
}

#[rstest]
fn test_timed_task_start() {
    let mut task = TimedTask::new("test");
    task.start();
    std::thread::sleep(Duration::from_millis(10));
    assert!(task.elapsed() >= Duration::from_millis(10));
}

#[rstest]
fn test_timed_task_execute_success() {
    let result: Result<TaskResult<i32>, &str> = TimedTask::execute("test", || Ok(42));
    assert!(result.is_ok());
    let task_result = result.unwrap();
    assert_eq!(task_result.value, 42);
}

#[rstest]
fn test_timed_task_execute_error() {
    let result: Result<TaskResult<i32>, &str> = TimedTask::execute("test", || Err("error"));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "error");
}

#[tokio::test]
async fn test_timed_task_execute_async_success() {
    let result: Result<TaskResult<i32>, &str> =
        TimedTask::execute_async("test", || async { Ok(42) }).await;
    assert!(result.is_ok());
    let task_result = result.unwrap();
    assert_eq!(task_result.value, 42);
}

#[tokio::test]
async fn test_timed_task_execute_async_error() {
    let result: Result<TaskResult<i32>, &str> =
        TimedTask::execute_async("test", || async { Err("error") }).await;
    assert!(result.is_err());
}
