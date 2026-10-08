# Kdenlive

通过 `kdenlive` Provider 使用 [Kdenlive](https://kdenlive.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `kdenlive` | 官方独立版自解压归档，x64 | AppImage，x64 | Homebrew cask / 已安装的应用，x64 / ARM64 |

```bash
vx kdenlive --version
```

自动安装检查仅确认图形应用的可执行文件存在，不会启动应用。应用运行及 DCC-MCP 适配器连接仍需在真实应用环境中验证。

`latest` 根据[官方下载页](https://kdenlive.org/download/)链接的对应平台已发布二进制文件选择版本，因为源码标签可能早于下载文件发布。Windows/Linux 指定版本时保留请求的版本，并要求 KDE 稳定下载区存在对应文件；vx 不会替换为较旧版本。指定版本支持从 24.04 起的 `.04`、`.08` 和 `.12` 稳定发行系列，并排除测试版和候选发行版。macOS 使用 Homebrew cask 或已安装的应用，版本不由 vx 固定。

Windows 使用官方独立版归档，包含 melt、FFprobe 和 FFmpeg。Provider 会在运行环境中设置 `DCC_MCP_KDENLIVE_EXECUTABLE`，并在 Windows 上额外设置 `DCC_MCP_KDENLIVE_MELT`、`DCC_MCP_KDENLIVE_FFPROBE` 和 `DCC_MCP_KDENLIVE_FFMPEG`。Linux/macOS 连接适配器时，需要另行配置渲染器和探测程序的路径。Linux AppImage 要求兼容的系统库及 AppImage/FUSE 支持；Qt 版本检查还需要显示服务或 Qt 离屏平台。

应用继续遵循上游许可证。安装应用不会自动安装或连接 DCC-MCP 适配器。使用 `vx where kdenlive` 获取可执行文件路径，再按 DCC-MCP 适配器安装说明完成适配器配置。
