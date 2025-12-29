//! Help text generation for argument parsers

use crate::parser::ArgParser;
use crate::types::{ArgDef, ArgType};

/// Help text formatter
pub struct HelpFormatter {
    /// Maximum width for wrapping
    max_width: usize,
    /// Indent for descriptions
    indent: usize,
}

impl HelpFormatter {
    /// Create a new help formatter
    pub fn new() -> Self {
        Self {
            max_width: 80,
            indent: 24,
        }
    }

    /// Set maximum width
    pub fn max_width(mut self, width: usize) -> Self {
        self.max_width = width;
        self
    }

    /// Set indent for descriptions
    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    /// Format help text for an argument parser
    pub fn format(&self, parser: &ArgParser) -> String {
        let mut output = String::new();

        // Usage line
        output.push_str(&self.format_usage(parser));
        output.push('\n');

        // Description
        if let Some(desc) = parser.description() {
            output.push('\n');
            output.push_str(desc);
            output.push('\n');
        }

        // Arguments section
        let positional: Vec<_> = parser.args().filter(|a| a.positional).collect();
        let options: Vec<_> = parser.args().filter(|a| !a.positional).collect();

        if !positional.is_empty() {
            output.push_str("\nArguments:\n");
            for arg in positional {
                output.push_str(&self.format_arg(arg));
            }
        }

        if !options.is_empty() {
            output.push_str("\nOptions:\n");
            for arg in options {
                output.push_str(&self.format_option(arg));
            }
        }

        // Standard options
        output.push_str("  -h, --help          Show this help message\n");

        output
    }

    /// Format the usage line
    fn format_usage(&self, parser: &ArgParser) -> String {
        let mut usage = format!("Usage: vx run {}", parser.name());

        // Add positional arguments
        for arg in parser.args().filter(|a| a.positional) {
            if arg.required {
                usage.push_str(&format!(" <{}>", arg.name));
            } else {
                usage.push_str(&format!(" [{}]", arg.name));
            }
        }

        // Add options indicator
        let has_options = parser.args().any(|a| !a.positional);
        if has_options {
            usage.push_str(" [options]");
        }

        usage
    }

    /// Format a positional argument
    fn format_arg(&self, arg: &ArgDef) -> String {
        let mut line = format!("  {:<20}", arg.name);

        if let Some(help) = &arg.help {
            line.push_str(help);
        }

        if arg.required {
            line.push_str(" (required)");
        }

        line.push('\n');

        // Add choices
        if !arg.choices.is_empty() {
            line.push_str(&format!(
                "  {:<20}Choices: {}\n",
                "",
                arg.choices.join(", ")
            ));
        }

        // Add default
        if let Some(default) = &arg.default {
            line.push_str(&format!("  {:<20}[default: {}]\n", "", default));
        }

        line
    }

    /// Format an option argument
    fn format_option(&self, arg: &ArgDef) -> String {
        let mut flags = String::new();

        // Short flag
        if let Some(short) = arg.short {
            flags.push_str(&format!("-{}", short));
            flags.push_str(", ");
        } else {
            flags.push_str("    ");
        }

        // Long flag
        flags.push_str(&arg.long_flag());

        // Value placeholder
        match arg.arg_type {
            ArgType::Flag => {}
            ArgType::Array => flags.push_str(&format!(" <{}>...", arg.name)),
            _ => flags.push_str(&format!(" <{}>", arg.name)),
        }

        let mut line = format!("  {:<22}", flags);

        // Help text
        if let Some(help) = &arg.help {
            line.push_str(help);
        }

        line.push('\n');

        // Add choices
        if !arg.choices.is_empty() {
            line.push_str(&format!(
                "  {:<22}Choices: {}\n",
                "",
                arg.choices.join(", ")
            ));
        }

        // Add default
        if let Some(default) = &arg.default {
            line.push_str(&format!("  {:<22}[default: {}]\n", "", default));
        }

        // Add env var
        if let Some(env) = &arg.env {
            line.push_str(&format!("  {:<22}[env: {}]\n", "", env));
        }

        line
    }

    /// Format a short usage hint for error messages
    pub fn format_short_usage(&self, parser: &ArgParser) -> String {
        format!(
            "Usage: vx run {} [options]\n\nFor more information, try '--help'",
            parser.name()
        )
    }
}

impl Default for HelpFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ArgParser;
    use crate::types::{ArgDef, ArgType};

    fn create_test_parser() -> ArgParser {
        let mut parser = ArgParser::new("deploy");
        parser.positional(
            ArgDef::new("environment")
                .required(true)
                .choices(vec!["dev", "staging", "prod"])
                .help("Target environment"),
        );
        parser.add_arg(
            ArgDef::new("region")
                .default("us-east-1")
                .help("Cloud region"),
        );
        parser.add_arg(
            ArgDef::new("verbose")
                .arg_type(ArgType::Flag)
                .short('v')
                .help("Enable verbose output"),
        );
        parser
    }

    #[test]
    fn test_format_help() {
        let parser = create_test_parser();
        let formatter = HelpFormatter::new();
        let help = formatter.format(&parser);

        assert!(help.contains("Usage: vx run deploy"));
        assert!(help.contains("<environment>"));
        assert!(help.contains("Target environment"));
        assert!(help.contains("--region"));
        assert!(help.contains("-v, --verbose"));
        assert!(help.contains("--help"));
    }

    #[test]
    fn test_format_choices() {
        let parser = create_test_parser();
        let formatter = HelpFormatter::new();
        let help = formatter.format(&parser);

        assert!(help.contains("Choices: dev, staging, prod"));
    }

    #[test]
    fn test_format_default() {
        let parser = create_test_parser();
        let formatter = HelpFormatter::new();
        let help = formatter.format(&parser);

        assert!(help.contains("[default: us-east-1]"));
    }
}
