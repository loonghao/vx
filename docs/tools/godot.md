# Godot

[Godot](https://godotengine.org) is available through the `godot` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `godot` | Portable ZIP, x64 / x86 / ARM64 | Portable ZIP, x64 / x86 / ARM64 / ARMv7 | Universal app ZIP, x64 / ARM64 |

```bash
vx godot --headless --version
vx godot --headless --path game --editor --quit
```

The Provider installs the standard editor distribution. Windows executes the editor directly and keeps its companion console wrapper beside it. Upstream command-line options pass through vx unchanged, including `--headless --version`. Stable release versions work with or without the `-stable` suffix.

Godot retains its upstream license. Resolve the executable with `vx where godot`, then follow the DCC-MCP adapter's installation contract. Installing Godot does not install or connect its adapter.
