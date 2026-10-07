# FreeCAD

[FreeCAD](https://www.freecad.org) is available through the `freecad` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `freecad` | Portable 7z, x64, FreeCAD 1.x | AppImage, x64 / ARM64, FreeCAD 1.x | Homebrew cask / installed app |
| `freecadcmd` | Console executable bundled with `freecad` | Unavailable | Unavailable |

```bash
# Windows: execute Python without starting the GUI.
vx freecadcmd model.py
vx where freecadcmd
```

`FreeCADCmd` and `freecad-cmd` are aliases for `freecadcmd`. Windows installation checks require both the GUI and console executables; the GUI check verifies the file without opening Qt. Linux AppImages require compatible system libraries and AppImage/FUSE support.

FreeCAD retains its upstream license. For the Windows DCC-MCP headless adapter, set `DCC_MCP_FREECAD_EXECUTABLE` to the path returned by `vx where freecadcmd`. Installing FreeCAD does not install or connect its adapter. The Linux AppImage is not exposed as a `freecadcmd` Runtime.
