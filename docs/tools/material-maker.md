# Material Maker

[Material Maker](https://www.materialmaker.org) is available through the `material-maker` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `material-maker` | Portable ZIP, x64 | Portable tar.gz, x64 | Existing application discovery |

```bash
vx material-maker --headless --version
vx where material-maker
```

`material_maker` and `materialmaker` are aliases. The headless version command reports the embedded Godot engine version; the application package version follows upstream release tags. macOS uses an existing installation at `/Applications/Material Maker.app`.

Material Maker retains its upstream license. Resolve the executable with `vx where material-maker`, then follow the DCC-MCP adapter's installation contract. Installing Material Maker does not install or connect its adapter.
