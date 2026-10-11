# OBS Studio

通过 `obs` Provider 使用 [OBS Studio](https://obsproject.com)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `obs` | 便携 ZIP，x64 / ARM64 | Debian/Ubuntu APT 软件包，x64 / ARM64 | Homebrew cask / 已安装的应用，x64 / ARM64 |

```bash
vx obs --version
```

Windows ARM64 发行包要求 OBS 32 或更新版本。在 Debian/Ubuntu 上，Linux 安装会使用已配置 APT 仓库中的 `obs-studio` 软件包，需要 root 或 sudo 权限。[Ubuntu 官方仓库](https://packages.ubuntu.com/noble/obs-studio)提供发行版软件包；vx 不会自动添加[上游 Linux 指南](https://obsproject.com/kb/linux-installation)中的 OBS PPA。macOS 使用 Homebrew 的 `obs` cask 或已安装的应用。系统包管理器会选择其提供的版本，无法保证与 vx 中指定的版本一致。

应用继续遵循上游许可证。安装应用不会自动安装或连接 DCC-MCP 适配器。使用 `vx where obs` 获取可执行文件路径，再按 DCC-MCP 适配器安装说明完成适配器配置。
