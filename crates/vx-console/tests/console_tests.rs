//! Console tests.

use rstest::rstest;
use vx_console::{ColorChoice, Console, ConsoleBuilder, Theme, Verbosity};

#[rstest]
fn test_console_new() {
    let console = Console::new();
    assert_eq!(console.shell().verbosity(), Verbosity::Normal);
}

#[rstest]
fn test_console_default() {
    let console = Console::default();
    assert_eq!(console.shell().verbosity(), Verbosity::Normal);
}

#[rstest]
fn test_console_builder() {
    let console = Console::builder()
        .verbosity(Verbosity::Verbose)
        .color_choice(ColorChoice::Never)
        .build();

    assert_eq!(console.shell().verbosity(), Verbosity::Verbose);
    assert_eq!(console.shell().color_choice(), ColorChoice::Never);
}

#[rstest]
fn test_console_builder_with_theme() {
    let theme = Theme::minimal();
    let console = Console::builder().theme(theme).build();

    assert_eq!(console.shell().theme().success_prefix(), "[OK]");
}

#[rstest]
fn test_console_set_verbosity() {
    let mut console = Console::new();
    console.set_verbosity(Verbosity::Quiet);
    assert_eq!(console.shell().verbosity(), Verbosity::Quiet);
}

#[rstest]
fn test_console_set_color_choice() {
    let mut console = Console::new();
    console.set_color_choice(ColorChoice::Always);
    assert_eq!(console.shell().color_choice(), ColorChoice::Always);
}

#[rstest]
fn test_console_shell_mut() {
    let mut console = Console::new();
    console.shell_mut().set_verbosity(Verbosity::Verbose);
    assert_eq!(console.shell().verbosity(), Verbosity::Verbose);
}

#[rstest]
fn test_console_global() {
    let console = Console::global();
    // Just verify we can access the global console
    let guard = console.read().unwrap();
    let _ = guard.shell().verbosity();
}

#[rstest]
fn test_console_builder_default() {
    let builder = ConsoleBuilder::new();
    let console = builder.build();
    assert_eq!(console.shell().verbosity(), Verbosity::Normal);
}
