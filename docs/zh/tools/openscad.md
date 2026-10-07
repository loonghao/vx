# OpenSCAD

通过 `openscad` Provider 使用 [OpenSCAD](https://openscad.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `openscad` | 便携 ZIP，x64 / x86 | AppImage，x64 | Homebrew cask / 已安装的应用 |

```bash
vx openscad --version
vx openscad -o part.stl part.scad
```

Windows 使用便携包中的 `openscad.com` 控制台入口。Linux AppImage 需要兼容的系统库和 AppImage/FUSE 支持。macOS 使用 Homebrew cask 或已有的应用安装。

OpenSCAD 保留其上游许可证。使用 `vx where openscad` 获取可执行文件路径，再遵循 DCC-MCP 适配器的安装契约。安装 OpenSCAD 后，仍需单独安装并连接适配器。
