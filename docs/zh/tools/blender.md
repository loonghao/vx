# Blender

通过 `blender` Provider 使用 [Blender](https://www.blender.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `blender` | 便携 ZIP，x64；4.4 起支持 ARM64 | 便携 tar.xz，x64 | Homebrew cask / 已安装的应用 |

```bash
vx blender --version
vx blender --background --python scene.py
```

便携下载支持 Blender 3.x 及更新版本。Windows ARM64 需要 Blender 4.4 或更新版本。macOS 使用 Homebrew cask 或已有的应用安装。

Blender 保留其上游许可证。使用 `vx where blender` 获取可执行文件路径，再遵循 DCC-MCP 适配器的安装契约。安装 Blender 后，仍需单独安装并连接适配器。
