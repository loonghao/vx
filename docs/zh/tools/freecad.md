# FreeCAD

通过 `freecad` Provider 使用 [FreeCAD](https://www.freecad.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `freecad` | 便携 7z，x64，FreeCAD 1.x | AppImage，x64 / ARM64，FreeCAD 1.x | Homebrew cask / 已安装的应用 |
| `freecadcmd` | 随 `freecad` 提供的控制台程序 | 不提供 | 不提供 |

```bash
# Windows：执行 Python 脚本，无需启动图形界面。
vx freecadcmd model.py
vx where freecadcmd
```

`FreeCADCmd` 和 `freecad-cmd` 是 `freecadcmd` 的别名。Windows 安装检查同时要求图形界面与控制台程序存在；图形界面检查仅验证文件，不会打开 Qt。Linux AppImage 需要兼容的系统库和 AppImage/FUSE 支持。

FreeCAD 保留其上游许可证。在 Windows 上使用 DCC-MCP 无界面适配器时，将 `DCC_MCP_FREECAD_EXECUTABLE` 设置为 `vx where freecadcmd` 返回的路径。安装 FreeCAD 后，仍需单独安装并连接适配器。Linux AppImage 不提供 `freecadcmd` Runtime。
