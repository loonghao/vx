use vx_manifest::{
    CommandDef, Ecosystem, EnvConfig, OutputConfig, ProviderManifest, ShellConfig, TestCommand,
    TestConfig, InlineTestScripts,
};

#[test]
fn test_parse_minimal_manifest() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "test-runtime"
executable = "test"
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    assert_eq!(manifest.provider.name, "test");
    assert_eq!(manifest.runtimes.len(), 1);
    assert_eq!(manifest.runtimes[0].name, "test-runtime");
}

#[test]
fn test_parse_full_manifest() {
    let toml = r#"
[provider]
name = "yarn"
description = "Fast, reliable, and secure dependency management"
homepage = "https://yarnpkg.com"
ecosystem = "nodejs"

[[runtimes]]
name = "yarn"
description = "Yarn package manager"
executable = "yarn"
aliases = ["yarnpkg"]

[[runtimes.constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=12, <23", reason = "Yarn 1.x requires Node.js 12-22" }
]

[[runtimes.constraints]]
when = ">=4"
requires = [
    { runtime = "node", version = ">=18", recommended = "22" }
]
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    assert_eq!(manifest.provider.name, "yarn");
    assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::NodeJs));

    let runtime = &manifest.runtimes[0];
    assert_eq!(runtime.name, "yarn");
    assert_eq!(runtime.aliases, vec!["yarnpkg"]);
    assert_eq!(runtime.constraints.len(), 2);

    let v1_constraints = runtime.get_constraints_for_version("1.22.22");
    assert_eq!(v1_constraints.len(), 1);
    assert_eq!(v1_constraints[0].requires.len(), 1);
    assert_eq!(v1_constraints[0].requires[0].runtime, "node");

    let v4_constraints = runtime.get_constraints_for_version("4.0.0");
    assert_eq!(v4_constraints.len(), 1);
    assert_eq!(v4_constraints[0].requires[0].version, ">=18");
}

#[test]
fn test_parse_manifest_with_platform_constraint() {
    let toml = r#"
[provider]
name = "msvc"
description = "Microsoft Visual C++ Compiler"
ecosystem = "system"

[provider.platforms]
os = ["windows"]

[[runtimes]]
name = "cl"
description = "MSVC C/C++ Compiler"
executable = "cl"
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    assert_eq!(manifest.provider.name, "msvc");

    let platform_constraint = manifest.provider.platform_constraint.as_ref().unwrap();
    assert_eq!(platform_constraint.os.len(), 1);
    assert_eq!(manifest.platform_description(), Some("Windows only".to_string()));
    assert_eq!(manifest.platform_label(), Some("Windows".to_string()));
}

#[test]
fn test_parse_runtime_with_platform_constraint() {
    let toml = r#"
[provider]
name = "xcode"
description = "Apple Xcode Command Line Tools"

[[runtimes]]
name = "xcodebuild"
executable = "xcodebuild"

[runtimes.platform_constraint]
os = ["macos"]
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let runtime = &manifest.runtimes[0];

    let platform_constraint = runtime.platform_constraint.as_ref().unwrap();
    assert_eq!(platform_constraint.os.len(), 1);
    assert_eq!(runtime.platform_description(), Some("macOS only".to_string()));
}

#[test]
fn test_supported_runtimes() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "cross-platform"
executable = "cross"

[[runtimes]]
name = "windows-only"
executable = "win"

[runtimes.platform_constraint]
os = ["windows"]

[[runtimes]]
name = "macos-only"
executable = "mac"

[runtimes.platform_constraint]
os = ["macos"]
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();

    let supported = manifest.supported_runtimes();
    assert!(supported.iter().any(|r| r.name == "cross-platform"));
}

#[test]
fn test_parse_extended_hooks() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "test"
executable = "test"

[runtimes.hooks]
pre_install = ["check-prereqs.sh"]
post_install = ["setup.sh", "verify.sh"]
pre_activate = ["save-env.sh"]
post_activate = ["load-settings.sh"]
on_install_error = ["rollback.sh"]

[runtimes.hooks.config]
fail_on_error = true
timeout_ms = 60000
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let hooks = manifest.runtimes[0].hooks.as_ref().unwrap();

    assert_eq!(hooks.pre_install, vec!["check-prereqs.sh"]);
    assert_eq!(hooks.post_install, vec!["setup.sh", "verify.sh"]);
    assert_eq!(hooks.pre_activate, vec!["save-env.sh"]);
    assert_eq!(hooks.on_install_error, vec!["rollback.sh"]);

    let config = hooks.config.as_ref().unwrap();
    assert!(config.fail_on_error);
    assert_eq!(config.timeout_ms, 60000);
}

#[test]
fn test_parse_detection_config() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.detection]
command = "{executable} --version"
pattern = "v?(\\d+\\.\\d+\\.\\d+)"
system_paths = ["/usr/bin/node", "/usr/local/bin/node"]
env_hints = ["NODE_HOME", "NVM_DIR"]
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let detection = manifest.runtimes[0].detection.as_ref().unwrap();

    assert_eq!(detection.command, "{executable} --version");
    assert_eq!(detection.pattern, r"v?(\d+\.\d+\.\d+)");
    assert_eq!(detection.system_paths.len(), 2);
    assert_eq!(detection.env_hints, vec!["NODE_HOME", "NVM_DIR"]);
}

#[test]
fn test_parse_health_config() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.health]
check_command = "{executable} -e 'console.log(process.version)'"
expected_pattern = "v\\d+\\.\\d+\\.\\d+"
exit_code = 0
timeout_ms = 3000
check_on = ["install", "activate"]
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let health = manifest.runtimes[0].health.as_ref().unwrap();

    assert_eq!(
        health.check_command,
        "{executable} -e 'console.log(process.version)'"
    );
    assert_eq!(health.expected_pattern, Some(r"v\d+\.\d+\.\d+".to_string()));
    assert_eq!(health.exit_code, Some(0));
    assert_eq!(health.timeout_ms, 3000);
    assert_eq!(health.check_on, vec!["install", "activate"]);
}

#[test]
fn test_parse_mirror_config() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "node"
executable = "node"

[[runtimes.mirrors]]
name = "taobao"
region = "cn"
url = "https://npmmirror.com/mirrors/node"
priority = 100

[[runtimes.mirrors]]
name = "ustc"
region = "cn"
url = "https://mirrors.ustc.edu.cn/node"
priority = 90
enabled = false
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let mirrors = &manifest.runtimes[0].mirrors;

    assert_eq!(mirrors.len(), 2);
    assert_eq!(mirrors[0].name, "taobao");
    assert_eq!(mirrors[0].region, Some("cn".to_string()));
    assert_eq!(mirrors[0].priority, 100);
    assert!(mirrors[0].enabled);

    assert_eq!(mirrors[1].name, "ustc");
    assert!(!mirrors[1].enabled);
}

#[test]
fn test_parse_cache_config() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.cache]
versions_ttl = 7200
cache_downloads = true
downloads_retention_days = 14
max_cache_size_mb = 1024
shared_cache = false
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let cache = manifest.runtimes[0].cache.as_ref().unwrap();

    assert_eq!(cache.versions_ttl, 7200);
    assert!(cache.cache_downloads);
    assert_eq!(cache.downloads_retention_days, 14);
    assert_eq!(cache.max_cache_size_mb, Some(1024));
    assert!(!cache.shared_cache);
}

#[test]
fn test_parse_priority_and_auto_installable() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "node"
executable = "node"
priority = 100
auto_installable = true

[[runtimes]]
name = "internal-tool"
executable = "itool"
priority = 50
auto_installable = false
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();

    assert_eq!(manifest.runtimes[0].priority, Some(100));
    assert_eq!(manifest.runtimes[0].auto_installable, Some(true));

    assert_eq!(manifest.runtimes[1].priority, Some(50));
    assert_eq!(manifest.runtimes[1].auto_installable, Some(false));
}

#[test]
fn test_env_config_get_vars_for_version() {
    let mut env_config = EnvConfig::default();
    env_config
        .vars
        .insert("PATH".to_string(), "{install_dir}/bin".to_string());
    env_config.conditional.insert(
        ">=18".to_string(),
        [
            (
                "NODE_OPTIONS".to_string(),
                "--experimental-vm-modules".to_string(),
            ),
        ]
        .into_iter()
        .collect(),
    );
    env_config.conditional.insert(
        "<16".to_string(),
        [
            (
                "NODE_OPTIONS".to_string(),
                "--experimental-modules".to_string(),
            ),
        ]
        .into_iter()
        .collect(),
    );

    let vars_v20 = env_config.get_vars_for_version("20.0.0");
    assert!(vars_v20.contains_key("PATH"));
    assert_eq!(
        vars_v20.get("NODE_OPTIONS"),
        Some(&"--experimental-vm-modules".to_string())
    );

    let vars_v14 = env_config.get_vars_for_version("14.0.0");
    assert!(vars_v14.contains_key("PATH"));
    assert_eq!(
        vars_v14.get("NODE_OPTIONS"),
        Some(&"--experimental-modules".to_string())
    );
}

#[test]
fn test_parse_custom_commands() {
    let toml = r#"
[provider]
name = "node"

[[runtimes]]
name = "node"
executable = "node"

[[runtimes.commands]]
name = "repl"
description = "Start interactive REPL"
command = "{executable}"
category = "development"

[[runtimes.commands]]
name = "eval"
description = "Evaluate JavaScript expression"
command = "{executable} -e"
pass_args = true

[[runtimes.commands]]
name = "doctor"
description = "Diagnose installation"
script = "scripts/doctor.sh"
category = "maintenance"

[[runtimes.commands]]
name = "internal"
command = "{executable} --internal"
hidden = true
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let runtime = &manifest.runtimes[0];

    assert_eq!(runtime.commands.len(), 4);

    let repl = runtime.get_command("repl").unwrap();
    assert_eq!(repl.description, Some("Start interactive REPL".to_string()));
    assert_eq!(repl.command, Some("{executable}".to_string()));
    assert_eq!(repl.category, Some("development".to_string()));
    assert!(!repl.pass_args);
    assert!(!repl.hidden);

    let eval = runtime.get_command("eval").unwrap();
    assert!(eval.pass_args);

    let doctor = runtime.get_command("doctor").unwrap();
    assert_eq!(doctor.script, Some("scripts/doctor.sh".to_string()));
    assert!(doctor.command.is_none());

    let internal = runtime.get_command("internal").unwrap();
    assert!(internal.hidden);

    let visible = runtime.visible_commands();
    assert_eq!(visible.len(), 3);
    assert!(visible.iter().all(|c| !c.hidden));

    let dev_commands = runtime.commands_by_category("development");
    assert_eq!(dev_commands.len(), 1);
    assert_eq!(dev_commands[0].name, "repl");

    let maint_commands = runtime.commands_by_category("maintenance");
    assert_eq!(maint_commands.len(), 1);
    assert_eq!(maint_commands[0].name, "doctor");
}

#[test]
fn test_command_def_builder() {
    let cmd = CommandDef::new("test")
        .with_command("echo hello")
        .with_description("Test command")
        .with_pass_args();

    assert_eq!(cmd.name, "test");
    assert_eq!(cmd.command, Some("echo hello".to_string()));
    assert_eq!(cmd.description, Some("Test command".to_string()));
    assert!(cmd.pass_args);
    assert!(cmd.is_valid());

    let invalid = CommandDef::new("invalid");
    assert!(!invalid.is_valid());
}

#[test]
fn test_parse_output_config() {
    let toml = r#"
[provider]
name = "node"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.output]
list_format = "{version:>12} {lts:>10} {installed:>10} {date}"
status_format = "{name} {version} ({path})"
formats = ["text", "json", "csv", "table"]
default_format = "text"

[runtimes.output.machine_flags]
list = "--json"
info = "--json"
status = "--json"

[runtimes.output.colors]
lts = "green"
current = "cyan"
installed = "blue"
outdated = "yellow"
error = "red"
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let output = manifest.runtimes[0].output.as_ref().unwrap();

    assert_eq!(
        output.list_format,
        Some("{version:>12} {lts:>10} {installed:>10} {date}".to_string())
    );
    assert_eq!(
        output.status_format,
        Some("{name} {version} ({path})".to_string())
    );
    assert_eq!(output.formats, vec!["text", "json", "csv", "table"]);
    assert_eq!(output.default_format, Some("text".to_string()));

    let flags = output.machine_flags.as_ref().unwrap();
    assert_eq!(flags.list, Some("--json".to_string()));
    assert_eq!(flags.info, Some("--json".to_string()));

    let colors = output.colors.as_ref().unwrap();
    assert_eq!(colors.lts, Some("green".to_string()));
    assert_eq!(colors.current, Some("cyan".to_string()));
    assert_eq!(colors.error, Some("red".to_string()));
}

#[test]
fn test_output_config_defaults() {
    let config = OutputConfig::default();

    assert_eq!(
        config.list_format_or_default(),
        "{version:>12} {installed:>10}"
    );
    assert_eq!(config.status_format_or_default(), "{name} {version}");
    assert!(config.supports_format("text"));
    assert!(config.supports_format("json"));
    assert!(!config.supports_format("yaml"));

    let config_with_formats = OutputConfig {
        formats: vec!["json".to_string(), "yaml".to_string()],
        ..Default::default()
    };
    assert!(config_with_formats.supports_format("json"));
    assert!(config_with_formats.supports_format("yaml"));
    assert!(!config_with_formats.supports_format("text"));
}

#[test]
fn test_parse_shell_config() {
    let toml = r#"
[provider]
name = "node"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.shell]
prompt_format = "(node-{version})"
activate_template = "templates/activate.sh"
deactivate_template = "templates/deactivate.sh"

[runtimes.shell.completions]
bash = "completions/node.bash"
zsh = "completions/_node"
fish = "completions/node.fish"
powershell = "completions/node.ps1"

[runtimes.shell.aliases]
n = "node"
nr = "npm run"
nrd = "npm run dev"
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let shell = manifest.runtimes[0].shell.as_ref().unwrap();

    assert_eq!(shell.prompt_format, Some("(node-{version})".to_string()));
    assert_eq!(
        shell.activate_template,
        Some("templates/activate.sh".to_string())
    );
    assert_eq!(
        shell.deactivate_template,
        Some("templates/deactivate.sh".to_string())
    );

    let completions = shell.completions.as_ref().unwrap();
    assert_eq!(completions.bash, Some("completions/node.bash".to_string()));
    assert_eq!(completions.zsh, Some("completions/_node".to_string()));
    assert_eq!(completions.fish, Some("completions/node.fish".to_string()));
    assert_eq!(
        completions.powershell,
        Some("completions/node.ps1".to_string())
    );

    assert_eq!(completions.for_shell("bash"), Some("completions/node.bash"));
    assert_eq!(completions.for_shell("ZSH"), Some("completions/_node"));
    assert_eq!(completions.for_shell("pwsh"), Some("completions/node.ps1"));
    assert_eq!(completions.for_shell("tcsh"), None);

    let shells = completions.configured_shells();
    assert_eq!(shells.len(), 4);
    assert!(shells.contains(&"bash"));
    assert!(shells.contains(&"zsh"));

    assert_eq!(shell.aliases.len(), 3);
    assert_eq!(shell.aliases.get("n"), Some(&"node".to_string()));
    assert_eq!(shell.aliases.get("nr"), Some(&"npm run".to_string()));
}

#[test]
fn test_shell_config_format_prompt() {
    let config = ShellConfig {
        prompt_format: Some("({name}-{version})".to_string()),
        ..Default::default()
    };

    let prompt = config.format_prompt("20.0.0", "node");
    assert_eq!(prompt, Some("(node-20.0.0)".to_string()));

    let empty = ShellConfig::default();
    assert!(empty.format_prompt("1.0.0", "test").is_none());
    assert!(empty.is_empty());
}

#[test]
fn test_parse_test_config() {
    let toml = r#"
[provider]
name = "node"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.test]
timeout_ms = 60000
skip_on = ["ci-windows"]

[[runtimes.test.functional_commands]]
command = "{executable} --version"
expect_success = true
name = "version check"

[[runtimes.test.functional_commands]]
command = "{executable} -e 'console.log(1)'"
expected_output = "1"

[[runtimes.test.install_verification]]
command = "{executable} -e 'process.exit(0)'"
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let test = manifest.runtimes[0].test.as_ref().unwrap();

    assert_eq!(test.timeout_ms, 60000);
    assert_eq!(test.skip_on, vec!["ci-windows"]);
    assert_eq!(test.functional_commands.len(), 2);
    assert_eq!(test.install_verification.len(), 1);

    let cmd1 = &test.functional_commands[0];
    assert_eq!(cmd1.command, "{executable} --version");
    assert!(cmd1.expect_success);
    assert_eq!(cmd1.name, Some("version check".to_string()));

    let cmd2 = &test.functional_commands[1];
    assert_eq!(cmd2.expected_output, Some("1".to_string()));

    assert!(test.should_skip("ci-windows"));
    assert!(!test.should_skip("ci-linux"));
}

#[test]
fn test_parse_test_config_with_platforms() {
    let toml = r#"
[provider]
name = "node"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.test]
[[runtimes.test.functional_commands]]
command = "{executable} --version"

[runtimes.test.platforms.windows]
[[runtimes.test.platforms.windows.functional_commands]]
command = "{executable} -e \"console.log('win32')\""
expected_output = "win32"

[runtimes.test.platforms.unix]
[[runtimes.test.platforms.unix.functional_commands]]
command = "{executable} -e \"console.log('unix')\""
expected_output = "unix"
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let test = manifest.runtimes[0].test.as_ref().unwrap();

    assert!(test.platforms.is_some());
    let platforms = test.platforms.as_ref().unwrap();

    let win = platforms.windows.as_ref().unwrap();
    assert_eq!(win.functional_commands.len(), 1);
    assert_eq!(
        win.functional_commands[0].expected_output,
        Some("win32".to_string())
    );

    let unix = platforms.unix.as_ref().unwrap();
    assert_eq!(unix.functional_commands.len(), 1);
}

#[test]
fn test_parse_test_config_with_inline_scripts() {
    let toml = r#"
[provider]
name = "node"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.test.inline_scripts]
windows = """
@echo off
{executable} --version
"""
unix = """
#!/bin/sh
{executable} --version
"""
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let test = manifest.runtimes[0].test.as_ref().unwrap();

    let scripts = test.inline_scripts.as_ref().unwrap();
    assert!(scripts.windows.is_some());
    assert!(scripts.unix.is_some());

    assert!(scripts.windows.as_ref().unwrap().contains("@echo off"));
    assert!(scripts.unix.as_ref().unwrap().contains("#!/bin/sh"));
}

#[test]
fn test_test_command_builder() {
    let cmd = TestCommand::new("{executable} --version")
        .with_expected_output(r"v\d+\.\d+\.\d+")
        .with_exit_code(0);

    assert_eq!(cmd.command, "{executable} --version");
    assert_eq!(cmd.expected_output, Some(r"v\d+\.\d+\.\d+".to_string()));
    assert_eq!(cmd.expected_exit_code, Some(0));
    assert_eq!(cmd.display_name(), "{executable} --version");
}

#[test]
fn test_test_config_has_tests() {
    let empty = TestConfig::default();
    assert!(!empty.has_tests());

    let with_cmds = TestConfig {
        functional_commands: vec![TestCommand::new("test")],
        ..Default::default()
    };
    assert!(with_cmds.has_tests());

    let with_scripts = TestConfig {
        inline_scripts: Some(InlineTestScripts {
            unix: Some("echo test".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(with_scripts.has_tests());
}
