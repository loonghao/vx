# OpenSCAD

通过 `openscad` Provider 使用 [OpenSCAD](https://openscad.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `openscad` | 便携 ZIP，x64 / x86 | AppImage，x64 | Homebrew cask / 已安装的应用 |

```bash
vx openscad --version
vx openscad -o part.stl part.scad
```

Windows 使用便携包中的 `openscad.com` 控制台入口。Linux 上，vx 会在安装时提取 AppImage，并运行 `squashfs-root/AppRun`，无需 FUSE。提取后的应用仍需兼容的宿主系统库，打开 GUI 还需要显示环境。macOS 使用 Homebrew cask 或已有的应用安装。

自动验收会对解析出的可执行文件运行 `--version`，并要求输出 OpenSCAD 版本。只有下载的 AppImage、缺少提取后启动入口的 Linux 缓存会被视为安装不完整。几何导出仍需使用实际输入模型单独验证。

OpenSCAD 保留其上游许可证。使用 `vx where openscad` 获取可执行文件路径，再遵循 DCC-MCP 适配器的安装契约。安装 OpenSCAD 后，仍需单独安装并连接适配器。
