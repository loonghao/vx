# OpenSCAD

[OpenSCAD](https://openscad.org) is available through the `openscad` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `openscad` | Portable ZIP, x64 / x86 | AppImage, x64 | Homebrew development snapshot / installed app |

```bash
vx openscad --version
vx openscad -o part.stl part.scad
```

Windows uses the `openscad.com` console entry point from the portable archive. On Linux, vx extracts the AppImage during installation and runs `squashfs-root/AppRun`, so FUSE is not required. The extracted application still needs compatible host libraries; opening the GUI also needs a display.

On macOS, installation resolves the `system` version and uses the officially recommended [`openscad@snapshot` Homebrew cask](https://openscad.org/downloads.html). This is a development snapshot selected by Homebrew, not a pinned stable release such as 2021.01. The [cask requires macOS 12 or newer](https://formulae.brew.sh/cask/openscad@snapshot) and conflicts with the stable `openscad` cask. The resolved application entry point is `/Applications/OpenSCAD.app/Contents/MacOS/OpenSCAD`; an existing application can also be discovered.

Automated validation runs `--version` against the resolved executable and requires OpenSCAD version output. A downloaded Linux AppImage without its extracted launcher is treated as an incomplete installation. Validate geometry exports separately with your input models.

OpenSCAD retains its upstream license. Resolve the executable with `vx where openscad`, then follow the DCC-MCP adapter's installation contract. Installing OpenSCAD does not install or connect its adapter.
