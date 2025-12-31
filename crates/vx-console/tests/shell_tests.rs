//! Shell tests.

use rstest::rstest;
use vx_console::{ColorChoice, Shell, Verbosity};

#[rstest]
fn test_shell_default() {
    let shell = Shell::new();
    assert_eq!(shell.verbosity(), Verbosity::Normal);
}

#[rstest]
fn test_shell_builder_verbosity() {
    let shell = Shell::builder().verbosity(Verbosity::Verbose).build();
    assert_eq!(shell.verbosity(), Verbosity::Verbose);
}

#[rstest]
fn test_shell_builder_color_choice() {
    let shell = Shell::builder().color_choice(ColorChoice::Never).build();
    assert_eq!(shell.color_choice(), ColorChoice::Never);
}

#[rstest]
#[case(Verbosity::Quiet)]
#[case(Verbosity::Normal)]
#[case(Verbosity::Verbose)]
fn test_shell_set_verbosity(#[case] verbosity: Verbosity) {
    let mut shell = Shell::new();
    shell.set_verbosity(verbosity);
    assert_eq!(shell.verbosity(), verbosity);
}

#[rstest]
#[case(ColorChoice::Always)]
#[case(ColorChoice::Never)]
#[case(ColorChoice::Auto)]
fn test_shell_set_color_choice(#[case] color_choice: ColorChoice) {
    let mut shell = Shell::new();
    shell.set_color_choice(color_choice);
    assert_eq!(shell.color_choice(), color_choice);
}

#[rstest]
fn test_verbosity_default() {
    assert_eq!(Verbosity::default(), Verbosity::Normal);
}

#[rstest]
fn test_color_choice_default() {
    assert_eq!(ColorChoice::default(), ColorChoice::Auto);
}

#[rstest]
fn test_shell_needs_clear() {
    let mut shell = Shell::new();
    assert!(!shell.needs_clear());

    shell.set_needs_clear(true);
    assert!(shell.needs_clear());

    shell.set_needs_clear(false);
    assert!(!shell.needs_clear());
}
