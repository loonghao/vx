# Tiled

通过 `tiled` Provider 使用 [Tiled](https://www.mapeditor.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `tiled` | MSI 提取到 vx 存储目录，x64 | AppImage，x64 | 通用应用 ZIP，x64 / ARM64 |

```bash
vx tiled --version
```

自动安装检查仅确认图形应用的可执行文件存在，不会启动应用。应用运行及 DCC-MCP 适配器连接仍需在真实应用环境中验证。

此 Provider 支持从 1.12.0 起的指定版本发行包。Windows 发行包要求 Windows 10 或更新版本，安装时会提取到 vx 存储目录。macOS 发行包要求 macOS 13 或更新版本，其中 1.12.0 支持 macOS 11。Linux AppImage 要求兼容的系统库及 AppImage/FUSE 支持。

Provider 会在运行环境中设置 `DCC_MCP_TILED_EXECUTABLE`。应用继续遵循上游许可证。安装应用不会自动安装或连接 DCC-MCP 适配器。使用 `vx where tiled` 获取可执行文件路径，再按 [DCC-MCP 集成指南](../guide/dcc-mcp.md)完成适配器配置。
