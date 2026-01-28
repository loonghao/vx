use vx_manifest::ProviderManifest;

fn main() {
    // Test 1: Minimal manifest (should work)
    let test1 = r#"
[provider]
name = "pwsh"
description = "Test"
ecosystem = "system"

[[runtimes]]
name = "pwsh"
executable = "pwsh"
"#;
    test_parse("Test 1 (minimal)", test1);

    // Test 2: With versions
    let test2 = r#"
[provider]
name = "pwsh"
description = "Test"
ecosystem = "system"

[[runtimes]]
name = "pwsh"
executable = "pwsh"

[runtimes.versions]
source = "github-releases"
owner = "PowerShell"
repo = "PowerShell"
strip_v_prefix = true
"#;
    test_parse("Test 2 (with versions)", test2);

    // Test 3: With layout
    let test3 = r#"
[provider]
name = "pwsh"
description = "Test"
ecosystem = "system"

[[runtimes]]
name = "pwsh"
executable = "pwsh"

[runtimes.versions]
source = "github-releases"
owner = "PowerShell"
repo = "PowerShell"
strip_v_prefix = true

[runtimes.layout]
download_type = "archive"
"#;
    test_parse("Test 3 (with layout)", test3);

    // Test 4: With layout.archive
    let test4 = r#"
[provider]
name = "pwsh"
description = "Test"
ecosystem = "system"

[[runtimes]]
name = "pwsh"
executable = "pwsh"

[runtimes.versions]
source = "github-releases"
owner = "PowerShell"
repo = "PowerShell"
strip_v_prefix = true

[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
executable_paths = ["pwsh.exe", "pwsh"]
"#;
    test_parse("Test 4 (with layout.archive)", test4);

    // Test 5: With platforms
    let test5 = r#"
[provider]
name = "pwsh"
description = "Test"
ecosystem = "system"

[[runtimes]]
name = "pwsh"
executable = "pwsh"

[runtimes.versions]
source = "github-releases"
owner = "PowerShell"
repo = "PowerShell"
strip_v_prefix = true

[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
executable_paths = ["pwsh.exe", "pwsh"]

[runtimes.platforms.windows]
executable_extensions = [".exe"]
"#;
    test_parse("Test 5 (with platforms.windows)", test5);
}

fn test_parse(name: &str, toml: &str) {
    match ProviderManifest::parse(toml) {
        Ok(_) => println!("{}: OK", name),
        Err(e) => eprintln!("{}: FAILED - {}", name, e),
    }
}
