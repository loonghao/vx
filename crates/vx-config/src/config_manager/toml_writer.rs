//! TOML Writer - Builder pattern for generating valid TOML output
//!
//! This module provides a safe way to generate TOML content using `toml_edit`,
//! which automatically handles key escaping and value formatting.
//!
//! ## Features
//!
//! - Automatic escaping of special characters in keys (`:`, `.`, spaces, etc.)
//! - Proper value escaping (backslashes, quotes, etc.)
//! - Format-preserving edits when modifying existing TOML
//! - Comment preservation

use std::collections::HashMap;
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value};

/// Builder for generating valid TOML output using `toml_edit`
///
/// Ensures all keys and values are properly escaped according to TOML spec.
/// Uses `toml_edit` internally for correct escaping and formatting.
///
/// # Example
///
/// ```rust,ignore
/// use vx_config::config_manager::TomlWriter;
///
/// let toml = TomlWriter::new()
///     .comment("VX Configuration")
///     .section("tools")
///     .kv("node", "20")
///     .kv("python", "3.11")
///     .section("scripts")
///     .kv("mcp:build", "npm run mcp:build")  // Colon in key is properly escaped
///     .build();
/// ```
#[derive(Debug)]
pub struct TomlWriter {
    doc: DocumentMut,
    current_section: Option<String>,
    pending_comments: Vec<String>,
}

impl Default for TomlWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl TomlWriter {
    /// Create a new TOML writer
    pub fn new() -> Self {
        Self {
            doc: DocumentMut::new(),
            current_section: None,
            pending_comments: Vec::new(),
        }
    }

    /// Add a comment line (will be attached to the next item)
    pub fn comment(mut self, text: &str) -> Self {
        self.pending_comments.push(format!("# {}", text));
        self
    }

    /// Add an empty line (for visual separation)
    pub fn newline(mut self) -> Self {
        self.pending_comments.push(String::new());
        self
    }

    /// Add a section header [section]
    pub fn section(mut self, name: &str) -> Self {
        self.flush_comments_to_section(name);
        self.current_section = Some(name.to_string());

        // Ensure the section exists as a table
        if self.doc.get(name).is_none() {
            self.doc[name] = Item::Table(Table::new());
        }

        self
    }

    /// Add a subsection header [parent.child]
    pub fn subsection(mut self, parent: &str, child: &str) -> Self {
        let full_path = format!("{}.{}", parent, child);
        self.flush_comments_to_section(&full_path);
        self.current_section = Some(full_path);

        // Ensure parent exists
        if self.doc.get(parent).is_none() {
            self.doc[parent] = Item::Table(Table::new());
        }

        // Ensure child exists under parent
        if let Some(Item::Table(parent_table)) = self.doc.get_mut(parent) {
            if parent_table.get(child).is_none() {
                parent_table[child] = Item::Table(Table::new());
            }
        }

        self
    }

    /// Add a key-value pair with string value
    pub fn kv(mut self, key: &str, value: &str) -> Self {
        let decorated = self.create_decorated_value(Value::from(value));
        self.set_value(key, Item::Value(decorated));
        self
    }

    /// Add a key-value pair with boolean value
    pub fn kv_bool(mut self, key: &str, value: bool) -> Self {
        let decorated = self.create_decorated_value(Value::from(value));
        self.set_value(key, Item::Value(decorated));
        self
    }

    /// Add a key-value pair with integer value
    pub fn kv_int(mut self, key: &str, value: i64) -> Self {
        let decorated = self.create_decorated_value(Value::from(value));
        self.set_value(key, Item::Value(decorated));
        self
    }

    /// Add a key-value pair with float value
    pub fn kv_float(mut self, key: &str, value: f64) -> Self {
        let decorated = self.create_decorated_value(Value::from(value));
        self.set_value(key, Item::Value(decorated));
        self
    }

    /// Add a key-value pair with array of strings
    pub fn kv_array(mut self, key: &str, values: &[&str]) -> Self {
        let mut array = Array::new();
        for v in values {
            array.push(*v);
        }
        let decorated = self.create_decorated_value(Value::Array(array));
        self.set_value(key, Item::Value(decorated));
        self
    }

    /// Add multiple key-value pairs from a HashMap (sorted by key)
    pub fn kv_map(mut self, map: &HashMap<String, String>) -> Self {
        let mut entries: Vec<_> = map.iter().collect();
        entries.sort_by_key(|(k, _)| *k);

        for (key, value) in entries {
            self = self.kv(key, value);
        }
        self
    }

    /// Add an inline table { key = "value", ... }
    pub fn kv_inline_table(mut self, key: &str, map: &HashMap<String, String>) -> Self {
        let mut inline = InlineTable::new();
        let mut entries: Vec<_> = map.iter().collect();
        entries.sort_by_key(|(k, _)| *k);

        for (k, v) in entries {
            inline.insert(k, Value::from(v.as_str()));
        }

        let decorated = self.create_decorated_value(Value::InlineTable(inline));
        self.set_value(key, Item::Value(decorated));
        self
    }

    /// Add multiple key-value pairs from a HashMap (sorted by key)
    /// This is an alias for `kv_map` which already sorts.
    pub fn kv_map_sorted(self, map: &HashMap<String, String>) -> Self {
        self.kv_map(map)
    }

    /// Add a key-value pair with a raw (pre-formatted) value
    ///
    /// Use this when you have a value that is already properly formatted
    /// (e.g., "true", "123", or a quoted string like "\"hello\"").
    pub fn kv_raw(mut self, key: &str, raw_value: &str) -> Self {
        // Try to parse the raw value as different TOML types
        let value = if raw_value == "true" {
            Value::from(true)
        } else if raw_value == "false" {
            Value::from(false)
        } else if let Ok(i) = raw_value.parse::<i64>() {
            Value::from(i)
        } else if let Ok(f) = raw_value.parse::<f64>() {
            Value::from(f)
        } else if raw_value.starts_with('"') && raw_value.ends_with('"') {
            // Already quoted string - extract inner value
            let inner = &raw_value[1..raw_value.len() - 1];
            // Unescape the string
            let unescaped = inner.replace("\\\"", "\"").replace("\\\\", "\\");
            Value::from(unescaped)
        } else {
            // Treat as plain string
            Value::from(raw_value)
        };

        let decorated = self.create_decorated_value(value);
        self.set_value(key, Item::Value(decorated));
        self
    }

    /// Build the final TOML string
    pub fn build(self) -> String {
        self.doc.to_string()
    }

    /// Get current length of output
    pub fn len(&self) -> usize {
        self.doc.to_string().len()
    }

    /// Check if output is empty
    pub fn is_empty(&self) -> bool {
        self.doc.as_table().is_empty()
    }

    // Helper: Set value in current section or root
    fn set_value(&mut self, key: &str, item: Item) {
        if let Some(ref section) = self.current_section {
            // Handle nested sections like "parent.child"
            let parts: Vec<&str> = section.split('.').collect();
            if parts.len() == 1 {
                if let Some(Item::Table(table)) = self.doc.get_mut(section) {
                    table[key] = item;
                }
            } else if parts.len() == 2 {
                if let Some(Item::Table(parent)) = self.doc.get_mut(parts[0]) {
                    if let Some(Item::Table(child)) = parent.get_mut(parts[1]) {
                        child[key] = item;
                    }
                }
            }
        } else {
            self.doc[key] = item;
        }
    }

    // Helper: Create a decorated value with pending comments
    fn create_decorated_value(&mut self, value: Value) -> Value {
        if self.pending_comments.is_empty() {
            return value;
        }

        // Build comment prefix
        let comment = self.pending_comments.join("\n") + "\n";
        self.pending_comments.clear();

        // Apply decoration based on value type
        match value {
            Value::String(mut s) => {
                let decor = s.decor_mut();
                decor.set_prefix(comment);
                Value::String(s)
            }
            Value::Integer(mut i) => {
                let decor = i.decor_mut();
                decor.set_prefix(comment);
                Value::Integer(i)
            }
            Value::Float(mut f) => {
                let decor = f.decor_mut();
                decor.set_prefix(comment);
                Value::Float(f)
            }
            Value::Boolean(mut b) => {
                let decor = b.decor_mut();
                decor.set_prefix(comment);
                Value::Boolean(b)
            }
            Value::Array(mut a) => {
                let decor = a.decor_mut();
                decor.set_prefix(comment);
                Value::Array(a)
            }
            Value::InlineTable(mut t) => {
                let decor = t.decor_mut();
                decor.set_prefix(comment);
                Value::InlineTable(t)
            }
            Value::Datetime(d) => Value::Datetime(d),
        }
    }

    // Helper: Flush pending comments to section header
    fn flush_comments_to_section(&mut self, _section: &str) {
        // Comments before sections are handled by toml_edit's decor system
        // For now, we just clear pending comments when starting a new section
        self.pending_comments.clear();
    }
}

/// Format-preserving TOML document editor
///
/// This wrapper around `toml_edit::DocumentMut` provides convenient methods
/// for modifying TOML while preserving comments and formatting.
#[derive(Debug, Clone)]
pub struct TomlDocument {
    doc: DocumentMut,
}

impl TomlDocument {
    /// Parse TOML content into a document
    pub fn parse(content: &str) -> Result<Self, toml_edit::TomlError> {
        let doc = content.parse::<DocumentMut>()?;
        Ok(Self { doc })
    }

    /// Create an empty document
    pub fn new() -> Self {
        Self {
            doc: DocumentMut::new(),
        }
    }

    /// Get a string value from a path like "tools.node"
    pub fn get_string(&self, path: &str) -> Option<String> {
        self.get_value(path)
            .and_then(|v| v.as_str().map(String::from))
    }

    /// Get an integer value from a path
    pub fn get_int(&self, path: &str) -> Option<i64> {
        self.get_value(path).and_then(|v| v.as_integer())
    }

    /// Get a boolean value from a path
    pub fn get_bool(&self, path: &str) -> Option<bool> {
        self.get_value(path).and_then(|v| v.as_bool())
    }

    /// Set a string value at a path, creating intermediate tables as needed
    pub fn set_string(&mut self, path: &str, value: &str) {
        self.set_value(path, Value::from(value));
    }

    /// Set an integer value at a path
    pub fn set_int(&mut self, path: &str, value: i64) {
        self.set_value(path, Value::from(value));
    }

    /// Set a boolean value at a path
    pub fn set_bool(&mut self, path: &str, value: bool) {
        self.set_value(path, Value::from(value));
    }

    /// Set an array of strings at a path
    pub fn set_array(&mut self, path: &str, values: &[&str]) {
        let mut array = Array::new();
        for v in values {
            array.push(*v);
        }
        self.set_value(path, Value::Array(array));
    }

    /// Remove a key at a path
    pub fn remove(&mut self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return false;
        }

        if parts.len() == 1 {
            return self.doc.remove(parts[0]).is_some();
        }

        // Navigate to parent
        let parent_path = parts[..parts.len() - 1].join(".");
        let key = parts[parts.len() - 1];

        if let Some(table) = self.get_table_mut(&parent_path) {
            return table.remove(key).is_some();
        }
        false
    }

    /// Check if a path exists
    pub fn contains(&self, path: &str) -> bool {
        self.get_item(path).is_some()
    }

    /// Get all keys in a section
    pub fn keys(&self, section: &str) -> Vec<String> {
        if let Some(table) = self.get_table(section) {
            table.iter().map(|(k, _)| k.to_string()).collect()
        } else {
            Vec::new()
        }
    }

    /// Convert to TOML string (preserves formatting)
    pub fn to_toml_string(&self) -> String {
        self.doc.to_string()
    }

    /// Get the underlying DocumentMut for advanced operations
    pub fn as_document(&self) -> &DocumentMut {
        &self.doc
    }

    /// Get mutable access to the underlying DocumentMut
    pub fn as_document_mut(&mut self) -> &mut DocumentMut {
        &mut self.doc
    }

    // Helper: Get item at path
    fn get_item(&self, path: &str) -> Option<&Item> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current: &Item = self.doc.as_item();

        for part in parts {
            current = current.get(part)?;
        }
        Some(current)
    }

    // Helper: Get value at path
    fn get_value(&self, path: &str) -> Option<&Value> {
        self.get_item(path).and_then(|i| i.as_value())
    }

    // Helper: Get table at path
    fn get_table(&self, path: &str) -> Option<&Table> {
        self.get_item(path).and_then(|i| i.as_table())
    }

    // Helper: Get mutable table at path
    fn get_table_mut(&mut self, path: &str) -> Option<&mut Table> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current: &mut Item = self.doc.as_item_mut();

        for part in parts {
            current = current.get_mut(part)?;
        }
        current.as_table_mut()
    }

    // Helper: Set value at path, creating tables as needed
    fn set_value(&mut self, path: &str, value: Value) {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            self.doc[parts[0]] = Item::Value(value);
            return;
        }

        // Ensure all parent tables exist
        let mut current = self.doc.as_table_mut();
        for part in &parts[..parts.len() - 1] {
            if current.get(part).is_none() {
                current[part] = Item::Table(Table::new());
            }
            current = current[*part].as_table_mut().unwrap();
        }

        // Set the final value
        let key = parts[parts.len() - 1];
        current[key] = Item::Value(value);
    }
}

impl Default for TomlDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TomlDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.doc)
    }
}

// ============ Helper Functions ============

/// Escape special characters in TOML string values.
///
/// This function escapes backslashes and double quotes, which are the
/// primary characters that need escaping in basic TOML strings.
///
/// # Examples
///
/// ```rust
/// use vx_config::config_manager::escape_toml_string;
///
/// assert_eq!(escape_toml_string("hello"), "hello");
/// assert_eq!(escape_toml_string("C:\\path"), "C:\\\\path");
/// assert_eq!(escape_toml_string("say \"hi\""), "say \\\"hi\\\"");
/// ```
pub fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Escape a TOML key if it contains characters not allowed in bare keys.
///
/// Per TOML specification, bare keys may only contain:
/// - ASCII letters (`A-Z`, `a-z`)
/// - ASCII digits (`0-9`)
/// - Underscores (`_`)
/// - Dashes (`-`)
///
/// Keys with any other characters (including `:`, `.`, spaces, etc.)
/// must be quoted.
///
/// # Examples
///
/// ```rust
/// use vx_config::config_manager::escape_toml_key;
///
/// // Valid bare keys
/// assert_eq!(escape_toml_key("build"), "build");
/// assert_eq!(escape_toml_key("pre-build"), "pre-build");
///
/// // Keys requiring quotes
/// assert_eq!(escape_toml_key("mcp:build"), "\"mcp:build\"");
/// assert_eq!(escape_toml_key("my.script"), "\"my.script\"");
/// ```
pub fn escape_toml_key(key: &str) -> String {
    let is_bare_key = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');

    if is_bare_key {
        key.to_string()
    } else {
        format!("\"{}\"", escape_toml_string(key))
    }
}

/// Format a key-value pair for TOML output.
///
/// This is a convenience function that properly escapes both the key
/// and the string value, producing a complete TOML line.
pub fn format_toml_kv(key: &str, value: &str) -> String {
    format!(
        "{} = \"{}\"",
        escape_toml_key(key),
        escape_toml_string(value)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ TomlWriter Tests ============

    mod toml_writer_tests {
        use super::*;

        #[test]
        fn test_simple_config() {
            let toml = TomlWriter::new()
                .section("tools")
                .kv("node", "20")
                .kv("python", "3.11")
                .build();

            assert!(toml.contains("[tools]"));
            assert!(toml.contains("node = \"20\""));
            assert!(toml.contains("python = \"3.11\""));

            // Verify output is valid TOML
            let parsed: Result<toml::Value, _> = toml::from_str(&toml);
            assert!(parsed.is_ok(), "Output should be valid TOML");
        }

        #[test]
        fn test_special_key_escaping() {
            let toml = TomlWriter::new()
                .section("scripts")
                .kv("mcp:build", "npm run mcp:build")
                .kv("dev:server", "npm run dev")
                .kv("my.dotted.key", "value")
                .kv("key with spaces", "value")
                .build();

            // Verify output is valid TOML
            let parsed: toml::Value = toml::from_str(&toml).expect("Should be valid TOML");
            let scripts = parsed.get("scripts").unwrap().as_table().unwrap();

            assert_eq!(
                scripts.get("mcp:build").unwrap().as_str().unwrap(),
                "npm run mcp:build"
            );
            assert_eq!(
                scripts.get("dev:server").unwrap().as_str().unwrap(),
                "npm run dev"
            );
            assert_eq!(
                scripts.get("my.dotted.key").unwrap().as_str().unwrap(),
                "value"
            );
            assert_eq!(
                scripts.get("key with spaces").unwrap().as_str().unwrap(),
                "value"
            );
        }

        #[test]
        fn test_subsection() {
            let toml = TomlWriter::new()
                .subsection("scripts", "mcp:build")
                .kv("command", "npm run mcp:build")
                .kv("description", "Build MCP")
                .build();

            // Verify output is valid TOML and has correct structure
            let parsed: toml::Value = toml::from_str(&toml).expect("Should be valid TOML");
            let scripts = parsed.get("scripts").unwrap().as_table().unwrap();
            let mcp_build = scripts.get("mcp:build").unwrap().as_table().unwrap();

            assert_eq!(
                mcp_build.get("command").unwrap().as_str().unwrap(),
                "npm run mcp:build"
            );
            assert_eq!(
                mcp_build.get("description").unwrap().as_str().unwrap(),
                "Build MCP"
            );
        }

        #[test]
        fn test_boolean_and_integer() {
            let toml = TomlWriter::new()
                .section("settings")
                .kv_bool("auto_install", true)
                .kv_bool("debug", false)
                .kv_int("timeout", 300)
                .kv_int("negative", -42)
                .kv_float("ratio", 1.5)
                .build();

            let parsed: toml::Value = toml::from_str(&toml).expect("Should be valid TOML");
            let settings = parsed.get("settings").unwrap().as_table().unwrap();

            assert!(settings.get("auto_install").unwrap().as_bool().unwrap());
            assert!(!settings.get("debug").unwrap().as_bool().unwrap());
            assert_eq!(settings.get("timeout").unwrap().as_integer().unwrap(), 300);
            assert_eq!(settings.get("negative").unwrap().as_integer().unwrap(), -42);
            assert!((settings.get("ratio").unwrap().as_float().unwrap() - 1.5).abs() < 0.001);
        }

        #[test]
        fn test_array() {
            let toml = TomlWriter::new()
                .section("hooks")
                .kv_array("post_setup", &["vx run migrate", "vx run seed"])
                .kv_array("empty", &[])
                .build();

            let parsed: toml::Value = toml::from_str(&toml).expect("Should be valid TOML");
            let hooks = parsed.get("hooks").unwrap().as_table().unwrap();
            let post_setup = hooks.get("post_setup").unwrap().as_array().unwrap();

            assert_eq!(post_setup.len(), 2);
            assert_eq!(post_setup[0].as_str().unwrap(), "vx run migrate");
            assert_eq!(post_setup[1].as_str().unwrap(), "vx run seed");

            let empty = hooks.get("empty").unwrap().as_array().unwrap();
            assert!(empty.is_empty());
        }

        #[test]
        fn test_value_escaping() {
            let toml = TomlWriter::new()
                .section("env")
                .kv("PATH", "C:\\Program Files\\node")
                .kv("MESSAGE", "Say \"hello\"")
                .kv("TABS", "col1\tcol2")
                .kv("MIXED", "path\\to\\\"file\"")
                .build();

            // Verify output is valid TOML and values are correctly escaped
            let parsed: toml::Value = toml::from_str(&toml).expect("Should be valid TOML");
            let env = parsed.get("env").unwrap().as_table().unwrap();

            assert_eq!(
                env.get("PATH").unwrap().as_str().unwrap(),
                "C:\\Program Files\\node"
            );
            assert_eq!(
                env.get("MESSAGE").unwrap().as_str().unwrap(),
                "Say \"hello\""
            );
            assert_eq!(env.get("TABS").unwrap().as_str().unwrap(), "col1\tcol2");
            assert_eq!(
                env.get("MIXED").unwrap().as_str().unwrap(),
                "path\\to\\\"file\""
            );
        }

        #[test]
        fn test_inline_table() {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());

            let toml = TomlWriter::new()
                .section("service")
                .kv_inline_table("env", &map)
                .build();

            let parsed: toml::Value = toml::from_str(&toml).expect("Should be valid TOML");
            let service = parsed.get("service").unwrap().as_table().unwrap();
            let env = service.get("env").unwrap().as_table().unwrap();

            assert_eq!(env.get("key1").unwrap().as_str().unwrap(), "value1");
            assert_eq!(env.get("key2").unwrap().as_str().unwrap(), "value2");
        }

        #[test]
        fn test_kv_map() {
            let mut map = HashMap::new();
            map.insert("node".to_string(), "20".to_string());
            map.insert("python".to_string(), "3.11".to_string());

            let toml = TomlWriter::new().section("tools").kv_map(&map).build();

            let parsed: toml::Value = toml::from_str(&toml).expect("Should be valid TOML");
            let tools = parsed.get("tools").unwrap().as_table().unwrap();

            assert_eq!(tools.get("node").unwrap().as_str().unwrap(), "20");
            assert_eq!(tools.get("python").unwrap().as_str().unwrap(), "3.11");
        }

        #[test]
        fn test_empty_writer() {
            let writer = TomlWriter::new();
            assert!(writer.is_empty());

            let toml = writer.build();
            assert!(toml.is_empty() || toml.trim().is_empty());
        }

        #[test]
        fn test_unicode_values() {
            let toml = TomlWriter::new()
                .section("i18n")
                .kv("greeting_zh", "你好世界")
                .kv("greeting_jp", "こんにちは")
                .kv("emoji", "🚀✨")
                .build();

            let parsed: toml::Value = toml::from_str(&toml).expect("Should be valid TOML");
            let i18n = parsed.get("i18n").unwrap().as_table().unwrap();

            assert_eq!(
                i18n.get("greeting_zh").unwrap().as_str().unwrap(),
                "你好世界"
            );
            assert_eq!(
                i18n.get("greeting_jp").unwrap().as_str().unwrap(),
                "こんにちは"
            );
            assert_eq!(i18n.get("emoji").unwrap().as_str().unwrap(), "🚀✨");
        }
    }

    // ============ TomlDocument Tests ============

    mod toml_document_tests {
        use super::*;

        const SAMPLE_TOML: &str = r#"
# VX Configuration
min_version = "0.1.0"

[tools]
node = "20"
python = "3.11"

[scripts]
build = "npm run build"
"mcp:dev" = "npm run mcp:dev"
"#;

        #[test]
        fn test_parse_and_preserve_comments() {
            let doc = TomlDocument::parse(SAMPLE_TOML).expect("Should parse");
            let output = doc.to_toml_string();

            // Comments should be preserved
            assert!(output.contains("# VX Configuration"));
        }

        #[test]
        fn test_get_string_values() {
            let doc = TomlDocument::parse(SAMPLE_TOML).expect("Should parse");

            assert_eq!(doc.get_string("min_version"), Some("0.1.0".to_string()));
            assert_eq!(doc.get_string("tools.node"), Some("20".to_string()));
            assert_eq!(doc.get_string("tools.python"), Some("3.11".to_string()));
            assert_eq!(
                doc.get_string("scripts.build"),
                Some("npm run build".to_string())
            );
            assert_eq!(
                doc.get_string("scripts.mcp:dev"),
                Some("npm run mcp:dev".to_string())
            );

            // Non-existent paths
            assert_eq!(doc.get_string("nonexistent"), None);
            assert_eq!(doc.get_string("tools.nonexistent"), None);
        }

        #[test]
        fn test_set_string_values() {
            let mut doc = TomlDocument::parse(SAMPLE_TOML).expect("Should parse");

            // Update existing value
            doc.set_string("tools.node", "22");
            assert_eq!(doc.get_string("tools.node"), Some("22".to_string()));

            // Add new value to existing section
            doc.set_string("tools.rust", "1.75");
            assert_eq!(doc.get_string("tools.rust"), Some("1.75".to_string()));

            // Add value to new section
            doc.set_string("settings.debug", "true");
            assert_eq!(doc.get_string("settings.debug"), Some("true".to_string()));

            // Verify output is valid TOML
            let output = doc.to_toml_string();
            let _: toml::Value = toml::from_str(&output).expect("Output should be valid TOML");
        }

        #[test]
        fn test_set_various_types() {
            let mut doc = TomlDocument::new();

            doc.set_string("name", "test");
            doc.set_int("count", 42);
            doc.set_bool("enabled", true);
            doc.set_array("items", &["a", "b", "c"]);

            assert_eq!(doc.get_string("name"), Some("test".to_string()));
            assert_eq!(doc.get_int("count"), Some(42));
            assert_eq!(doc.get_bool("enabled"), Some(true));

            // Verify output is valid TOML
            let output = doc.to_toml_string();
            let parsed: toml::Value = toml::from_str(&output).expect("Should be valid TOML");
            let items = parsed.get("items").unwrap().as_array().unwrap();
            assert_eq!(items.len(), 3);
        }

        #[test]
        fn test_remove_values() {
            let mut doc = TomlDocument::parse(SAMPLE_TOML).expect("Should parse");

            // Remove existing key
            assert!(doc.contains("tools.node"));
            assert!(doc.remove("tools.node"));
            assert!(!doc.contains("tools.node"));

            // Remove non-existent key
            assert!(!doc.remove("tools.nonexistent"));

            // Verify output is valid TOML
            let output = doc.to_toml_string();
            let _: toml::Value = toml::from_str(&output).expect("Output should be valid TOML");
        }

        #[test]
        fn test_contains() {
            let doc = TomlDocument::parse(SAMPLE_TOML).expect("Should parse");

            assert!(doc.contains("min_version"));
            assert!(doc.contains("tools"));
            assert!(doc.contains("tools.node"));
            assert!(doc.contains("scripts.mcp:dev"));

            assert!(!doc.contains("nonexistent"));
            assert!(!doc.contains("tools.nonexistent"));
        }

        #[test]
        fn test_keys() {
            let doc = TomlDocument::parse(SAMPLE_TOML).expect("Should parse");

            let tool_keys = doc.keys("tools");
            assert!(tool_keys.contains(&"node".to_string()));
            assert!(tool_keys.contains(&"python".to_string()));

            let script_keys = doc.keys("scripts");
            assert!(script_keys.contains(&"build".to_string()));
            assert!(script_keys.contains(&"mcp:dev".to_string()));

            // Non-existent section
            let empty_keys = doc.keys("nonexistent");
            assert!(empty_keys.is_empty());
        }

        #[test]
        fn test_format_preservation() {
            let toml_with_comments = r#"# Header comment
min_version = "0.1.0"

# Tools section
[tools]
node = "20"  # Node version
python = "3.11"
"#;

            let mut doc = TomlDocument::parse(toml_with_comments).expect("Should parse");

            // Modify a value
            doc.set_string("tools.node", "22");

            let output = doc.to_toml_string();

            // Header comment should be preserved
            assert!(output.contains("# Header comment"));

            // Section comment should be preserved
            assert!(output.contains("# Tools section"));

            // Value should be updated
            let reparsed: toml::Value = toml::from_str(&output).expect("Should be valid TOML");
            assert_eq!(
                reparsed
                    .get("tools")
                    .unwrap()
                    .get("node")
                    .unwrap()
                    .as_str()
                    .unwrap(),
                "22"
            );
        }

        #[test]
        fn test_nested_table_creation() {
            let mut doc = TomlDocument::new();

            // Set deeply nested value - should create intermediate tables
            doc.set_string("a.b.c", "deep");

            assert_eq!(doc.get_string("a.b.c"), Some("deep".to_string()));

            let output = doc.to_toml_string();
            let parsed: toml::Value = toml::from_str(&output).expect("Should be valid TOML");

            assert_eq!(
                parsed
                    .get("a")
                    .unwrap()
                    .get("b")
                    .unwrap()
                    .get("c")
                    .unwrap()
                    .as_str()
                    .unwrap(),
                "deep"
            );
        }

        #[test]
        fn test_parse_error_handling() {
            let invalid_toml = "this is not valid toml [";
            let result = TomlDocument::parse(invalid_toml);
            assert!(result.is_err());
        }

        #[test]
        fn test_empty_document() {
            let doc = TomlDocument::new();
            assert!(!doc.contains("anything"));
            assert!(doc.keys("anything").is_empty());

            let output = doc.to_toml_string();
            assert!(output.is_empty() || output.trim().is_empty());
        }

        #[test]
        fn test_special_characters_in_keys() {
            // Test keys with colons, spaces (but NOT dots, as dots are path separators)
            let toml_with_special_keys = r#"
[scripts]
"mcp:build" = "npm run mcp:build"
"key with space" = "value"
"key:with:colons" = "multiple colons"
"#;

            let doc = TomlDocument::parse(toml_with_special_keys).expect("Should parse");

            assert_eq!(
                doc.get_string("scripts.mcp:build"),
                Some("npm run mcp:build".to_string())
            );
            assert_eq!(
                doc.get_string("scripts.key with space"),
                Some("value".to_string())
            );
            assert_eq!(
                doc.get_string("scripts.key:with:colons"),
                Some("multiple colons".to_string())
            );

            // Note: Keys with dots like "dev.server" cannot be accessed via get_string()
            // because dots are used as path separators. Use document() directly for such keys.
        }
    }

    // ============ Integration Tests ============

    mod integration_tests {
        use super::*;

        #[test]
        fn test_writer_output_can_be_parsed_by_document() {
            let toml_str = TomlWriter::new()
                .section("tools")
                .kv("node", "20")
                .kv("mcp:runtime", "latest")
                .section("scripts")
                .kv("build", "npm run build")
                .kv("test:unit", "npm test")
                .build();

            // Parse with TomlDocument
            let doc = TomlDocument::parse(&toml_str).expect("Should parse writer output");

            assert_eq!(doc.get_string("tools.node"), Some("20".to_string()));
            assert_eq!(
                doc.get_string("tools.mcp:runtime"),
                Some("latest".to_string())
            );
            assert_eq!(
                doc.get_string("scripts.build"),
                Some("npm run build".to_string())
            );
            assert_eq!(
                doc.get_string("scripts.test:unit"),
                Some("npm test".to_string())
            );
        }

        #[test]
        fn test_roundtrip_modification() {
            // Create initial config with TomlWriter
            let initial = TomlWriter::new()
                .section("tools")
                .kv("node", "18")
                .kv("python", "3.10")
                .build();

            // Parse and modify with TomlDocument
            let mut doc = TomlDocument::parse(&initial).expect("Should parse");
            doc.set_string("tools.node", "20");
            doc.set_string("tools.rust", "1.75");

            // Verify modifications
            let output = doc.to_toml_string();
            let final_doc = TomlDocument::parse(&output).expect("Should parse modified");

            assert_eq!(final_doc.get_string("tools.node"), Some("20".to_string()));
            assert_eq!(
                final_doc.get_string("tools.python"),
                Some("3.10".to_string())
            );
            assert_eq!(final_doc.get_string("tools.rust"), Some("1.75".to_string()));
        }
    }
}
