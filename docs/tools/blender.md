# Blender

[Blender](https://www.blender.org) is available through the `blender` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `blender` | Portable ZIP, x64; ARM64 from 4.4 | Portable tar.xz, x64 | Homebrew cask / installed app |

```bash
vx blender --version
vx blender --background --python scene.py
```

Portable downloads support Blender 3.x and later. Windows ARM64 requires Blender 4.4 or later. macOS uses the Homebrew cask or an existing application installation.

Blender retains its upstream license. Resolve the executable with `vx where blender`, then follow the DCC-MCP adapter's installation contract. Installing Blender does not install or connect its adapter.
