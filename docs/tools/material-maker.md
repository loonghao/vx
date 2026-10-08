# Material Maker

[Material Maker](https://www.materialmaker.org) is available through the `material-maker` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `material-maker` | Portable ZIP, x64 | Portable tar.gz, x64 | Homebrew cask / installed app |

```bash
vx material-maker --headless --version
vx where material-maker
```

`material_maker` and `materialmaker` are aliases. The headless version command reports the embedded Godot engine version; the application package version follows upstream release tags. macOS uses the [Homebrew cask](https://github.com/Homebrew/homebrew-cask/blob/main/Casks/m/material-maker.rb), which installs the upstream DMG as `/Applications/Material Maker.app`, or discovers an existing installation there.

Material Maker retains its upstream license. Resolve the executable with `vx where material-maker`, then follow the DCC-MCP adapter's installation contract. Installing Material Maker does not install or connect its adapter.
