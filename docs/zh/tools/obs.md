# OBS Studio

通过 `obs` Provider 使用 [OBS Studio](https://obsproject.com)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `obs` | 便携 ZIP，x64 / ARM64 | APT 软件包，x64 / ARM64 | Homebrew cask / 已安装的应用，x64 / ARM64 |

```bash
vx obs --version
```

Windows ARM64 发行包要求 OBS 32 或更新版本。Linux 使用 APT 的 `obs-studio` 软件包；macOS 使用 Homebrew 的 `obs` cask 或已安装的应用。系统包管理器会选择其提供的版本，无法保证与 vx 中指定的版本一致。

应用继续遵循上游许可证。安装应用不会自动安装或连接 DCC-MCP 适配器。使用 `vx where obs` 获取可执行文件路径，再按 [DCC-MCP 集成指南](../guide/dcc-mcp.md)完成适配器配置。
