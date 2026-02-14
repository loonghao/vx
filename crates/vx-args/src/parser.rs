//! Argument parser implementation

use crate::error::{ArgError, ArgResult};
use crate::types::{ArgDef, ArgType, ArgValue};
use indexmap::IndexMap;
use regex::Regex;
use std::collections::HashMap;

/// Argument parser
#[derive(Debug)]
pub struct ArgParser {
    /// Script/command name
    name: String,
    /// Description
    description: Option<String>,
    /// Argument definitions (ordered)
    args: IndexMap<String, ArgDef>,
    /// Positional argument order
    positional_order: Vec<String>,
    /// Short flag mapping
    short_map: HashMap<char, String>,
    /// Whether to allow unknown arguments
    allow_unknown: bool,
    /// Extra arguments after --
    allow_passthrough: bool,
}

impl ArgParser {
    /// Create a new argument parser
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            args: IndexMap::new(),
            positional_order: Vec::new(),
            short_map: HashMap::new(),
            allow_unknown: false,
            allow_passthrough: true,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Allow unknown arguments
    pub fn allow_unknown(mut self, allow: bool) -> Self {
        self.allow_unknown = allow;
        self
    }

    /// Allow passthrough arguments after --
    pub fn allow_passthrough(mut self, allow: bool) -> Self {
        self.allow_passthrough = allow;
        self
    }

    /// Add an argument definition
    pub fn add_arg(&mut self, arg: ArgDef) {
        // Track short flag
        if let Some(short) = arg.short {
            self.short_map.insert(short, arg.name.clone());
        }

        // Track positional order
        if arg.positional {
            self.positional_order.push(arg.name.clone());
        }

        self.args.insert(arg.name.clone(), arg);
    }

    /// Add a positional argument
    pub fn positional(&mut self, arg: ArgDef) {
        let mut arg = arg;
        arg.positional = true;
        self.add_arg(arg);
    }

    /// Get argument definition by name
    pub fn get_arg(&self, name: &str) -> Option<&ArgDef> {
        self.args.get(name)
    }

    /// Get all argument definitions
    pub fn args(&self) -> impl Iterator<Item = &ArgDef> {
        self.args.values()
    }

    /// Get the parser name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Parse command-line arguments
    pub fn parse(&self, args: &[impl AsRef<str>]) -> ArgResult<ParsedArgs> {
        let mut result = ParsedArgs::new();
        let mut positional_idx = 0;
        let mut i = 0;
        let args: Vec<&str> = args.iter().map(|a| a.as_ref()).collect();

        while i < args.len() {
            let arg = args[i];

            // Check for passthrough marker
            if arg == "--" {
                if self.allow_passthrough {
                    result.passthrough = args[i + 1..].iter().map(|s| s.to_string()).collect();
                }
                break;
            }

            // Check for long flag
            if let Some(stripped) = arg.strip_prefix("--") {
                i = self.parse_long_flag(stripped, &args, i, &mut result)?;
            }
            // Check for short flag
            else if let Some(stripped) = arg.strip_prefix('-') {
                if stripped.is_empty() {
                    // Single dash is treated as positional
                    self.parse_positional("-", &mut positional_idx, &mut result)?;
                } else {
                    i = self.parse_short_flags(stripped, &args, i, &mut result)?;
                }
            }
            // Positional argument
            else {
                self.parse_positional(arg, &mut positional_idx, &mut result)?;
            }

            i += 1;
        }

        // Apply defaults and check required
        self.finalize(&mut result)?;

        Ok(result)
    }

    /// Parse a long flag (--name or --name=value)
    fn parse_long_flag(
        &self,
        flag: &str,
        args: &[&str],
        idx: usize,
        result: &mut ParsedArgs,
    ) -> ArgResult<usize> {
        let (name, inline_value) = if let Some(eq_pos) = flag.find('=') {
            (&flag[..eq_pos], Some(&flag[eq_pos + 1..]))
        } else {
            (flag, None)
        };

        // Convert kebab-case to snake_case for lookup
        let lookup_name = name.replace('-', "_");

        let arg_def = self.args.get(&lookup_name).ok_or_else(|| {
            ArgError::unknown_argument(format!("--{}", name), self.find_similar(&lookup_name))
        })?;

        match arg_def.arg_type {
            ArgType::Flag => {
                // Handle --no-xxx for negation
                if let Some(stripped) = name.strip_prefix("no-") {
                    let pos_name = stripped.replace('-', "_");
                    if self.args.contains_key(&pos_name) {
                        result.set(&pos_name, ArgValue::Bool(false));
                        return Ok(idx);
                    }
                }
                result.set(&lookup_name, ArgValue::Bool(true));
                Ok(idx)
            }
            ArgType::Array => {
                let value = if let Some(v) = inline_value {
                    v.to_string()
                } else if idx + 1 < args.len() && !args[idx + 1].starts_with('-') {
                    let v = args[idx + 1].to_string();
                    result.append_array(&lookup_name, v);
                    return Ok(idx + 1);
                } else {
                    return Err(ArgError::invalid_value(
                        &lookup_name,
                        "expected a value",
                        None,
                    ));
                };
                result.append_array(&lookup_name, value);
                Ok(idx)
            }
            _ => {
                let value = if let Some(v) = inline_value {
                    v.to_string()
                } else if idx + 1 < args.len() {
                    let v = args[idx + 1].to_string();
                    let parsed = self.parse_value(arg_def, &v)?;
                    result.set(&lookup_name, parsed);
                    return Ok(idx + 1);
                } else {
                    return Err(ArgError::invalid_value(
                        &lookup_name,
                        "expected a value",
                        None,
                    ));
                };
                let parsed = self.parse_value(arg_def, &value)?;
                result.set(&lookup_name, parsed);
                Ok(idx)
            }
        }
    }

    /// Parse short flags (-v, -vvv, -abc, -o value)
    fn parse_short_flags(
        &self,
        flags: &str,
        args: &[&str],
        idx: usize,
        result: &mut ParsedArgs,
    ) -> ArgResult<usize> {
        let chars: Vec<char> = flags.chars().collect();
        let mut i = 0;
        let mut consumed_next = false;

        while i < chars.len() {
            let c = chars[i];
            let name = self
                .short_map
                .get(&c)
                .ok_or_else(|| ArgError::unknown_argument(format!("-{}", c), vec![]))?;

            let arg_def = self.args.get(name).unwrap();

            match arg_def.arg_type {
                ArgType::Flag => {
                    // Count consecutive same flags for -vvv style
                    let mut count = 1;
                    while i + count < chars.len() && chars[i + count] == c {
                        count += 1;
                    }
                    if count > 1 {
                        // Multiple flags = count value
                        result.set(name, ArgValue::Number(count as f64));
                        i += count;
                    } else {
                        result.set(name, ArgValue::Bool(true));
                        i += 1;
                    }
                }
                _ => {
                    // Value-taking flag
                    let value = if i + 1 < chars.len() {
                        // Inline value: -ovalue
                        chars[i + 1..].iter().collect::<String>()
                    } else if idx + 1 < args.len() {
                        // Next argument: -o value
                        consumed_next = true;
                        args[idx + 1].to_string()
                    } else {
                        return Err(ArgError::invalid_value(name, "expected a value", None));
                    };

                    let parsed = self.parse_value(arg_def, &value)?;
                    result.set(name, parsed);
                    break;
                }
            }
        }

        Ok(if consumed_next { idx + 1 } else { idx })
    }

    /// Parse a positional argument
    fn parse_positional(
        &self,
        value: &str,
        idx: &mut usize,
        result: &mut ParsedArgs,
    ) -> ArgResult<()> {
        if *idx >= self.positional_order.len() {
            if self.allow_unknown {
                result.extra.push(value.to_string());
                return Ok(());
            }
            return Err(ArgError::TooManyArguments {
                max: self.positional_order.len(),
                actual: *idx + 1,
            });
        }

        let name = &self.positional_order[*idx];
        let arg_def = self.args.get(name).unwrap();

        // Check if this is a variadic array argument
        if arg_def.arg_type == ArgType::Array {
            result.append_array(name, value.to_string());
            // Don't increment index for variadic args
        } else {
            let parsed = self.parse_value(arg_def, value)?;
            result.set(name, parsed);
            *idx += 1;
        }

        Ok(())
    }

    /// Parse a value according to argument definition
    fn parse_value(&self, arg_def: &ArgDef, value: &str) -> ArgResult<ArgValue> {
        // Validate choices
        if !arg_def.choices.is_empty() && !arg_def.choices.contains(&value.to_string()) {
            return Err(ArgError::invalid_value(
                &arg_def.name,
                format!("'{}' is not a valid choice", value),
                Some(arg_def.choices.clone()),
            ));
        }

        // Validate pattern
        if let Some(pattern) = &arg_def.pattern {
            let re = Regex::new(pattern).map_err(|e| ArgError::InvalidPattern {
                name: arg_def.name.clone(),
                pattern: pattern.clone(),
                message: e.to_string(),
            })?;
            if !re.is_match(value) {
                return Err(ArgError::invalid_value(
                    &arg_def.name,
                    format!("'{}' does not match pattern '{}'", value, pattern),
                    None,
                ));
            }
        }

        // Parse by type
        match arg_def.arg_type {
            ArgType::String => Ok(ArgValue::String(value.to_string())),
            ArgType::Number => {
                let num: f64 = value
                    .parse()
                    .map_err(|_| ArgError::type_mismatch(&arg_def.name, "number", value))?;
                Ok(ArgValue::Number(num))
            }
            ArgType::Flag => {
                let b = matches!(value.to_lowercase().as_str(), "true" | "1" | "yes" | "on");
                Ok(ArgValue::Bool(b))
            }
            ArgType::Array => Ok(ArgValue::String(value.to_string())),
        }
    }

    /// Apply defaults and check required arguments
    fn finalize(&self, result: &mut ParsedArgs) -> ArgResult<()> {
        for arg_def in self.args.values() {
            if result.values.contains_key(&arg_def.name) {
                continue;
            }

            // Try environment variable
            if let Some(env_var) = &arg_def.env
                && let Ok(value) = std::env::var(env_var)
            {
                let parsed = self.parse_value(arg_def, &value)?;
                result.set(&arg_def.name, parsed);
                continue;
            }

            // Apply default
            if let Some(default) = &arg_def.default {
                let parsed = self.parse_value(arg_def, default)?;
                result.set(&arg_def.name, parsed);
                continue;
            }

            // Set default for flags
            if arg_def.arg_type == ArgType::Flag {
                result.set(&arg_def.name, ArgValue::Bool(false));
                continue;
            }

            // Set empty array for array args
            if arg_def.arg_type == ArgType::Array {
                result.set(&arg_def.name, ArgValue::Array(vec![]));
                continue;
            }

            // Check required
            if arg_def.required {
                return Err(ArgError::missing_required(
                    &arg_def.name,
                    arg_def.help.clone(),
                ));
            }

            // Set none for optional
            result.set(&arg_def.name, ArgValue::None);
        }

        Ok(())
    }

    /// Find similar argument names for suggestions
    fn find_similar(&self, name: &str) -> Vec<String> {
        self.args
            .keys()
            .filter(|k| {
                // Simple similarity check
                k.contains(name) || name.contains(k.as_str()) || levenshtein(k, name) <= 2
            })
            .take(3)
            .map(|k| format!("--{}", k.replace('_', "-")))
            .collect()
    }
}

/// Parsed arguments result
#[derive(Debug, Default)]
pub struct ParsedArgs {
    /// Parsed values
    values: HashMap<String, ArgValue>,
    /// Extra positional arguments (when allow_unknown is true)
    extra: Vec<String>,
    /// Passthrough arguments (after --)
    passthrough: Vec<String>,
}

impl ParsedArgs {
    /// Create new empty parsed args
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a value
    pub fn set(&mut self, name: &str, value: ArgValue) {
        self.values.insert(name.to_string(), value);
    }

    /// Append to an array value
    pub fn append_array(&mut self, name: &str, value: String) {
        match self.values.get_mut(name) {
            Some(ArgValue::Array(arr)) => arr.push(value),
            _ => {
                self.values
                    .insert(name.to_string(), ArgValue::Array(vec![value]));
            }
        }
    }

    /// Get a value
    pub fn get(&self, name: &str) -> Option<&ArgValue> {
        self.values.get(name)
    }

    /// Get as string
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.values.get(name).and_then(|v| v.as_string())
    }

    /// Get as bool
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.values.get(name).and_then(|v| v.as_bool())
    }

    /// Get as number
    pub fn get_number(&self, name: &str) -> Option<f64> {
        self.values.get(name).and_then(|v| v.as_number())
    }

    /// Get as array
    pub fn get_array(&self, name: &str) -> Option<&[String]> {
        self.values.get(name).and_then(|v| v.as_array())
    }

    /// Get extra arguments
    pub fn extra(&self) -> &[String] {
        &self.extra
    }

    /// Get passthrough arguments
    pub fn passthrough(&self) -> &[String] {
        &self.passthrough
    }

    /// Check if a value exists and is not None
    pub fn has(&self, name: &str) -> bool {
        self.values.get(name).map(|v| !v.is_none()).unwrap_or(false)
    }

    /// Get all values as a HashMap for environment variable injection
    pub fn to_env_map(&self) -> HashMap<String, String> {
        self.values
            .iter()
            .filter(|(_, v)| !v.is_none())
            .map(|(k, v)| (k.to_uppercase(), v.to_string_value()))
            .collect()
    }

    /// Iterate over all values
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ArgValue)> {
        self.values.iter()
    }
}

/// Simple Levenshtein distance for suggestions
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut matrix = vec![vec![0; b.len() + 1]; a.len() + 1];

    for (i, row) in matrix.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate() {
        *cell = j;
    }

    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a.len()][b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

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
        parser.add_arg(
            ArgDef::new("services")
                .arg_type(ArgType::Array)
                .short('s')
                .help("Services to deploy"),
        );
        parser
    }

    #[test]
    fn test_positional_args() {
        let parser = create_test_parser();
        let args = vec!["prod"];
        let parsed = parser.parse(&args).unwrap();

        assert_eq!(parsed.get_string("environment"), Some("prod"));
        assert_eq!(parsed.get_string("region"), Some("us-east-1")); // default
    }

    #[test]
    fn test_named_args() {
        let parser = create_test_parser();
        let args = vec!["dev", "--region", "us-west-2"];
        let parsed = parser.parse(&args).unwrap();

        assert_eq!(parsed.get_string("environment"), Some("dev"));
        assert_eq!(parsed.get_string("region"), Some("us-west-2"));
    }

    #[test]
    fn test_flag_args() {
        let parser = create_test_parser();
        let args = vec!["dev", "-v"];
        let parsed = parser.parse(&args).unwrap();

        assert_eq!(parsed.get_bool("verbose"), Some(true));
    }

    #[test]
    fn test_array_args() {
        let parser = create_test_parser();
        let args = vec!["dev", "--services", "api", "--services", "web"];
        let parsed = parser.parse(&args).unwrap();

        assert_eq!(
            parsed.get_array("services"),
            Some(&["api".to_string(), "web".to_string()][..])
        );
    }

    #[test]
    fn test_invalid_choice() {
        let parser = create_test_parser();
        let args = vec!["invalid"];
        let result = parser.parse(&args);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ArgError::InvalidValue { .. }));
    }

    #[test]
    fn test_missing_required() {
        let parser = create_test_parser();
        let args: Vec<&str> = vec![];
        let result = parser.parse(&args);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ArgError::MissingRequired { .. }
        ));
    }

    #[test]
    fn test_passthrough() {
        let parser = create_test_parser();
        let args = vec!["dev", "--", "extra1", "extra2"];
        let parsed = parser.parse(&args).unwrap();

        assert_eq!(parsed.passthrough(), &["extra1", "extra2"]);
    }

    #[test]
    fn test_inline_value() {
        let parser = create_test_parser();
        let args = vec!["dev", "--region=eu-west-1"];
        let parsed = parser.parse(&args).unwrap();

        assert_eq!(parsed.get_string("region"), Some("eu-west-1"));
    }

    #[test]
    fn test_to_env_map() {
        let parser = create_test_parser();
        let args = vec!["prod", "--region", "us-west-2", "-v"];
        let parsed = parser.parse(&args).unwrap();

        let env_map = parsed.to_env_map();
        assert_eq!(env_map.get("ENVIRONMENT"), Some(&"prod".to_string()));
        assert_eq!(env_map.get("REGION"), Some(&"us-west-2".to_string()));
        assert_eq!(env_map.get("VERBOSE"), Some(&"true".to_string()));
    }
}
