//! Shell command parsing and quoting utilities
//!
//! This module provides safe command parsing and quoting using the `shell-words` crate.
//! It handles the complexities of shell escaping across different platforms.

use crate::error::EnvError;

/// Parse a shell command string into individual arguments
///
/// This handles quoted strings, escaped characters, and other shell syntax.
///
/// # Example
///
/// ```rust
/// use vx_env::parse_command;
///
/// let args = parse_command("echo 'hello world' --flag").unwrap();
/// assert_eq!(args, vec!["echo", "hello world", "--flag"]);
/// ```
pub fn parse_command(cmd: &str) -> Result<Vec<String>, EnvError> {
    shell_words::split(cmd).map_err(|e| EnvError::CommandParse(e.to_string()))
}

/// Quote a single argument for safe shell usage
///
/// This ensures the argument is properly escaped for use in a shell command.
///
/// # Example
///
/// ```rust
/// use vx_env::quote_arg;
///
/// assert_eq!(quote_arg("hello"), "hello");
/// assert_eq!(quote_arg("hello world"), "'hello world'");
/// assert_eq!(quote_arg("it's"), "'it'\\''s'");
/// ```
pub fn quote_arg(arg: &str) -> String {
    shell_words::quote(arg).into_owned()
}

/// Join arguments into a shell command string
///
/// Each argument is properly quoted if necessary.
///
/// # Example
///
/// ```rust
/// use vx_env::join_args;
///
/// let cmd = join_args(&["echo", "hello world", "--flag"]);
/// assert_eq!(cmd, "echo 'hello world' --flag");
/// ```
pub fn join_args(args: &[&str]) -> String {
    shell_words::join(args)
}

/// Quote arguments and join them into a command string
///
/// Useful when you have owned strings instead of references.
pub fn join_args_owned(args: &[String]) -> String {
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    shell_words::join(&refs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let args = parse_command("echo hello world").unwrap();
        assert_eq!(args, vec!["echo", "hello", "world"]);
    }

    #[test]
    fn test_parse_quoted() {
        let args = parse_command("echo 'hello world'").unwrap();
        assert_eq!(args, vec!["echo", "hello world"]);
    }

    #[test]
    fn test_parse_double_quoted() {
        let args = parse_command(r#"echo "hello world""#).unwrap();
        assert_eq!(args, vec!["echo", "hello world"]);
    }

    #[test]
    fn test_parse_escaped() {
        let args = parse_command(r"echo hello\ world").unwrap();
        assert_eq!(args, vec!["echo", "hello world"]);
    }

    #[test]
    fn test_parse_complex() {
        let args = parse_command(r#"npm run build --env="production" --flag"#).unwrap();
        assert_eq!(
            args,
            vec!["npm", "run", "build", "--env=production", "--flag"]
        );
    }

    #[test]
    fn test_quote_simple() {
        assert_eq!(quote_arg("hello"), "hello");
    }

    #[test]
    fn test_quote_with_space() {
        let quoted = quote_arg("hello world");
        assert!(quoted.contains("hello world"));
    }

    #[test]
    fn test_quote_with_single_quote() {
        let quoted = quote_arg("it's");
        // Should be properly escaped
        assert!(quoted.contains("it"));
        assert!(quoted.contains("s"));
    }

    #[test]
    fn test_join_args() {
        let cmd = join_args(&["echo", "hello world", "--flag"]);
        assert!(cmd.starts_with("echo"));
        assert!(cmd.contains("hello world") || cmd.contains("'hello world'"));
        assert!(cmd.ends_with("--flag"));
    }

    #[test]
    fn test_roundtrip() {
        let original = vec!["npm", "run", "build --prod", "arg with spaces"];
        let joined = join_args(&original);
        let parsed = parse_command(&joined).unwrap();
        assert_eq!(parsed, original);
    }
}
