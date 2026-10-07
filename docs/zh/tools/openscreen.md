# OpenScreen

通过 `openscreen` Provider 使用 [OpenScreen](https://getopenscreen.com)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `openscreen` | 静默用户安装到 vx store，x64 | AppImage, x64 | 应用 ZIP，x64 / ARM64 |

```bash
vx openscreen --help
```

从 2.0 开始支持官方发行包与无界面的帮助命令。Windows 使用静默用户安装器，安装目录限制在 vx store 内。

应用保留其上游许可证。安装应用后，需要单独安装并连接 DCC-MCP 适配器。使用 `vx where openscreen` 获取可执行文件路径，再遵循适配器的安装契约。
